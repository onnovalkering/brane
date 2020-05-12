use crate::compiler;
use crate::configuration::Configuration;
use crate::functions::FunctionPattern;
use linefeed::{Interface, ReadResult};
use serde_yaml;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<T, String>;

///
///
///
pub fn handle(config: Configuration) -> FResult<()> {
    let interface = Interface::new("bakery-repl")?;
    interface.set_prompt("bakery> ")?;

    let mut functions = Vec::<FunctionPattern>::new();
    let mut variables = Map::<String>::new();

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        };

        let instructions = compiler::compile(line, &mut functions, &mut variables, &config).unwrap();
        let instructions = serde_yaml::to_string(&instructions)?;

        println!("{}\n", instructions);
    }

    println!("Goodbye.");
    Ok(())
}
