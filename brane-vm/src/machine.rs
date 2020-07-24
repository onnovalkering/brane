use crate::environment::Environment;
use crate::vault::Vault;
use anyhow::Result;
use brane_exec::delegate;
use brane_sys::System;
use flate2::read::GzDecoder;
use futures::executor::block_on;
use semver::Version;
use specifications::common::Value;
use specifications::instructions::ActInstruction;
use specifications::instructions::{Instruction, Instruction::*, Move, Move::*, Operator::*};
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use tar::Archive;
use std::rc::Rc;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub struct Machine {
    cursor: usize,
    pub environment: Box<dyn Environment>,
    packages_dir: Option<PathBuf>,
    pub system: Rc<dyn System>,
    pub vault: Rc<dyn Vault>,
}

impl Machine {
    ///
    ///
    ///
    pub fn new(
        environment: Box<dyn Environment>,
        system: Rc<dyn System>,
        vault: Rc<dyn Vault>,
        packages_dir: Option<PathBuf>,
    ) -> Self {
        Machine {
            cursor: 0,
            environment,
            packages_dir,
            system,
            vault,
        }
    }

    ///
    ///
    ///
    pub fn interpret(
        &mut self,
        instructions: Vec<Instruction>,
    ) -> Result<Option<Value>> {
        let cursor_max = instructions.len();

        self.cursor = 0;
        self.environment.remove("terminate");

        while self.cursor != cursor_max {
            let instruction = instructions.get(self.cursor).unwrap().clone();
            let movement = match instruction {
                Act(_) => self.handle_act(instruction)?,
                Mov(_) => self.handle_mov(instruction)?,
                Sub(_) => self.handle_sub(instruction)?,
                Var(_) => self.handle_var(instruction)?,
            };

            match movement {
                Backward => self.cursor -= 1,
                Forward => self.cursor += 1,
                Skip => self.cursor += 2,
            }
        }

        // Return terminate (return) value
        if self.environment.exists("terminate") {
            let value = self.environment.get("terminate");

            return Ok(Some(value));
        }

        Ok(None)
    }

    ///
    ///
    ///
    fn handle_act(
        &mut self,
        instruction: Instruction,
    ) -> Result<Move> {
        let act = if let Act(act) = instruction {
            act
        } else {
            bail!("Not a ACT instruction.");
        };

        debug!("Handling ACT instruction:\n{:#?}", act);

        // Prepare arguments
        let mut arguments = Map::<Value>::new();
        for (name, value) in &act.input {
            match &value {
                Value::Pointer { variable, secret, .. } => {
                    if *secret {
                        let value = self.vault.get(variable);
                        arguments.insert(name.clone(), value.clone());
                    } else if variable.contains(".") {
                        let segments: Vec<_> = variable.split(".").collect();
                        let arch_value = self.environment.get(segments[0]);

                        match arch_value {
                            Value::Array { entries, .. } => {
                                if segments[1] == "length" {
                                    arguments.insert(name.clone(), Value::Integer(entries.len() as i64));
                                } else {
                                    panic!("Trying to access undeclared variable.");
                                }
                            }
                            Value::Struct { properties, .. } => {
                                if let Some(value) = properties.get(segments[1]) {
                                    arguments.insert(name.clone(), value.clone());
                                } else {
                                    panic!("Trying to access undeclared variable.");
                                }
                            }
                            _ => unreachable!(),
                        };
                    } else {
                        let value = self.environment.get(variable);
                        arguments.insert(name.clone(), value);
                    }
                }
                _ => {
                    arguments.insert(name.clone(), value.clone());
                }
            }
        }

        let kind = act.meta.get("kind").expect("Missing `kind` metadata property.");
        let output = match kind.as_str() {
            "cwl" => block_on(delegate::exec_cwl(&act, arguments))?,
            "dsl" => self.exec_dsl(&act, arguments)?,
            "ecu" => block_on(delegate::exec_ecu(&act, arguments))?,
            "oas" => block_on(delegate::exec_oas(&act, arguments))?,
            "std" => delegate::exec_std(&act, arguments, self.system.clone())?,
            _ => unreachable!(),
        };

        if let Some(ref assignment) = act.assignment {
            let output = output.expect("Missing output.");

            // Assert that types match
            let actual_type = output.data_type();
            let expected_type = act.data_type.expect("Missing `data_type` property.");

            if actual_type != expected_type {
                return Err(anyhow!(
                    "Type assertion failed. Expected '{}', but was '{}'.",
                    expected_type,
                    actual_type
                ));
            }

            self.environment.set(assignment, &output);
        }

        Ok(Forward)
    }

