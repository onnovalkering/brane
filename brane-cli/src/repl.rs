use crate::{packages, registry};
use anyhow::Result;
use brane_dsl::compiler::{Compiler, CompilerOptions};
use brane_vm::machine::Machine;
use brane_vm::environment::InMemoryEnvironment;
use brane_vm::vault::InMemoryVault;
use brane_sys::local::LocalSystem;
use futures::executor::block_on;
use linefeed::{Interface, ReadResult};
use specifications::common::Value;
use specifications::instructions::Instruction;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use std::path::PathBuf;
use serde_yaml;
use uuid::Uuid;

type Map<T> = std::collections::HashMap<String, T>;

pub async fn start(secrets_file: Option<PathBuf>) -> Result<()> {
    println!("Starting interactive session, press Ctrl+D to exit.\n");

    let interface = Interface::new("brane-repl")?;
    interface.set_prompt("brane> ")?;

    // Prepare DSL compiler
    let package_index = registry::get_package_index().await?;
    let mut compiler = Compiler::new(CompilerOptions::repl(), package_index)?;

    // Prepare machine
    let secrets: Map<Value> = if let Some(secrets_file) = secrets_file {
        let reader = BufReader::new(File::open(&secrets_file)?);
        serde_yaml::from_reader(reader)?
    } else {
        Default::default()
    };

    let session_id = Uuid::new_v4();

    let environment = InMemoryEnvironment::new(None, None);
    let system = LocalSystem::new(session_id);
    let vault = InMemoryVault::new(secrets);
    let mut machine = Machine::new(Box::new(environment), Rc::new(system), Rc::new(vault), Some(packages::get_packages_dir()));

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        };

        let instructions = compiler.compile(&line);
        debug!("Instructions: {:?}", instructions);

        match instructions {
            Ok(instructions) => {
                let instructions = preprocess_instructions(&instructions).await?;
                let output = machine.interpret(instructions)?;

                if let Some(value) = output {
                    let value = if let Value::Pointer { ref variable, .. } = value {
                        machine.environment.get(variable)
                    } else {
                        value
                    };

                    print(&value, false);
                }
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }

    println!("Goodbye.");
    Ok(())
}

///
///
///
async fn preprocess_instructions(instructions: &Vec<Instruction>) -> Result<Vec<Instruction>> {
    let mut instructions = instructions.clone();

    for instruction in &mut instructions {
        match instruction {
            Instruction::Act(act) => {
                let name = act.meta.get("name").expect("No `name` property in metadata.");
                let version = act.meta.get("version").expect("No `version` property in metadata.");
                let kind = act.meta.get("kind").expect("No `kind` property in metadata.");

                match kind.as_str() {
                    "cwl" => {
                        let cwl_file = registry::get_package_source(&name, &version, &kind).await?;
                        if cwl_file.exists() {
                            act.meta
                                .insert(String::from("cwl_file"), String::from(cwl_file.to_string_lossy()));
                        }
                    }
                    "dsl" => {
                        let instr_file = registry::get_package_source(&name, &version, &kind).await?;
                        if instr_file.exists() {
                            act.meta
                                .insert(String::from("instr_file"), String::from(instr_file.to_string_lossy()));
                        }
                    }
                    "ecu" => {
                        let image_file = registry::get_package_source(&name, &version, &kind).await?;
                        if image_file.exists() {
                            act.meta
                                .insert(String::from("image_file"), String::from(image_file.to_string_lossy()));
                        }
                    }
                    "oas" => {
                        let oas_file = registry::get_package_source(&name, &version, &kind).await?;
                        if oas_file.exists() {
                            act.meta
                                .insert(String::from("oas_file"), String::from(oas_file.to_string_lossy()));
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

                sub.instructions = block_on(preprocess_instructions(&sub.instructions))?;
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
        Value::Boolean(b) => println!("{}", b),
        Value::Integer(i) => println!("{}", i),
        Value::Real(r) => println!("{}", r),
        Value::Unicode(s) => println!("{}", s),
        _ => println!("{:#?}", value),
    }
}
