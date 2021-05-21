use crate::docker::{self, ExecuteInfo};
use crate::packages;
use anyhow::{Context, Result};
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Password};
use dialoguer::{Input as Prompt, Select};
use serde::de::DeserializeOwned;
use specifications::common::{Function, Parameter, Type, Value};
use specifications::package::PackageInfo;
use std::fs;
use std::path::PathBuf;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

type Map<T> = std::collections::HashMap<String, T>;

const PACKAGE_NOT_FOUND: &str = "Package not found.";
const UNSUPPORTED_PACKAGE_KIND: &str = "Package kind not supported.";

///
///
///
pub async fn handle(
    name: String,
    version: Option<String>,
    data: Option<PathBuf>,
) -> Result<()> {
    let version_or_latest = version.unwrap_or_else(|| String::from("latest"));
    let package_dir = packages::get_package_dir(&name, Some(&version_or_latest))?;
    if !package_dir.exists() {
        return Err(anyhow!(PACKAGE_NOT_FOUND));
    }

    let package_info = PackageInfo::from_path(package_dir.join("package.yml"))?;
    let output = match package_info.kind.as_str() {
        "ecu" => test_generic("code", package_dir, package_info, data).await?,
        "oas" => test_generic("oas", package_dir, package_info, data).await?,
        _ => {
            return Err(anyhow!(UNSUPPORTED_PACKAGE_KIND));
        }
    };

    print_output(&output);

    Ok(())
}

///
///
///
pub async fn test_generic(
    package_kind: &str,
    package_dir: PathBuf,
    package_info: PackageInfo,
    data: Option<PathBuf>,
) -> Result<Value> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap_or_default();
    let (function, arguments) = prompt_for_input(&functions, &types)?;

    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = Some(package_dir.join("image.tar"));

    let command = vec![
        String::from("-d"),
        String::from("--application-id"),
        String::from("test"),
        String::from("--location-id"),
        String::from("localhost"),
        String::from("--job-id"),
        String::from("1"),
        package_kind.to_string(),
        function,
        base64::encode(serde_json::to_string(&arguments)?),
    ];

    let mounts = if let Some(data) = data {
        let data = fs::canonicalize(data)?;
        if data.exists() {
            Some(vec![format!("{}:/data", data.into_os_string().into_string().unwrap())])
        } else {
            None
        }
    } else {
        None
    };

    let exec = ExecuteInfo::new(image, image_file, mounts, Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    debug!("stderr: {}", stderr);
    debug!("stdout: {}", stdout);

    let output = stdout.lines().last().unwrap_or_default().to_string();
    match decode_b64(output) {
        Ok(value) => Ok(value),
        Err(err) => {
            println!("{:?}", err);
            Ok(Value::Unit)
        }
    }
}

///
///
///
fn prompt_for_input(
    functions: &Map<Function>,
    types: &Map<Type>,
) -> Result<(String, Map<Value>)> {
    let mut function_list: Vec<String> = functions.keys().map(|k| k.to_string()).collect();
    function_list.sort();
    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("The function the execute")
        .default(0)
        .items(&function_list[..])
        .interact()?;

    let function_name = &function_list[index];
    let function = &functions[function_name];

    println!("\nPlease provide input for the chosen function:\n");

    let mut arguments = Map::<Value>::new();
    for p in &function.parameters {
        let data_type = p.data_type.as_str();

        debug!("{:?}", types);
        let value = if let Some(input_type) = types.get(data_type) {
            let mut properties = Map::<Value>::new();

            for p in &input_type.properties {
                let p = p.clone().into_parameter();
                let data_type = p.data_type.as_str();

                let value = prompt_for_value(data_type, &p)?;
                properties.insert(p.name.clone(), value);
            }

            Value::Struct {
                data_type: input_type.name.clone(),
                properties,
            }
        } else {
            prompt_for_value(data_type, p)?
        };

        arguments.insert(p.name.clone(), value);
    }

    debug!("Arguments: {:#?}", arguments);

    println!();

    Ok((function_name.clone(), arguments))
}

