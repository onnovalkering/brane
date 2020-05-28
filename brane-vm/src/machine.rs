use crate::environment::Environment;
use brane_exec::{docker, ExecuteInfo};
use serde_json::json;
use specifications::common::{Literal, Value};
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
                Act { .. } => self.handle_act(instruction).await?,
                Mov { .. } => self.handle_mov(instruction)?,
                Sub { .. } => self.handle_sub(instruction)?,
                Var { .. } => self.handle_var(instruction)?,
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
        let (meta, assignment, name, input, data_type) = instruction.as_act()?;
        let mut arguments = Map::<Literal>::new();
        for (key, value) in input.iter() {
            let literal = match value {
                Value::Literal(literal) => literal.clone(),
                Value::Variable(variable) => {
                    if let Value::Literal(literal) = self.environment.get(variable) {
                        literal
                    } else {
                        unreachable!()
                    }
                }
                _ => unreachable!(),
            };

            arguments.insert(key.clone(), literal);
        }

        let image = meta.get("image").expect("Missing `image` metadata property");
        let image_file = meta.get("image_file").map(|p| PathBuf::from(p));
        let payload = json!({
            "identifier": "1+1",
            "action": name,
            "arguments": arguments,
        });

        let exec = ExecuteInfo::new(image.clone(), image_file, payload);
        let (stdout, _) = docker::run_and_wait(exec).await?;

        if let Some(ref assignment) = assignment {
            let output: Map<Value> = serde_json::from_str(&stdout).unwrap();
            let value = output.get("c").unwrap();

            if value.get_complex() != data_type.unwrap() {
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
        let (_, conditions, branches) = instruction.as_mov()?;

        let mut movement = if conditions.len() == branches.len() {
            // Default, may be overriden based on the specific branch
            Forward
        } else {
            // Else branch is at n+1 position, where n = conditions.len()
            branches.last().unwrap().clone()
        };

        for (i, condition) in conditions.iter().enumerate() {
            // Get values from environment, in the case of variables
            let lhs = if let Value::Variable(ref name) = &condition.left {
                self.environment.get(name)
            } else {
                condition.left.clone()
            };
            let rhs = if let Value::Variable(ref name) = &condition.right {
                self.environment.get(name)
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
                movement = branches.get(i).unwrap().clone();
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
        // let (_, instructions) = instruction.as_sub()?;
        // let child_env = self.environment.child();

        // let mut child_vm = Machine::new(child_env);
        // child_vm.interpret(instructions)?;

        unimplemented!();
    }

    ///
    ///
    ///
    fn handle_var(
        &mut self,
        instruction: Instruction,
    ) -> FResult<Move> {
        let (_, get, set) = instruction.as_var()?;

        for variable in get {
            let variable_exists = self.environment.exists(&variable.name);
            ensure!(variable_exists, "Variable '{}' does not exists.", variable.name);
        }

        for variable in set {
            self.environment.set(&variable.name, &variable.value);
        }

        Ok(Forward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::InMemoryEnvironment;
    use specifications::common::{Literal, Value};

    type Map<T> = std::collections::HashMap<String, T>;

    #[tokio::test]
    async fn it_works() {
        let mut meta = Map::<String>::new();
        meta.insert("image".to_string(), "arithmetic:1.0.0".to_string());
        let mut input = Map::<Value>::new();
        input.insert(String::from("a"), Value::Variable(String::from("a")));
        input.insert(String::from("b"), Value::Variable(String::from("b")));

        let instructions = vec![
            Instruction::new_set_var(
                String::from("a"),
                Value::Literal(Literal::Integer(1)),
                String::from("local"),
            ),
            Instruction::new_set_var(
                String::from("b"),
                Value::Literal(Literal::Integer(2)),
                String::from("local"),
            ),
            Instruction::new_act(
                String::from("add"),
                input,
                meta,
                Some(String::from("c")),
                Some(String::from("integer")),
            ),
        ];

        let environment = InMemoryEnvironment::new(None, None);
        let mut machine = Machine::new(Box::new(environment));
        machine.interpret(instructions).await.unwrap();

        assert_eq!(machine.environment.get("c"), Value::Literal(Literal::Integer(3)));
    }
}
