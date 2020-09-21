use crate::environment::{InMemoryEnvironment, Environment};
use crate::vault::{InMemoryVault, Vault};
use anyhow::Result;
use brane_sys::System;
use brane_exec::{schedule, ExecuteInfo, docker};
use specifications::common::Value;
use specifications::instructions::{Instruction, Instruction::*, Move::*, Operator::*};
use specifications::instructions::*;
use crate::cursor::{Cursor, InMemoryCursor};
use std::path::PathBuf;
use futures::executor::block_on;
use std::fs::File;
use std::io::BufReader;

type Map<T> = std::collections::HashMap<String, T>;

pub enum MachineResult {
    Waiting,
    Complete
}

///
///
///
pub struct AsyncMachine {
    pub cursor: Box<dyn Cursor>,
    pub environment: Box<dyn Environment>,
    pub instructions: Vec<Instruction>,
    pub invocation_id: i32,
    pub system: Box<dyn System>,
    pub vault: Box<dyn Vault>,
}

impl AsyncMachine {
    ///
    ///
    ///
    pub fn new(
        instructions: Vec<Instruction>,
        invocation_id: i32,
        cursor: Box<dyn Cursor>,
        environment: Box<dyn Environment>,
        system: Box<dyn System>,
        vault: Box<dyn Vault>,
    ) -> Self {
        AsyncMachine {
            cursor,
            environment,
            instructions,
            invocation_id,
            system,
            vault,
        }
    }

    ///
    ///
    ///
    pub async fn walk(
        &mut self,
    ) -> Result<MachineResult> {
        while let Some(instruction) = get_current_instruction(&self.instructions, &self.cursor) {
            match instruction {
                Act(act) => {
                    let arguments = prepare_arguments(&act.input, &self.environment, &self.vault)?;
                    let kind = act.meta.get("kind").expect("Missing `kind` metadata property.");

                    // Run standard library functions in-place, other ACTs will be scheduled.
                    if kind == "std" {
                        let package = act.meta.get("name").expect("No `name` property in metadata.");
                        let function = brane_std::FUNCTIONS.get(package).unwrap().get(&act.name).unwrap();
                        
                        let output = function(&arguments, &self.system)?;
                        self.callback(output)?;
                    } else {
                        match kind.as_str() {
                            "cwl" => schedule::cwl(&act, arguments, self.invocation_id, &self.system).await?,
                            "dsl" => schedule::dsl(&act, arguments, self.invocation_id).await?,
                            "ecu" => schedule::ecu(&act, arguments, self.invocation_id, &self.system).await?,
                            "oas" => schedule::oas(&act, arguments, self.invocation_id, &self.system).await?,
                            _ => unreachable!()
                        };

                        // This will put the current machine in a waiting state
                        return Ok(MachineResult::Waiting);
                    }
                },
                Mov(mov) => handle_mov(&mov, &mut self.cursor, &self.environment),
                Sub(sub) => handle_sub(&sub, &mut self.cursor),
                Var(var) => handle_var(&var, &mut self.cursor, &mut self.environment),
            }
        }

        Ok(MachineResult::Complete)
    }

    ///
    ///
    ///
    pub fn callback(
        &mut self,
        value: Value,
    ) -> Result<()> {
        let instruction = get_current_instruction(&self.instructions, &self.cursor).unwrap();
        let act = if let Act(act) = instruction {
            act
        } else {
            unreachable!();
        };

        if let Some(ref assignment) = act.assignment {
            // Assert that types match
            let actual_type = value.data_type();
            let expected_type = act.data_type.expect("Missing `data_type` property.");

            if actual_type != expected_type {
                return Err(anyhow!(
                    "Type assertion failed. Expected '{}', but was '{}'.",
                    expected_type,
                    actual_type
                ));
            }

            self.environment.set(assignment, &value);
        }

        self.cursor.go(Forward);
        Ok(())
    }
}

///
///
///
pub struct Machine {
    pub environment: Box<dyn Environment>,
    pub system: Box<dyn System>,
    pub vault: Box<dyn Vault>,
}

impl Machine {
    ///
    ///
    ///
    pub fn new(
        environment: Box<dyn Environment>,
        system: Box<dyn System>,
        vault: Box<dyn Vault>,
    ) -> Self {
        Machine {
            environment,
            system,
            vault,
        }
    }