    ///
    ///
    ///
    pub fn exec_dsl(
        &mut self,
        act: &ActInstruction,
        arguments: Map<Value>,
    ) -> Result<Option<Value>> {
        let instructions: Vec<Instruction> = if let Some(instr_file) = act.meta.get("instr_file").map(PathBuf::from) {
            let instr_reader = BufReader::new(File::open(&instr_file)?);
            serde_yaml::from_reader(instr_reader)?
        } else {
            unimplemented!()
        };

        let mut sub_environment = self.environment.child();
        for (name, value) in arguments {
            sub_environment.set(&name, &value);
        }
        let mut sub_machine = Machine::new(
            sub_environment,
            self.system.clone(),
            self.vault.clone(),
            self.packages_dir.clone()
        );

        let instructions = if let Some(ref packages_dir) = self.packages_dir {
            preprocess_instructions(&instructions, packages_dir)?
        } else {
            instructions
        };

        sub_machine.interpret(instructions)
    }

    ///
    ///
    ///
    fn handle_mov(
        &mut self,
        instruction: Instruction,
    ) -> Result<Move> {
        let mov = if let Mov(mov) = instruction {
            mov
        } else {
            bail!("Not a MOV instruction.");
        };

        let mut movement = if mov.conditions.len() == mov.branches.len() {
            // Default, may be overriden based on the specific branch
            Forward
        } else {
            // Else branch is at n+1 position, where n = conditions.len()
            mov.branches.last().unwrap().clone()
        };

        for (i, condition) in mov.conditions.iter().enumerate() {
            // Get values from environment, in the case of variables
            let lhs = if let Value::Pointer { variable, .. } = &condition.left {
                self.environment.get(variable)
            } else {
                condition.left.clone()
            };
            let rhs = if let Value::Pointer { variable, .. } = &condition.right {
                self.environment.get(variable)
            } else {
                condition.right.clone()
            };

            let truthy = match condition.operator {
                Equals => lhs == rhs,
                NotEquals => lhs != rhs,
                Greater => lhs > rhs,
                Less => lhs < rhs,
                GreaterOrEqual => lhs >= rhs,
                LessOrEqual => lhs <= rhs,
            };

            if truthy {
                movement = mov.branches.get(i).unwrap().clone();
                break;
            }
        }

        Ok(movement)
    }

    ///
    ///
    ///
    fn handle_sub(
        &mut self,
        instruction: Instruction,
    ) -> Result<Move> {
        let sub = if let Sub(sub) = instruction {
            sub
        } else {
            bail!("Not a SUB instruction.");
        };

        let mut sub_machine = Machine::new(
            self.environment.child(),
            self.system.clone(),
            self.vault.clone(),
            self.packages_dir.clone()
        );
        sub_machine.interpret(sub.instructions)?;

        // Commit variables to parent (this) machine
        let sub_variables = sub_machine.variables();
        for (name, value) in sub_variables.iter() {
            self.environment.set(&name, &value);
        }

        Ok(Forward)
    }

    fn variables(&self) -> Map<Value> {
        self.environment.variables()
    }

    ///
    ///
    ///
    fn handle_var(
        &mut self,
        instruction: Instruction,
    ) -> Result<Move> {
        let var = if let Var(var) = instruction {
            var
        } else {
            bail!("Not a VAR instruction.");
        };

        for variable in var.get {
            let variable_exists = self.environment.exists(&variable.name);
            ensure!(variable_exists, "Variable '{}' does not exists.", variable.name);
        }

        for variable in var.set {
            self.environment.set(&variable.name, &variable.value.unwrap());
        }

        Ok(Forward)
    }
}

