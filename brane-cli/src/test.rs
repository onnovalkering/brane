use crate::packages;
use anyhow::Result;
use brane_vm::machine::{self, Machine};
use brane_exec::{docker, ExecuteInfo};
use brane_vm::environment::InMemoryEnvironment;
use brane_vm::vault::InMemoryVault;
use brane_sys::local::LocalSystem;
use console::style;
use dialoguer::{Confirm, Password};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input as Prompt, Select};
use serde_yaml;
use specifications::common::{Function, Parameter, Type, Value};
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::fs::File;
use std::io::{BufReader};
use std::path::PathBuf;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use url::Url;
use uuid::Uuid;
use std::env;
use std::fs;

type Map<T> = std::collections::HashMap<String, T>;

const PACKAGE_NOT_FOUND: &str = "Package not found.";
const UNSUPPORTED_PACKAGE_KIND: &str = "Package kind not supported.";

///
///
///
pub async fn handle(
    name: String,
    version: Option<String>,
) -> Result<()> {
    let version_or_latest = version.unwrap_or_else(|| String::from("latest"));
    let package_dir = packages::get_package_dir(&name, Some(&version_or_latest))?;
    if !package_dir.exists() {
        return Err(anyhow!(PACKAGE_NOT_FOUND));
    }

    let package_info = PackageInfo::from_path(package_dir.join("package.yml"))?;
    let output = match package_info.kind.as_str() {
        "cwl" => test_cwl(package_dir, package_info).await?,
        "dsl" => test_dsl(package_dir, package_info)?,
        "ecu" => test_ecu(package_dir, package_info).await?,
        "oas" => test_oas(package_dir, package_info).await?,
        _ => {
            return Err(anyhow!(UNSUPPORTED_PACKAGE_KIND));
        }
    };

    print_output(&output);

    Ok(())
}

fn url_to_path(url: &Url) -> Result<PathBuf> {
    Ok(PathBuf::from(url.path()))
}