    ///
    ///
    ///
    pub fn walk(
        &mut self,
        instructions: &Vec<Instruction>,
    ) -> Result<Value> {
        debug!("instructions: {:?}", instructions);
    
        let mut cursor: Box<dyn Cursor> = Box::new(InMemoryCursor::new());

        while let Some(instruction) = get_current_instruction(instructions, &cursor) {
            match instruction {
                Act(act) => {
                    let arguments = prepare_arguments(&act.input, &self.environment, &self.vault)?;
                    let kind = act.meta.get("kind").expect("Missing `kind` metadata property.");

                    // Run standard library functions in-place, other ACTs will be scheduled.
                    let output = if kind == "std" {
                        let package = act.meta.get("name").expect("No `name` property in metadata.");
                        let function = brane_std::FUNCTIONS.get(package).unwrap().get(&act.name).unwrap();
                        
                        function(&arguments, &self.system)?
                    } else {
                        match kind.as_str() {
                            "cwl" => handle_act(&act, arguments, kind, &self.system)?,
                            "dsl" => handle_act_dsl(&act, arguments, self.system.clone())?,
                            "ecu" => handle_act(&act, arguments, kind, &self.system)?,
                            "oas" => handle_act(&act, arguments, kind, &self.system)?,
                            _ => unreachable!()
                        }
                    };
                    
                    if let Some(ref assignment) = act.assignment {
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
            
                    cursor.go(Forward);                    
                },
                Mov(mov) => handle_mov(&mov, &mut cursor, &self.environment),
                Sub(sub) => handle_sub(&sub, &mut cursor),
                Var(var) => handle_var(&var, &mut cursor, &mut self.environment),
            }
        }

        debug!("env: {:#?}", self.environment.variables());

        if self.environment.exists("terminate") {
            let mut value = self.environment.get("terminate");
            if let Value::Pointer { ref variable, .. } = value {
                value = self.environment.get(variable);
            }

            self.environment.remove("terminate");
            Ok(value)
        } else {
            Ok(Value::Unit)
        }
    }
}

///
///
///
fn get_current_instruction(
    instructions: &Vec<Instruction>,
    cursor: &Box<dyn Cursor>,
) -> Option<Instruction> {
    let position = cursor.get_position();
    if position == instructions.len() {
        return None
    }

    let mut instruction = instructions.get(position).unwrap();
    (1..cursor.get_depth()+1).for_each(|d| {
        let subposition = cursor.get_subposition(d);
        if let Instruction::Sub(sub) = instruction {
            instruction = sub.instructions.get(subposition).unwrap();
        }
    });

    Some(instruction.clone())
}

///
///
///
fn prepare_arguments(
    input: &Map<Value>,
    environment: &Box<dyn Environment>,
    vault: &Box<dyn Vault>,
) -> Result<Map<Value>> {
    let mut arguments = Map::<Value>::new();

    for (name, value) in input {
        match &value {
            Value::Pointer { variable, secret, .. } => {
                if *secret {
                    let value = vault.get(variable)?;
                    arguments.insert(name.clone(), value.clone());
                } else {
                    let value = resolve_variable(variable, environment);
                    arguments.insert(name.clone(), value);
                }
            }
            _ => {
                arguments.insert(name.clone(), value.clone());
            }
        }
    }

    Ok(arguments)
}

fn handle_act(
    act: &ActInstruction,
    arguments: Map<Value>,
    kind: &str,
    system: &Box::<dyn System>,
) -> Result<Value> {
    let image = act.meta.get("image").expect("Missing `image` metadata property.").clone();
    let image_file = act.meta.get("image_file").map(PathBuf::from).expect("Missing `image_file` metadata property.");

    let temp_dir = system.get_temp_dir();
    let session_dir = system.get_session_dir();

    let mounts = vec![
        format!("{0}:{0}", temp_dir.into_os_string().into_string().unwrap()),
        format!("{0}:{0}", session_dir.into_os_string().into_string().unwrap()),
        String::from("/var/run/docker.sock:/var/run/docker.sock"),
    ];

    let arguments = base64::encode(serde_json::to_string(&arguments)?);
    let command = vec![
        String::from(kind),
        String::from(&act.name),
        arguments,
    ];

    let exec = ExecuteInfo::new(image, Some(image_file), Some(mounts), Some(command));
    let (stdout, stderr) = block_on(docker::run_and_wait(exec))?;

    if stderr.len() > 0 {
        error!("stderr: {}", stderr);
    }

    debug!("stdout: {}", stdout);

    let output: Value = serde_json::from_str(&stdout)?;
    Ok(output)
}

///
///
///
fn handle_act_dsl(
    act: &ActInstruction,
    arguments: Map<Value>,
    system: Box<dyn System>,
) -> Result<Value> {
    let instr_file = act.meta.get("instr_file").map(PathBuf::from).expect("Missing `instr_file` metadata property").clone();
    let buf_reader = BufReader::new(File::open(instr_file)?);
    let instructions: Vec<Instruction> = serde_yaml::from_reader(buf_reader)?;
    let instructions = preprocess_instructions(&instructions)?;

    debug!("preprocessed: {:#?}", instructions);

    let environment = InMemoryEnvironment::new(Some(arguments), None);
    let vault = InMemoryVault::new(Default::default());

    let mut machine = Machine::new(
        Box::new(environment),
        system,
        Box::new(vault),
    );

    machine.walk(&instructions)
}

///
///
///
fn handle_mov(
    mov: &MovInstruction,
    cursor: &mut Box<dyn Cursor>,
    environment: &Box<dyn Environment>,
) -> () {
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
            resolve_variable(variable, &environment)
        } else {
            condition.left.clone()
        };
        let rhs = if let Value::Pointer { variable, .. } = &condition.right {
            resolve_variable(variable, &environment)
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

    cursor.go(movement);
}

///
///
///
fn handle_sub(
    sub: &SubInstruction,
    cursor: &mut Box<dyn Cursor>,
) -> () {
    let max_subposition = sub.instructions.len() - 1;
    cursor.enter_sub(max_subposition);
}

///
///
///
fn handle_var(
    var: &VarInstruction,
    cursor: &mut Box<dyn Cursor>,
    environment: &mut Box<dyn Environment>,
) -> () {
    for variable in &var.get {
        let variable_exists = environment.exists(&variable.name);
        if !variable_exists {
            panic!("Variable '{}' does not exists.", variable.name);
        }
    }

    for variable in &var.set {
        if let Some(value) = &variable.value {
            if let Value::Pointer { variable: p_variable, .. } = value {
                let value = resolve_variable(p_variable, environment);
                environment.set(&variable.name, &value);
            } else {
                environment.set(&variable.name, &value);
            }
        }
    }

    cursor.go(Forward);
}

///
///
///
fn resolve_variable(
    variable: &String,
    environment: &Box<dyn Environment>,
) -> Value {
    if variable.contains(".") {
        let segments: Vec<_> = variable.split(".").collect();
        let arch_value = environment.get(segments[0]);
    
        match arch_value {
            Value::Array { entries, .. } => {
                if segments[1] == "length" {
                    Value::Integer(entries.len() as i64)
                } else {
                    panic!("Trying to access undeclared variable.");
                }
            }
            Value::Struct { properties, .. } => {
                if let Some(value) = properties.get(segments[1]) {
                    value.clone()
                } else {
                    panic!("Trying to access undeclared variable.");
                }
            }
            _ => unreachable!(),
        }
    } else {
        environment.get(variable)
    }
}

///
///
///
pub fn preprocess_instructions(
    instructions: &Vec<Instruction>,
) -> Result<Vec<Instruction>> {
    let mut instructions = instructions.clone();

    for instruction in &mut instructions {
        match instruction {
            Instruction::Act(act) => {
                let name = act.meta.get("name").expect("No `name` property in metadata.");
                let version = act.meta.get("version").expect("No `version` property in metadata.");
                let kind = act.meta.get("kind").expect("No `kind` property in metadata.");

                match kind.as_str() {
                    "dsl" => {
                        let instr_file = get_package_source(&name, &version, &kind)?;
                        if instr_file.exists() {
                            act.meta
                                .insert(String::from("instr_file"), String::from(instr_file.to_string_lossy()));
                        }
                    }
                    "cwl" | "ecu" | "oas" => {
                        let image_file = get_package_source(&name, &version, &kind)?;
                        if image_file.exists() {
                            act.meta
                                .insert(String::from("image_file"), String::from(image_file.to_string_lossy()));
                        }
                    }
                    _ => {}
                }
            }
            Instruction::Sub(sub) => {
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
fn get_package_source(
    name: &String,
    version: &String,
    kind: &String,
) -> Result<PathBuf> {
    let packages_dir = appdirs::user_data_dir(Some("brane"), None, false)
        .expect("Couldn't determine Brane data directory.")
        .join("packages");
    let package_dir = packages_dir.join(name).join(version);

    let path = match kind.as_str() {
        "dsl" => package_dir.join("instructions.yml"),
        "cwl" | "ecu" | "oas" => package_dir.join("image.tar"),
        _ => unreachable!(),
    };

    Ok(path)
}
