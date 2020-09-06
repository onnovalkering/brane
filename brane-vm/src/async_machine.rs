use crate::environment::Environment;
use crate::vault::Vault;
use anyhow::Result;
use brane_sys::System;
use specifications::common::Value;
use specifications::instructions::{Instruction, Instruction::*, Move, Move::*, Operator::*, ActInstruction};
use crate::cursor::Cursor;
use brane_exec::delegate;

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
    pub system: Box<dyn System>,
    pub vault: Box<dyn Vault>,
}

impl AsyncMachine {
    ///
    ///
    ///
    pub fn new(
        instructions: Vec<Instruction>,
        cursor: Box<dyn Cursor>,
        environment: Box<dyn Environment>,
        system: Box<dyn System>,
        vault: Box<dyn Vault>,
    ) -> Self {
        AsyncMachine {
            cursor,
            environment,
            instructions,
            system,
            vault,
        }
    }

    ///
    ///
    ///
    pub fn walk(
        &mut self,
    ) -> Result<MachineResult> {
        while let Some(instruction) = get_current_instruction(&self.instructions, &self.cursor) {
            let movement = match instruction {
                Act(act) => {
                    let arguments = prepare_arguments(&act.input, &self.environment, &self.vault);
                    let kind = act.meta.get("kind").expect("Missing `kind` metadata property.");

                    match kind.as_str() {
                        "std" => {
                            let value = delegate::exec_std(&act, arguments, &self.system)?;
                            if let Some(value) = value {
                                self.callback(value)?;
                            }

                            Some(Forward)
                        },
                        _ => handle_act(&act, &self.environment, &self.system, &self.vault)
                    }

                },
                Mov(_) => handle_mov(&instruction, &self.environment),
                Sub(_) => handle_sub(&instruction, &mut self.cursor),
                Var(_) => handle_var(&instruction, &mut self.environment),
            };

            if let Some(movement) = movement {
                self.cursor.go(movement);
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
) -> Map<Value> {
    let mut arguments = Map::<Value>::new();

    for (name, value) in input {
        match &value {
            Value::Pointer { variable, secret, .. } => {
                if *secret {
                    let value = vault.get(variable);
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

    arguments
}


fn handle_act(
    instruction: &ActInstruction,
    environment: &Box<dyn Environment>,
    system: &Box<dyn System>,
    vault: &Box<dyn Vault>,
) -> Option<Move> {
    unimplemented!()
}

///
///
///
fn handle_mov(
    instruction: &Instruction,
    environment: &Box<dyn Environment>,
) -> Option<Move> {
    let mov = if let Mov(mov) = instruction {
        mov
    } else {
        unreachable!();
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

    Some(movement)
}

///
///
///
fn handle_sub(
    instruction: &Instruction,
    cursor: &mut Box<dyn Cursor>,
) -> Option<Move> {
    let sub = if let Sub(sub) = instruction {
        sub
    } else {
        unreachable!();
    };

    let max_subposition = sub.instructions.len() - 1;
    cursor.enter_sub(max_subposition);

    None
}

///
///
///
fn handle_var(
    instruction: &Instruction,
    environment: &mut Box<dyn Environment>,
) -> Option<Move> {
    let var = if let Var(var) = instruction {
        var
    } else {
        unreachable!();
    };

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

    Some(Forward)
}
