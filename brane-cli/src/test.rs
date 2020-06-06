use crate::packages;
use brane_exec::openapi;
use brane_vm::{environment::InMemoryEnvironment, machine::Machine};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input as Prompt, Select};
use fs_extra::{copy_items, dir::CopyOptions};
use serde_json::Value as JValue;
use openapiv3::OpenAPI;
use console::style;
use serde_yaml;
use specifications::common::{Function, Type, Value};
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn handle(
    name: String,
    version: Option<String>,
) -> FResult<()> {
    let version_or_latest = version.unwrap_or_else(|| String::from("latest"));
    let package_dir = packages::get_package_dir(&name, Some(&version_or_latest))?;
    ensure!(package_dir.exists(), "No package found.");

    let package_info = PackageInfo::from_path(package_dir.join("package.yml"))?;
    match package_info.kind.as_str() {
        "oas" => test_oas(package_dir, package_info).await,
        "cwl" => test_cwl(package_dir, package_info),
        "dsl" => test_dsl(package_dir, package_info),
        "ecu" => test_ecu(package_dir, package_info),
        _ => unreachable!(),
    }
}

///
///
///
async fn test_oas(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> FResult<()> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap();
    let (function_name, arguments) = prompt_for_input(&functions, &types)?;

    let oas_reader = BufReader::new(File::open(&package_dir.join("document.yml"))?);
    let oas_document: OpenAPI = serde_yaml::from_reader(oas_reader)?;

    let json = openapi::execute(function_name.clone(), arguments, &oas_document).await?;

    let function = functions.get(&function_name).unwrap();
    let output_type = types.get(&function.return_type).unwrap();

    for property in &output_type.properties {
        println!("{}:\n{}\n", style(&property.name).bold().cyan(), json[&property.name].as_str().unwrap());
    }

    Ok(())
}

