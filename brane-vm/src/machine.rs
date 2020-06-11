use crate::environment::Environment;
use brane_exec::delegate;
use anyhow::Result;
use specifications::common::Value;
use futures::executor::block_on;
use specifications::instructions::{Instruction, Instruction::*, Move, Move::*, Operator::*};

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub struct Machine {
    cursor: usize,
    pub environment: Box<dyn Environment>,
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
                Value::Pointer { variable, .. } => {
                    if variable.contains(".") {
                        let segments: Vec<_> = variable.split(".").collect();
                        let arch_value = self.environment.get(segments[0]);

                        if let Value::Struct { properties, .. } = arch_value {
                            if let Some(value) = properties.get(segments[1]) {
                                arguments.insert(name.clone(), value.clone());
                            } else {
                                panic!("Trying to access undeclared variable.");
                            }
                        }
                    } else {
                        let value = self.environment.get(variable);
                        arguments.insert(name.clone(), value);
                    }
                },
                _ => unimplemented!()
            }
        }

        let kind = act.meta.get("kind").expect("Missing `kind` metadata property.");
        let output = match kind.as_str() {
            "cwl" => delegate::exec_cwl(&act, arguments)?,
            "ecu" => block_on(delegate::exec_ecu(&act, arguments))?,
            "oas" => block_on(delegate::exec_oas(&act, arguments))?,
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

        let mut sub_machine = Machine::new(self.environment.child());
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
