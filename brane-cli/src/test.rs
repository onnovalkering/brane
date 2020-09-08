use crate::packages;
use anyhow::Result;
use brane_exec::{docker, openapi, ExecuteInfo};
use brane_vm::machine::Machine;
use brane_vm::environment::InMemoryEnvironment;
use brane_vm::vault::InMemoryVault;
use brane_sys::local::LocalSystem;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input as Prompt, Select};
use fs_extra::{copy_items, dir::CopyOptions};
use openapiv3::OpenAPI;
use serde_json::{json, Value as JValue};
use serde_yaml;
use specifications::common::{Function, Parameter, Type, Value};
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufReader, Write};
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
    match package_info.kind.as_str() {
        "cwl" => test_cwl(package_dir, package_info).await,
        "dsl" => test_dsl(package_dir, package_info),
        "ecu" => test_ecu(package_dir, package_info).await,
        "oas" => test_oas(package_dir, package_info).await,
        _ => {
            return Err(anyhow!(UNSUPPORTED_PACKAGE_KIND));
        }
    }
}

///
///
///
async fn test_cwl(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> Result<()> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap();
    let (_, arguments) = prompt_for_input(&functions, &types)?;

    // Create (temporary) working directory
    let working_dir = tempfile::tempdir()?.into_path();
    let working_dir_str = working_dir.to_string_lossy().to_string();

    // Copy package directory to working dir
    let package_content = fs::read_dir(package_dir)?
        .map(|e| e.unwrap().path())
        .collect::<Vec<PathBuf>>();

    copy_items(&package_content, &working_dir, &CopyOptions::new())?;

    // Create file containing input arguments
    let mut input = Map::<JValue>::new();
    for (name, value) in arguments.iter() {
        input.insert(name.clone(), value.as_json());
    }

    let input_path = working_dir.join("input.json");
    let mut input_file = File::create(input_path)?;
    writeln!(input_file, "{}", &serde_json::to_string(&input)?)?;

    // Setup execution
    let image = String::from("commonworkflowlanguage/cwltool");
    let mounts = vec![
        String::from("/var/run/docker.sock:/var/run/docker.sock"),
        String::from("/tmp:/tmp"),
        format!("{}:{}", working_dir_str, working_dir_str),
    ];
    let command = vec![
        String::from("--quiet"),
        String::from("document.cwl"),
        String::from("input.json"),
    ];

    let exec = ExecuteInfo::new(image, None, Some(mounts), Some(working_dir_str), Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    if stderr.len() > 0 {
        warn!("{}", stderr);
    }

    let output: Map<JValue> = serde_json::from_str(&stdout)?;
    for (name, value) in output.iter() {
        let mut value_file = File::open(value["path"].as_str().unwrap())?;
        let mut value = String::new();
        value_file.read_to_string(&mut value)?;

        println!("{}:\n{}\n", style(&name).bold().cyan(), value);
    }

    Ok(())
}

///
///
///
fn test_dsl(
    package_dir: PathBuf,
    _package_info: PackageInfo,
) -> Result<()> {
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
        println!();
        println!("{}:\n{}\n", style("output").bold().cyan(), value);
    }

    Ok(())
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
) -> Result<()> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap_or_default();
    let (function_name, arguments) = prompt_for_input(&functions, &types)?;

    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = Some(package_dir.join("image.tar"));
    let payload = json!({
        "identifier": String::from("test"),
        "action": function_name,
        "arguments": arguments,
    });
    let command = vec![String::from("exec"), base64::encode(serde_json::to_string(&payload)?)];
    debug!("{:?}", command);

    let exec = ExecuteInfo::new(image, image_file, None, None, Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    if stderr.len() > 0 {
        warn!("{}", stderr);
    }

    debug!("stdout: {}", stdout);

    let output: Value = serde_json::from_str(&stdout)?;
    println!("{}", style(&output).bold().cyan());

    Ok(())
}

///
///
///
async fn test_oas(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> Result<()> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap();
    let (function_name, arguments) = prompt_for_input(&functions, &types)?;

    let oas_reader = BufReader::new(File::open(&package_dir.join("document.yml"))?);
    let oas_document: OpenAPI = serde_yaml::from_reader(oas_reader)?;

    let json = openapi::execute(&function_name, arguments, &oas_document).await?;

    let function = functions.get(&function_name).unwrap();
    let output_type = types.get(&function.return_type).unwrap();

    for property in &output_type.properties {
        println!(
            "{}:\n{}\n",
            style(&property.name).bold().cyan(),
            json[&property.name].as_str().unwrap()
        );
    }

    Ok(())
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
                Value::Unicode(prompt(&p, default)?)
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