///
///
///
fn test_cwl(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> FResult<()> {
    let functions = package_info.functions.unwrap();
    let types = package_info.types.unwrap();
    let (_, arguments) = prompt_for_input(&functions, &types)?;

    // Copy package directory to working dir
    let working_dir = tempfile::tempdir()?.into_path();
    let package_content = fs::read_dir(package_dir)?
        .map(|e| e.unwrap().path())
        .collect::<Vec<PathBuf>>();

    copy_items(&package_content, &working_dir, &CopyOptions::new())?;

    let input_path = working_dir.join("input.json");
    let mut input_file = File::create(input_path)?;
    writeln!(input_file, "{}", &serde_json::to_string(&arguments)?)?;

    let working_dir_path = working_dir.to_string_lossy().to_string();
    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .args(vec!["-v", "/var/run/docker.sock:/var/run/docker.sock"])
        .args(vec!["-v", "/tmp:/tmp"])
        .args(vec!["-v", &format!("{}:{}", working_dir_path, working_dir_path)])
        .args(vec!["-w", &working_dir_path])
        .arg("commonworkflowlanguage/cwltool")
        .arg("--quiet")
        .arg("document.cwl")
        .arg("input.json")
        .output()
        .expect("Couldn't run 'docker' command.");

    ensure!(output.status.success(), "Failed to run CWL workflow.");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let output_obj: Map<JValue> = serde_json::from_str(&stdout)?;

    for (name, value_info) in output_obj.iter() {
        let mut value = String::new();
        let mut value_file = File::open(value_info["path"].as_str().unwrap())?;
        value_file.read_to_string(&mut value)?;

        println!("{}: {}", name, value);
    }

    Ok(())
}

///
///
///
fn test_dsl(
    package_dir: PathBuf,
    _package_info: PackageInfo,
) -> FResult<()> {
    let instructions_file = package_dir.join("instructions.yml");
    ensure!(instructions_file.exists(), "No instructions found.");

    // Load instructions
    let buf_reader = BufReader::new(File::open(instructions_file)?);
    let mut instructions: Vec<Instruction> = serde_yaml::from_reader(buf_reader)?;

    let mut arguments = Map::<Value>::new();
    test_dsl_preprocess_instructions(&mut instructions, &mut arguments)?;

    let environment = InMemoryEnvironment::new(Some(arguments), None);
    let mut machine = Machine::new(Box::new(environment));
    let output = machine.interpret(instructions)?;

    if let Some(value) = output {
        if let Value::Pointer { variable, .. } = value {
            let value = machine.environment.get(&variable);
            println!("\n{:?}", value);
        } else {
            println!("\n{:?}", value);
        }
    }

    Ok(())
}

///
///
///
fn test_dsl_preprocess_instructions(
    instructions: &mut Vec<Instruction>,
    arguments: &mut Map<Value>,
) -> FResult<()> {
    for instruction in instructions {
        match instruction {
            Instruction::Act(act) => {
                let name = act.meta.get("name").expect("No `name` property in metadata.");
                let version = act.meta.get("version").expect("No `version` property in metadata.");

                let package_dir = packages::get_package_dir(&name, Some(version))?;
                let image_file = package_dir.join("image.tar");
                if image_file.exists() {
                    act.meta
                        .insert(String::from("image_file"), String::from(image_file.to_string_lossy()));
                }
            }
            Instruction::Sub(sub) => {
                test_dsl_preprocess_instructions(&mut sub.instructions, arguments)?;
            }
            Instruction::Var(var) => {
                for get in &var.get {
                    let value = match get.data_type.as_str() {
                        "boolean" => Value::Boolean(prompt(&get.name, &get.data_type)?),
                        "integer" => Value::Integer(prompt(&get.name, &get.data_type)?),
                        "real" => Value::Real(prompt(&get.name, &get.data_type)?),
                        "unicode" => Value::Unicode(prompt(&get.name, &get.data_type)?),
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
fn test_ecu(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> FResult<()> {
    let image_tag = format!("{}:{}", package_info.name, package_info.version);
    let image_file = package_dir.join("image.tar");
    ensure!(image_file.exists(), "No image found.");

    // Load image
    let output = Command::new("docker")
        .arg("load")
        .arg("-i")
        .arg(image_file)
        .output()
        .expect("Couldn't run 'docker' command.");

    ensure!(output.status.success(), "Failed to load image.");

    // Run image
    Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-it")
        .arg(&image_tag)
        .arg("test")
        .status()
        .expect("Couldn't run 'docker' command.");

    // Unload image
    let output = Command::new("docker")
        .arg("image")
        .arg("rm")
        .arg(&image_tag)
        .output()
        .expect("Couldn't run 'docker' command.");

    if !output.status.success() {
        warn!("Failed to unload '{}', image remains loaded in Docker.", image_tag);
    }

    Ok(())
}

///
///
///
fn prompt_for_input(
    functions: &Map<Function>,
    types: &Map<Type>,
) -> FResult<(String, Map<String>)> {
    let function_list: Vec<String> = functions.keys().map(|k| k.to_string()).collect();
    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("The function the execute")
        .default(0)
        .items(&function_list[..])
        .interact()?;

    let function_name = &function_list[index];
    let function = &functions[function_name];
    let input_type = function.parameters[0].data_type.clone();
    let input_obj = types.get(&input_type).unwrap();

    println!("\nPlease provide input for the chosen function:\n");

    let mut arguments = Map::<String>::new();
    for p in &input_obj.properties {
        let data_type = p.data_type.as_str();
        let value = match data_type {
            "boolean" => {
                let value: bool = prompt(&p.name, data_type)?;
                value.to_string()
            }
            "integer" => {
                let value: i64 = prompt(&p.name, data_type)?;
                value.to_string()
            }
            "real" => {
                let value: f64 = prompt(&p.name, data_type)?;
                value.to_string()
            }
            "string" => {
                let value: String = prompt(&p.name, data_type)?;
                value
            }
            _ => continue,
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
