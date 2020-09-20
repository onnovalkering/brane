use crate::environment::Environment;
use crate::vault::Vault;
use anyhow::Result;
use brane_sys::System;
use brane_exec::{ExecuteInfo, docker};
use specifications::common::Value;
use specifications::instructions::{Instruction, Instruction::*, Move::*, Operator::*};
use specifications::instructions::*;
use crate::cursor::{Cursor, InMemoryCursor};
use std::path::PathBuf;
use futures::executor::block_on;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub struct SyncMachine {
    pub environment: Box<dyn Environment>,
    pub system: Box<dyn System>,
    pub vault: Box<dyn Vault>,
}

impl SyncMachine {
    ///
    ///
    ///
    pub fn new(
        environment: Box<dyn Environment>,
        system: Box<dyn System>,
        vault: Box<dyn Vault>,
    ) -> Self {
        SyncMachine {
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
    ) -> Result<Option<Value>> {
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
                            "cwl" => handle_act(&act, arguments, kind)?,
                            "dsl" => handle_act_dsl(&act, arguments)?,
                            "ecu" => handle_act(&act, arguments, kind)?,
                            "oas" => handle_act(&act, arguments, kind)?,
                            _ => unreachable!()
                        }
                    };

                    self.callback(instructions, output, &mut cursor)?;
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
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    ///
    ///
    ///
    pub fn callback(
        &mut self,
        instructions: &Vec<Instruction>,
        value: Value,
        cursor: &mut Box<dyn Cursor>,
    ) -> Result<()> {
        let instruction = get_current_instruction(instructions, &cursor).unwrap();
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

        cursor.go(Forward);
        Ok(())
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
                } else if variable.contains(".") {
                    let segments: Vec<_> = variable.split(".").collect();
                    let arch_value = environment.get(segments[0]);

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
                    let value = environment.get(variable);
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

///
///
///
fn handle_act_dsl(
    act: &ActInstruction,
    arguments: Map<Value>,
) -> Result<Value> {
    unimplemented!()
}

///
///
///
fn handle_act(
    act: &ActInstruction,
    arguments: Map<Value>,
    kind: &str
) -> Result<Value> {
    let image = act.meta.get("image").expect("Missing `image` metadata property.").clone();
    let image_file = act.meta.get("image_file").map(PathBuf::from).expect("Missing `image_file` metadata property.");

    let mounts = vec![
        String::from("/tmp:/tmp"),
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
            environment.get(variable)
        } else {
            condition.left.clone()
        };
        let rhs = if let Value::Pointer { variable, .. } = &condition.right {
            environment.get(variable)
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
                let value = environment.get(p_variable);
                environment.set(&variable.name, &value);
            } else {
                environment.set(&variable.name, &value);
            }
        }
    }

    cursor.go(Forward);
}
