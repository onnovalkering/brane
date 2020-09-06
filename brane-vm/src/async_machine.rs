use crate::environment::Environment;
use crate::vault::Vault;
use anyhow::Result;
use brane_sys::System;
use specifications::common::Value;
use specifications::instructions::{Instruction, Instruction::*, Move, Move::*, Operator::*};
use crate::cursor::Cursor;

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
                Act(_) => unimplemented!(),
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
