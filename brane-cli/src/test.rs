use crate::packages;
use brane_vm::{environment::InMemoryEnvironment, machine::Machine};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input as Prompt;
use serde_yaml;
use specifications::common::Value;
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use std::path::PathBuf;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn handle(
    name: String,
    version: Option<String>,
) -> FResult<()> {
    let version_or_latest = version.unwrap_or_else(|| String::from("latest"));
    let package_dir = packages::get_package_dir(&name, Some(&version_or_latest))?;
    ensure!(package_dir.exists(), "No package found.");

    let package_info = PackageInfo::from_path(package_dir.join("package.yml"))?;
    match package_info.kind.as_str() {
        "api" => test_api(package_dir, package_info),
        "cwl" => test_cwl(package_dir, package_info),
        "dsl" => test_dsl(package_dir, package_info),
        "ecu" => test_ecu(package_dir, package_info),
        _ => unreachable!(),
    }
}

///
///
///
fn test_api(
    _package_dir: PathBuf,
    _package_info: PackageInfo,
) -> FResult<()> {
    unimplemented!()
}

///
///
///
fn test_cwl(
    _package_dir: PathBuf,
    _package_info: PackageInfo,
) -> FResult<()> {
    unimplemented!()
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
