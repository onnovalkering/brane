use crate::{packages, registry};
use anyhow::Result;
use brane_dsl::compiler::Compiler;
use console::style;
use specifications::common::{CallPattern, Function, Parameter};
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use yaml_rust::yaml::YamlLoader;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn handle(
    context: PathBuf,
    file: PathBuf,
) -> Result<()> {
    let dsl_file = context.join(file);
    let dsl_document = fs::read_to_string(&dsl_file)?;

    // Compile to instructions
    let package_index = registry::get_package_index().await?;
    let instructions = Compiler::quick_compile(package_index, &dsl_document)?;

    // Prepare package directory
    let package_info = generate_package_info(&dsl_document, &instructions)?;
    let package_dir = packages::get_package_dir(&package_info.name, Some(&package_info.version))?;
    prepare_directory(&instructions, &dsl_file, &package_info, &package_dir)?;

    println!(
        "Successfully built version {} of DSL package {}.",
        style(&package_info.version).bold().cyan(),
        style(&package_info.name).bold().cyan(),
    );

    Ok(())
}

///
///
///
fn generate_package_info(
    dsl_document: &str,
    instructions: &[Instruction],
) -> Result<PackageInfo> {
    let yamls = YamlLoader::load_from_str(&dsl_document).unwrap();
    let info = yamls.first().expect("Document doesn't start with a info section");

    let name = info["name"]
        .as_str()
        .map(String::from)
        .expect("Info section doesn't contain a 'name' property.");

    let version = info["version"]
        .as_str()
        .map(String::from)
        .expect("Info section doesn't contain a 'version' property.");

    let description = info["description"].as_str().map(String::from);

    // Construct function descriptions
    let mut input_parameters = Vec::<Parameter>::new();
    let call_pattern = CallPattern::new(Some(name.to_lowercase()), None, None);

    for instruction in instructions {
        match instruction {
            Instruction::Var(var) => {
                for get in &var.get {
                    let parameter = Parameter::new(get.name.clone(), get.data_type.clone(), None, None);
                    input_parameters.push(parameter);
                }
            }
            _ => continue,
        }
    }

    let return_type = if let Some(data_type) = determine_return_type(instructions) {
        data_type
    } else {
        String::from("unit")
    };

    let mut functions = Map::<Function>::new();
    functions.insert(
        name.clone(),
        Function::new(input_parameters, Some(call_pattern), return_type.clone()),
    );

    let package_info = PackageInfo::new(name, version, description, String::from("dsl"), Some(functions), None);

    Ok(package_info)
}

///
///
///
fn determine_return_type(instructions: &[Instruction]) -> Option<String> {
    for instruction in instructions {
        match instruction {
            Instruction::Var(var) => {
                for set in &var.set {
                    if let Some(scope) = &set.scope {
                        if scope == "output" {
                            return Some(set.data_type.clone());
                        }
                    }
                }
            }
            Instruction::Act(ref act) => {
                if let Some(assignment) = &act.assignment {
                    if assignment == "terminate" {
                        if let Some(ref data_type) = &act.data_type {
                            return Some(data_type.clone());
                        }
                    }
                }
            }
            Instruction::Sub(sub) => {
                let data_type = determine_return_type(&sub.instructions);
                if data_type.is_some() {
                    return data_type;
                }
            }
            _ => continue,
        }
    }

    None
}

///
///
///
fn prepare_directory(
    instructions: &[Instruction],
    dsl_file: &PathBuf,
    package_info: &PackageInfo,
    package_dir: &PathBuf,
) -> Result<()> {
    fs::create_dir_all(&package_dir)?;

    // Copy DSL document to package directory
    fs::copy(dsl_file, package_dir.join("document.bs"))?;

    // Write instructions.yml to package directory
    let mut buffer = File::create(package_dir.join("instructions.yml"))?;
    write!(buffer, "{}", serde_yaml::to_string(&instructions)?)?;

    // Write package.yml to package directory
    let mut buffer = File::create(package_dir.join("package.yml"))?;
    write!(buffer, "{}", serde_yaml::to_string(&package_info)?)?;

    Ok(())
}