///
///
///
fn prompt_for_value(
    data_type: &str,
    p: &Parameter,
) -> Result<Value> {
    let value = if data_type.ends_with("[]") {
        let entry_data_type = data_type[..data_type.len() - 2].to_string();
        let mut entries = vec![];

        loop {
            let mut p = p.clone();
            p.data_type = format!("{}[{}]", entry_data_type, entries.len());
            entries.push(prompt_for_value(&entry_data_type, &p)?);

            if !Confirm::new()
                .with_prompt(format!(
                    "Do you want to more items to the {} array?",
                    style(p.name).bold().cyan()
                ))
                .interact()?
            {
                break;
            }
        }

        Value::Array {
            data_type: data_type.to_string(),
            entries,
        }
    } else {
        match data_type {
            "boolean" => {
                let default = p.clone().default.map(|d| d.as_bool().unwrap());
                Value::Boolean(prompt(&p, default)?)
            }
            "Directory" | "File" => {
                let default = p.clone().default.map(|d| d.as_string().unwrap());
                let url = Value::Unicode(format!("file:///{}", prompt(&p, default)?));

                let mut properties = Map::<Value>::default();
                properties.insert(String::from("url"), url);

                Value::Struct {
                    data_type: String::from(data_type),
                    properties,
                }
            }
            "integer" => {
                let default = p.clone().default.map(|d| d.as_i64().unwrap());
                Value::Integer(prompt(&p, default)?)
            }
            "real" => {
                let default = p.clone().default.map(|d| d.as_f64().unwrap());
                Value::Real(prompt(&p, default)?)
            }
            "string" => {
                let default = p.clone().default.map(|d| d.as_string().unwrap());
                let value = if p.name.to_lowercase().contains("password") {
                    prompt_password(&p, default)?
                } else {
                    prompt(&p, default)?
                };

                Value::Unicode(value)
            }
            _ => {
                error!("Unreachable, because data type is '{}'", data_type);
                unreachable!()
            }
        }
    };

    Ok(value)
}

///
///
///
fn prompt<T>(
    parameter: &Parameter,
    default: Option<T>,
) -> std::io::Result<T>
where
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    let colorful = ColorfulTheme::default();
    let mut prompt = Prompt::with_theme(&colorful);
    prompt
        .with_prompt(&format!("{} ({})", parameter.name, parameter.data_type))
        .allow_empty(parameter.optional.unwrap_or_default());

    if let Some(default) = default {
        prompt.default(default);
    }

    prompt.interact()
}

///
///
///
fn prompt_password(
    parameter: &Parameter,
    _default: Option<String>,
) -> std::io::Result<String> {
    let colorful = ColorfulTheme::default();
    let mut prompt = Password::with_theme(&colorful);
    prompt
        .with_prompt(&format!("{} ({})", parameter.name, parameter.data_type))
        .allow_empty_password(parameter.optional.unwrap_or_default());

    prompt.interact()
}

///
///
///
fn print_output(value: &Value) -> () {
    match value {
        Value::Array { entries, .. } => {
            println!("{}", style("[").bold().cyan());
            for entry in entries {
                println!("   {}", style(entry).bold().cyan());
            }
            println!("{}", style("]").bold().cyan());
        }
        Value::Boolean(boolean) => println!("{}", style(boolean).bold().cyan()),
        Value::Integer(integer) => println!("{}", style(integer).bold().cyan()),
        Value::Real(real) => println!("{}", style(real).bold().cyan()),
        Value::Unicode(unicode) => println!("{}", style(unicode).bold().cyan()),
        Value::Unit => println!("_ (unit)"),
        Value::Pointer { .. } => unreachable!(),
        Value::Struct { properties, .. } => {
            for (name, value) in properties.iter() {
                println!("{}:", style(name).bold().cyan());
                println!("{}\n", style(value).cyan());
            }
        }
    }
}

///
///
///
fn decode_b64<T>(input: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let input =
        base64::decode(input).with_context(|| "Decoding failed, encoded input doesn't seem to be Base64 encoded.")?;

    let input = String::from_utf8(input[..].to_vec())
        .with_context(|| "Conversion failed, decoded input doesn't seem to be UTF-8 encoded.")?;

    serde_json::from_str(&input)
        .with_context(|| "Deserialization failed, decoded input doesn't seem to be as expected.")
}
