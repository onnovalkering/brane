use crate::registry;
use anyhow::Result;
use brane_dsl::compiler::{Compiler, CompilerOptions};
use brane_vm::{environment::InMemoryEnvironment, machine::Machine};
use linefeed::{Interface, ReadResult};
use specifications::common::Value;

pub async fn start() -> Result<()> {
    println!("Starting interactive session, press Ctrl+D to exit.\n");

    let interface = Interface::new("brane-repl")?;
    interface.set_prompt("brane> ")?;

    // Prepare DSL compiler
    let package_index = registry::get_package_index(false).await?;
    let mut compiler = Compiler::new(CompilerOptions::repl(), package_index)?;

    // Prepare machine
    let environment = InMemoryEnvironment::new(None, None);
    let mut machine = Machine::new(Box::new(environment));

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        };

        let instructions = compiler.compile(&line);

        match instructions {
            Ok(instructions) => {
                let output = machine.interpret(instructions)?;
                if let Some(value) = output {
                    if let Value::Pointer { ref variable, .. } = value {
                        let value = machine.environment.get(variable);
                        println!("{:?}", value);
                    } else {
                        println!("{:?}", value);
                    }
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
