use crate::environment::Environment;
use specifications::common::{Literal, Value};
use specifications::instructions::{Instruction, Instruction::*, Move, Move::*, Operator::*};

type FResult<T> = Result<T, failure::Error>;

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
    pub fn interpret(
        &mut self,
        instructions: Vec<Instruction>,
    ) -> FResult<()> {
        let cursor_max = instructions.len();

        while self.cursor != cursor_max {
            let instruction = instructions.get(self.cursor).unwrap().clone();
            let movement = match instruction {
                Act { .. } => self.handle_act(instruction)?,
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
    fn handle_act(
        &mut self,
        _instruction: Instruction,
    ) -> FResult<Move> {
        let name = String::from("counter");
        let counter = self.environment.get(&name);
        if let Value::Literal(Literal::Integer(value)) = counter {
            println!("Counter: {}", value + 1);
            self.environment
                .set(&name, &Value::Literal(Literal::Integer(value + 1)));
        } else {
            bail!("invalid");
        };

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
        instruction: Instruction,
    ) -> FResult<Move> {
        let (_, instructions) = instruction.as_sub()?;
        let child_env = self.environment.child();

        let mut child_vm = Machine::new(child_env);
        child_vm.interpret(instructions)?;

        Ok(Forward)
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
    use specifications::common::{Argument, Literal, Value};
    use specifications::instructions::{Condition};

    type Map<T> = std::collections::HashMap<String, T>;

    #[test]
    fn it_works() {
        let instructions = vec![
            Instruction::new_set_var(
                String::from("counter"),
                Value::Literal(Literal::Integer(0)),
                String::from("local"),
            ),
            Instruction::new_sub(vec![
                Instruction::new_act(
                    String::from("???"),
                    Map::<Argument>::new(),
                    Map::<String>::new(),
                    None,
                    None,
                ),
                Instruction::new_mov(
                    vec![Condition::le(
                        Value::Variable(String::from("counter")),
                        Value::Literal(Literal::Integer(10)),
                    )],
                    vec![Backward],
                ),
            ]),
            Instruction::new_act(
                String::from("???"),
                Map::<Argument>::new(),
                Map::<String>::new(),
                None,
                None,
            ),
            Instruction::new_mov(
                vec![Condition::le(
                    Value::Variable(String::from("counter")),
                    Value::Literal(Literal::Integer(3)),
                )],
                vec![Backward],
            ),
        ];

        let environment = InMemoryEnvironment::new(None, None);
        let mut machine = Machine::new(Box::new(environment));
        machine.interpret(instructions).unwrap();
    }
}
