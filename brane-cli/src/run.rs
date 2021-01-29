use crate::{packages, registry};
use anyhow::Result;
use brane_dsl::compiler::Compiler;
use brane_sys::local::LocalSystem;
use brane_vm::environment::InMemoryEnvironment;
use brane_vm::machine::Machine;
use brane_vm::vault::InMemoryVault;
use console::style;
use futures::executor::block_on;
use serde_yaml;
use specifications::common::Value;
use specifications::instructions::Instruction;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use uuid::Uuid;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn handle(
    file: PathBuf,
    secrets_file: Option<PathBuf>,
) -> Result<()> {
    let dsl_file = fs::canonicalize(file)?;
    let dsl_document = fs::read_to_string(&dsl_file)?;

    debug!("Using {:?} as input DSL file", dsl_file);

    // Remove shebang, if present
    let dsl_document = if dsl_document.starts_with("#!") {
        dsl_document.split('\n').skip(1).collect::<Vec<&str>>().join("\n")
    } else {
        dsl_document
    };

    // Compile to instructions
    let package_index = registry::get_package_index().await?;
    let instructions = Compiler::quick_compile(package_index, &dsl_document)?;

    // Prepare secrets, if any
    let secrets: Map<Value> = if let Some(secrets_file) = secrets_file {
        let secrets_file = fs::canonicalize(secrets_file)?;
        let reader = BufReader::new(File::open(&secrets_file)?);
        serde_yaml::from_reader(reader)?
    } else {
        Default::default()
    };

    start(instructions, secrets)?;

    Ok(())
}

///
///
///
fn start(
    instructions: Vec<Instruction>,
    secrets: Map<Value>,
) -> Result<()> {
    // Prepare machine
    let session_id = Uuid::new_v4();

    let environment = InMemoryEnvironment::new(None, None);
    let system = LocalSystem::new(session_id);
    let vault = InMemoryVault::new(secrets);

    let mut machine = Machine::new(Box::new(environment), Box::new(system), Box::new(vault));

    let instructions = preprocess_instructions(&instructions)?;
    let output = machine.walk(&instructions)?;

    match output {
        Value::Unit => {}
        _ => print(&output, false),
    }

    Ok(())
}

///
///
///
fn preprocess_instructions(instructions: &Vec<Instruction>) -> Result<Vec<Instruction>> {
    let mut instructions = instructions.clone();

    for instruction in &mut instructions {
        match instruction {
            Instruction::Act(act) => {
                let name = act.meta.get("name").expect("No `name` property in metadata.");
                let version = act.meta.get("version").expect("No `version` property in metadata.");
                let kind = act.meta.get("kind").expect("No `kind` property in metadata.");

                match kind.as_str() {
                    "dsl" => {
                        let instr_file = block_on(registry::get_package_source(&name, &version, &kind))?;
                        if instr_file.exists() {
                            act.meta
                                .insert(String::from("instr_file"), String::from(instr_file.to_string_lossy()));
                        }
                    }
                    "cwl" | "ecu" | "oas" => {
                        let image_file = block_on(registry::get_package_source(&name, &version, &kind))?;
                        if image_file.exists() {
                            act.meta
                                .insert(String::from("image_file"), String::from(image_file.to_string_lossy()));
                        }
                    }
                    _ => {}
                }
            }
            Instruction::Sub(sub) => {
                if let Some(_) = sub.meta.get("kind") {
                    let name = sub.meta.get("name").expect("No `name` property in metadata.");
                    let version = sub.meta.get("version").expect("No `version` property in metadata.");

                    let package_dir = packages::get_package_dir(&name, Some(version))?;
                    let instructions_file = package_dir.join("instructions.yml");
                    let instructions_reader = BufReader::new(File::open(&instructions_file)?);

                    sub.instructions = serde_yaml::from_reader(instructions_reader)?;
                }

                debug!("Preprocess nested sub instrucations.");
                sub.instructions = preprocess_instructions(&sub.instructions)?;
            }
            _ => continue,
        }
    }

    debug!("{:#?}", instructions);

    Ok(instructions)
}

///
///
///
fn print(
    value: &Value,
    indent: bool,
) {
    if indent {
        print!("   ");
    }

    match value {
        Value::Array { entries, .. } => {
            println!("[");
            for entry in entries.iter() {
                print(entry, true);
            }
            println!("]")
        }
        Value::Boolean(b) => println!("{}", style(b).cyan()),
        Value::Integer(i) => println!("{}", style(i).cyan()),
        Value::Real(r) => println!("{}", style(r).cyan()),
        Value::Unicode(s) => println!("{}", style(s).cyan()),
        _ => println!("{:#?}", value),
    }
}