///
///
///
fn preprocess_instructions(
    instructions: &Vec<Instruction>,
    packages_dir: &PathBuf,
) -> Result<Vec<Instruction>> {
    let mut instructions = instructions.clone();

    for instruction in &mut instructions {
        match instruction {
            Instruction::Act(act) => {
                let name = act.meta.get("name").expect("No `name` property in metadata.");
                let version = act.meta.get("version").expect("No `version` property in metadata.");
                let kind = act.meta.get("kind").expect("No `kind` property in metadata.");

                match kind.as_str() {
                    "cwl" => {
                        let cwl_file = get_package_source(&name, &version, &kind)?;
                        if cwl_file.exists() {
                            act.meta
                                .insert(String::from("cwl_file"), String::from(cwl_file.to_string_lossy()));
                        }
                    }
                    "dsl" => {
                        let instr_file = get_package_source(&name, &version, &kind)?;
                        if instr_file.exists() {
                            act.meta
                                .insert(String::from("instr_file"), String::from(instr_file.to_string_lossy()));
                        }
                    }
                    "ecu" => {
                        let image_file = get_package_source(&name, &version, &kind)?;
                        if image_file.exists() {
                            act.meta
                                .insert(String::from("image_file"), String::from(image_file.to_string_lossy()));
                        }
                    }
                    "oas" => {
                        let oas_file = get_package_source(&name, &version, &kind)?;
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

                    let package_dir = packages_dir.join(name).join(version);
                    let instructions_file = package_dir.join("instructions.yml");
                    let instructions_reader = BufReader::new(File::open(&instructions_file)?);

                    sub.instructions = serde_yaml::from_reader(instructions_reader)?;
                }

                sub.instructions = preprocess_instructions(&sub.instructions, &packages_dir)?;
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
pub fn get_packages_dir() -> PathBuf {
    appdirs::user_data_dir(Some("brane"), None, false)
        .expect("Couldn't determine Brane data directory.")
        .join("packages")
}

///
///
///
pub fn get_package_dir(
    name: &str,
    version: Option<&str>,
) -> Result<PathBuf> {
    let packages_dir = get_packages_dir();
    let package_dir = packages_dir.join(&name);

    if version.is_none() {
        return Ok(package_dir);
    }

    let version = version.unwrap();
    let version = if version == "latest" {
        if !package_dir.exists() {
            return Err(anyhow!("Package not found."));
        }

        let versions = fs::read_dir(&package_dir)?;
        let mut versions: Vec<Version> = versions
            .map(|v| v.unwrap().file_name())
            .map(|v| Version::parse(&v.into_string().unwrap()).unwrap())
            .collect();

        versions.sort();
        versions.reverse();

        versions[0].to_string()
    } else {
        Version::parse(&version)
            .expect("Not a valid semantic version.")
            .to_string()
    };

    Ok(package_dir.join(version))
}

lazy_static! {
    static ref API_HOST: String = env::var("API_HOST").unwrap_or_else(|_| String::from("brane-api:8080"));
}

///
///
///
pub fn get_package_source(
    name: &String,
    version: &String,
    kind: &String,
) -> Result<PathBuf> {
    let package_dir = get_package_dir(name, Some(version))?;
    let temp_dir = PathBuf::from("/tmp"); // TODO: get from OS

    let path = match kind.as_str() {
        "cwl" => {
            let cwl_file = package_dir.join("document.cwl");
            if cwl_file.exists() {
                cwl_file
            } else {
                let cwl_file = temp_dir.join(format!("{}-{}-document.cwl", name, version));
                if !cwl_file.exists() {
                    let url = format!("http://{}/packages/{}/{}/source", API_HOST.as_str(), name, version);
                    let source = reqwest::blocking::get(&url)?;

                    // Write package archive to temporary file
                    let mut source_file = File::create(&cwl_file)?;
                    source_file.write_all(&source.bytes()?)?;
                }

                cwl_file
            }
        }
        "dsl" => {
            let instructions = package_dir.join("instructions.yml");
            if instructions.exists() {
                instructions
            } else {
                let instructions = temp_dir.join(format!("{}-{}-instructions.yml", name, version));
                if !instructions.exists() {
                    let url = format!("http://{}/packages/{}/{}/source", API_HOST.as_str(), name, version);
                    let source = reqwest::blocking::get(&url)?;

                    // Write package archive to temporary file
                    let mut source_file = File::create(&instructions)?;
                    source_file.write_all(&source.bytes()?)?;
                }

                instructions
            }
        }
        "ecu" => {
            let image_file = package_dir.join("image.tar");
            if false && image_file.exists() {
                image_file
            } else {
                let archive_dir = temp_dir.join(format!("{}-{}-archive", name, version));
                fs::create_dir_all(&archive_dir)?;

                let image_file = archive_dir.join("image.tar");
                if !image_file.exists() {
                    let url = format!("http://{}/packages/{}/{}/archive", API_HOST.as_str(), name, version);
                    let archive = reqwest::blocking::get(&url)?;

                    // Write package archive to temporary file
                    let archive_path = temp_dir.join(format!("{}-{}-archive.tar.gz", name, version));
                    let mut archive_file = File::create(&archive_path)?;
                    archive_file.write_all(&archive.bytes()?)?;

                    // Unpack
                    let archive_file = File::open(archive_path)?;
                    let mut archive = Archive::new(GzDecoder::new(archive_file));
                    archive.unpack(&archive_dir)?;
                }

                image_file
            }
        }
        "oas" => {
            let oas_file = package_dir.join("document.yml");
            if oas_file.exists() {
                oas_file
            } else {
                let oas_file = temp_dir.join(format!("{}-{}-document.yml", name, version));
                if !oas_file.exists() {
                    let url = format!("http://{}/packages/{}/{}/source", API_HOST.as_str(), name, version);
                    let source = reqwest::blocking::get(&url)?;

                    // Write package archive to temporary file
                    let mut source_file = File::create(&oas_file)?;
                    source_file.write_all(&source.bytes()?)?;
                }

                oas_file
            }
        }
        _ => unreachable!(),
    };

    Ok(path)
}
