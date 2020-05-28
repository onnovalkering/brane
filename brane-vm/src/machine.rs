use crate::environment::Environment;
use brane_exec::{docker, ExecuteInfo};
use serde_json::json;
use specifications::common::Value;
use specifications::instructions::{Instruction, Instruction::*, Move, Move::*, Operator::*};
use std::path::PathBuf;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub struct Machine {
    cursor: usize,
    environment: Box<dyn Environment>,
}

impl Machine {
    ///
    ///
    ///
    pub fn new(environment: Box<dyn Environment>) -> Self {
        Machine { cursor: 0, environment }
    }

    ///
    ///
    ///
    pub async fn interpret(
        &mut self,
        instructions: Vec<Instruction>,
    ) -> FResult<()> {
        let cursor_max = instructions.len();

        while self.cursor != cursor_max {
            let instruction = instructions.get(self.cursor).unwrap().clone();
            let movement = match instruction {
                Act(_) => self.handle_act(instruction).await?,
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

        Ok(())
    }

    ///
    ///
    ///
    async fn handle_act(
        &mut self,
        instruction: Instruction,
    ) -> FResult<Move> {
        let act = if let Act(act) = instruction {
            act
        } else {
            bail!("Not a ACT instruction.");
        };

        let image = act.meta.get("image").expect("Missing `image` metadata property");
        let image_file = act.meta.get("image_file").map(PathBuf::from);
        let payload = json!({
            "identifier": "1+1",
            "action": act.name,
            "arguments": act.input,
        });

        let exec = ExecuteInfo::new(image.clone(), image_file, payload);
        let (stdout, _) = docker::run_and_wait(exec).await?;

        if let Some(ref assignment) = act.assignment {
            let output: Map<Value> = serde_json::from_str(&stdout).unwrap();
            let value = output.get("c").unwrap();

            if value.data_type() != act.data_type.unwrap() {
                bail!("Data types don't match!");
            }

            self.environment.set(assignment, value);
        }

        Ok(Forward)
    }

    ///
    ///
    ///
    fn handle_mov(
        &mut self,
        instruction: Instruction,
    ) -> FResult<Move> {
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
        _instruction: Instruction,
    ) -> FResult<Move> {
        unimplemented!();
    }

    ///
    ///
    ///
    fn handle_var(
        &mut self,
        instruction: Instruction,
    ) -> FResult<Move> {
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