///
///
///
async fn test_cwl(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> Result<Value> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap_or_default();
    let (function, arguments) = prompt_for_input(&functions, &types)?;

    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = Some(package_dir.join("image.tar"));

    let mut mounts = vec![
        String::from("/var/run/docker.sock:/var/run/docker.sock"),
        String::from("/tmp:/tmp"),
    ];

    let input = arguments.get("input").expect("Missing `input` argument");
    if let Value::Struct { properties, .. } = input {
        for (_, value) in properties.iter() {
            match value {
                Value::Array { data_type, entries } => {
                    if data_type == "Directory[]" || data_type == "File[]" {
                        for entry in entries {
                            if let Value::Struct { properties, .. } = entry {
                                let url = properties.get("url").expect(&format!("Missing `url` property on {}", data_type));
                                let path = url_to_path(&Url::parse(&url.as_string()?)?)?;
            
                                mounts.push(format!("{0}:{0}", path.into_os_string().to_string_lossy()));   
                            }
                        }
                    }
                },
                Value::Struct { data_type, properties} => {
                    if data_type == "Directory" || data_type == "File" {
                        let url = properties.get("url").expect(&format!("Missing `url` property on {}", data_type));
                        let path = url_to_path(&Url::parse(&url.as_string()?)?)?;

                        mounts.push(format!("{0}:{0}", path.into_os_string().to_string_lossy()));
                    }
                },
                _ => continue
            }
        }
    }

    let mut output_dir = env::temp_dir();
    output_dir.push(format!("{}", Uuid::new_v4()));
    fs::create_dir(&output_dir)?;

    debug!("Mounts: {:#?}", mounts);
    
    let command = vec![
        String::from("-d"),
        String::from("cwl"),
        String::from("-o"),
        String::from(output_dir.as_os_str().to_string_lossy()),
        function, 
        base64::encode(serde_json::to_string(&arguments)?)
    ];

    debug!("{:?}", command);

    let exec = ExecuteInfo::new(image, image_file, Some(mounts), Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    if stderr.len() > 0 {
        warn!("{}", stderr);
    }

    debug!("stdout: {}", stdout);
    Ok(serde_json::from_str(&stdout)?)    
}

///
///
///
fn test_dsl(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> Result<Value> {
    let instructions_file = package_dir.join("instructions.yml");
    ensure!(instructions_file.exists(), "No instructions found.");

    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap_or_default();
    let (_, arguments) = prompt_for_input(&functions, &types)?;

    // Load instructions
    let buf_reader = BufReader::new(File::open(instructions_file)?);
    let instructions: Vec<Instruction> = serde_yaml::from_reader(buf_reader)?;
    let instructions = machine::preprocess_instructions(&instructions)?;

    debug!("preprocessed: {:#?}", instructions);

    let session_id = Uuid::new_v4();

    let environment = InMemoryEnvironment::new(Some(arguments), None);
    let system = LocalSystem::new(session_id);
    let vault = InMemoryVault::new(Default::default());

    let mut machine = Machine::new(
        Box::new(environment),
        Box::new(system),
        Box::new(vault),
    );

    machine.walk(&instructions)
}

///
///
///
async fn test_ecu(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> Result<Value> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap_or_default();
    let (function, arguments) = prompt_for_input(&functions, &types)?;

    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = Some(package_dir.join("image.tar"));

    let command = vec![
        String::from("ecu"), 
        function, 
        base64::encode(serde_json::to_string(&arguments)?)
    ];

    debug!("{:?}", command);

    let exec = ExecuteInfo::new(image, image_file, None, Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    if stderr.len() > 0 {
        warn!("{}", stderr);
    }

    debug!("stdout: {}", stdout);
    Ok(serde_json::from_str(&stdout)?)
}

///
///
///
async fn test_oas(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> Result<Value> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap_or_default();
    let (function, arguments) = prompt_for_input(&functions, &types)?;

    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = Some(package_dir.join("image.tar"));

    let command = vec![
        String::from("oas"),
        function, 
        base64::encode(serde_json::to_string(&arguments)?)
    ];
    
    debug!("{:?}", command);

    let exec = ExecuteInfo::new(image, image_file, None, Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    if stderr.len() > 0 {
        warn!("{}", stderr);
    }

    debug!("stdout: {}", stdout);
    Ok(serde_json::from_str(&stdout)?)
}

///
///
///
fn prompt_for_input(
    functions: &Map<Function>,
    types: &Map<Type>,
) -> Result<(String, Map<Value>)> {
    let function_list: Vec<String> = functions.keys().map(|k| k.to_string()).collect();
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
    p: &Parameter
) -> Result<Value> {
    let value = if data_type.ends_with("[]") {
        let entry_data_type = data_type[..data_type.len()-2].to_string();
        let mut entries = vec![];

        loop {
            let mut p = p.clone();
            p.data_type = format!("{}[{}]", entry_data_type, entries.len());
            entries.push(prompt_for_value(&entry_data_type, &p)?);
        
            if !Confirm::new().with_prompt(format!("Do you want to more items to the {} array?", style(p.name).bold().cyan())).interact()? {
                break
            }
        }

        Value::Array {
            data_type: data_type.to_string(),
            entries
        }
    } else {
        match data_type {
            "boolean" => {
                let default = p.clone().default.map(|d| d.as_bool().unwrap());
                Value::Boolean(prompt(&p, default)?)
            }
            "Directory" | "File" => {
                let default = p.clone().default.map(|d| d.as_string().unwrap());
                let url = Value::Unicode(format!("file://{}", prompt(&p, default)?));

                let mut properties = Map::<Value>::default();
                properties.insert(String::from("url"), url);

                Value::Struct {
                    data_type: String::from(data_type),
                    properties
                }
            },
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
            _ => unreachable!()
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
fn print_output(
    value: &Value
) -> () {
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