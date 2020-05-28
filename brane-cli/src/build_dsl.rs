use crate::{packages, registry};
use brane_dsl::compiler::Compiler;
use specifications::instructions::Instruction;
use specifications::package::{Function, PackageInfo};
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use yaml_rust::yaml::YamlLoader;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn handle(
    context: PathBuf,
    file: PathBuf,
) -> FResult<()> {
    let dsl_file = context.join(file);
    let dsl_document = fs::read_to_string(&dsl_file)?;

    // Compile to instructions
    let package_index = registry::get_package_index().await?;
    let instructions = Compiler::quick_compile(package_index, &dsl_document)?;

    // Prepare package directory
    let package_info = generate_package_info(&dsl_document, &instructions)?;
    let package_dir = packages::get_package_dir(&package_info.name, Some(&package_info.version))?;
    prepare_directory(&instructions, &dsl_file, &package_info, &package_dir)?;

    Ok(())
}

///
///
///
fn generate_package_info(
    dsl_document: &str,
    _instructions: &[Instruction],
) -> FResult<PackageInfo> {
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
    let functions = Map::<Function>::new();
    // for (action_name, action) in &container_info.actions {
    //     let arguments = action.input.clone();
    //     let notation = action.notation.clone();
    //     let return_type = action.output[0].data_type.to_string();

    //     let function = Function::new(arguments, notation, return_type);
    //     functions.insert(action_name.clone(), function);
    // }

    // Create and write a package.yml file.
    let package_info = PackageInfo::new(name, version, description, String::from("dsl"), Some(functions), None);

    Ok(package_info)
}

///
///
///
fn prepare_directory(
    instructions: &[Instruction],
    dsl_file: &PathBuf,
    package_info: &PackageInfo,
    package_dir: &PathBuf,
) -> FResult<()> {
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
