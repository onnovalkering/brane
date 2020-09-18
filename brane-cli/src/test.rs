use crate::packages;
use anyhow::Result;
use brane_exec::{docker, ExecuteInfo};
use brane_vm::machine::Machine;
use brane_vm::environment::InMemoryEnvironment;
use brane_vm::vault::InMemoryVault;
use brane_sys::local::LocalSystem;
use console::style;
use dialoguer::Password;
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
use std::rc::Rc;
use uuid::Uuid;

type Map<T> = std::collections::HashMap<String, T>;

const NESTED_OBJ_NOT_SUPPORTED: &str = "Nested parameter objects are not supported";
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

    let mounts = vec![
        String::from("/var/run/docker.sock:/var/run/docker.sock"),
        String::from("/tmp:/tmp"),
    ];

    let command = vec![
        String::from("cwl"), 
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
    _package_info: PackageInfo,
) -> Result<Value> {
    let instructions_file = package_dir.join("instructions.yml");
    ensure!(instructions_file.exists(), "No instructions found.");

    // Load instructions
    let buf_reader = BufReader::new(File::open(instructions_file)?);
    let mut instructions: Vec<Instruction> = serde_yaml::from_reader(buf_reader)?;

    let mut arguments = Map::<Value>::new();
    test_dsl_preprocess_instructions(&mut instructions, &mut arguments)?;

    let session_id = Uuid::new_v4();

    let secrets = Map::<Value>::new();
    let vault = InMemoryVault::new(secrets);

    let environment = InMemoryEnvironment::new(Some(arguments), None);
    let system = LocalSystem::new(session_id);

    let mut machine = Machine::new(Box::new(environment), Rc::new(system), Rc::new(vault), Some(packages::get_packages_dir()));
    let output = machine.interpret(instructions)?;

    let output = output.map(|o| {
        if let Value::Pointer { variable, .. } = o {
            machine.environment.get(&variable)
        } else {
            o
        }
    });

    if let Some(value) = output {
        Ok(value)
    } else {
        Ok(Value::Unit)
    }
}

///
///
///
fn test_dsl_preprocess_instructions(
    instructions: &mut Vec<Instruction>,
    arguments: &mut Map<Value>,
) -> Result<()> {
    for instruction in instructions {
        match instruction {
            Instruction::Act(act) => {
                let name = act.meta.get("name").expect("No `name` property in metadata.");
                let version = act.meta.get("version").expect("No `version` property in metadata.");
                let kind = act.meta.get("kind").expect("No `kind` property in metadata.");

                let package_dir = packages::get_package_dir(&name, Some(version))?;
                match kind.as_str() {
                    "cwl" => {
                        let cwl_file = package_dir.join("document.cwl");
                        if cwl_file.exists() {
                            act.meta
                                .insert(String::from("cwl_file"), String::from(cwl_file.to_string_lossy()));
                        }
                    }
                    "ecu" => {
                        let image_file = package_dir.join("image.tar");
                        if image_file.exists() {
                            act.meta
                                .insert(String::from("image_file"), String::from(image_file.to_string_lossy()));
                        }
                    }
                    "oas" => {
                        let oas_file = package_dir.join("document.yml");
                        if oas_file.exists() {
                            act.meta
                                .insert(String::from("oas_file"), String::from(oas_file.to_string_lossy()));
                        }
                    }
                    _ => {}
                }
            }
            Instruction::Sub(sub) => {
                test_dsl_preprocess_instructions(&mut sub.instructions, arguments)?;
            }
            Instruction::Var(var) => {
                for get in &var.get {
                    let value = match get.data_type.as_str() {
                        "boolean" => Value::Boolean(prompt_var(&get.name, &get.data_type)?),
                        "integer" => Value::Integer(prompt_var(&get.name, &get.data_type)?),
                        "real" => Value::Real(prompt_var(&get.name, &get.data_type)?),
                        "string" => Value::Unicode(prompt_var(&get.name, &get.data_type)?),
                        _ => unimplemented!(),
                    };

                    arguments.insert(get.name.clone(), value);
                }
            }
            _ => {}
        }
    }

    Ok(())
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
        let value = match data_type {
            "boolean" => {
                let default = p.clone().default.map(|d| d.as_bool().unwrap());
                Value::Boolean(prompt(&p, default)?)
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
                let value = if p.name == "password" {
                    prompt_password(&p, default)?
                } else {
                    prompt(&p, default)?
                };

                Value::Unicode(value)
            }
            _ => {
                if let Some(input_type) = types.get(data_type) {
                    let mut properties = Map::<Value>::new();

                    for p in &input_type.properties {
                        let p = p.clone().into_parameter();
                        let data_type = p.data_type.as_str();
                        let value = match data_type {
                            "boolean" => {
                                let default = p.clone().default.map(|d| d.as_bool().unwrap());
                                Value::Boolean(prompt(&p, default)?)
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
                                Value::Unicode(prompt(&p, default)?)
                            }
                            _ => {
                                return Err(anyhow!(NESTED_OBJ_NOT_SUPPORTED));
                            }
                        };

                        properties.insert(p.name.clone(), value);
                    }

                    Value::Struct {
                        data_type: input_type.name.clone(),
                        properties,
                    }
                } else {
                    return Err(anyhow!("Unsupported parameter type: {}", data_type));
                }
            }
        };

        arguments.insert(p.name.clone(), value);
    }

    println!();

    Ok((function_name.clone(), arguments))
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
    default: Option<String>,
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
fn prompt_var<T>(
    name: &str,
    data_type: &str,
) -> std::io::Result<T>
where
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    Prompt::with_theme(&ColorfulTheme::default())
        .with_prompt(&format!("{} ({})", name, data_type))
        .interact()
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
                println!("{}", style(value).cyan());
            }
        }
    }
}