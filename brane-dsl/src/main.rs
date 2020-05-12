#[macro_use]
extern crate human_panic;
#[macro_use]
extern crate log;

use bakery::compiler;
use bakery::configuration;
use bakery::repl;
use bakery::setup;
use log::LevelFilter;
use serde::ser::Serialize;
use serde_yaml;
use serde_yaml::Error as YError;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "bakery", about = "The Bakery command-line interface.")]
struct CLI {
    #[structopt(subcommand)]
    command: Command,
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "compile", about = "Compile a Bakery file to Brane RI instructions")]
    Compile {
        /// Path to the source code file
        input: PathBuf,
        // Path to the output file
        output: Option<PathBuf>,
    },
    #[structopt(name = "repl", about = "Start a Bakery REPL shell")]
    Repl {},
    #[structopt(name = "setup", about = "Setup and configure this command-line interface")]
    Setup {},
}

fn main() {
    let options = CLI::from_args();

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if options.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();

        setup_panic!(Metadata {
            name: "Bakery CLI".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        });
    }

    let command = options.command;
    let config = configuration::load();

    if let Some(config) = config {
        debug!("Using registry: {}", config.registry_api_url);

        match command {
            Command::Compile { input, output } => {
                if !input.exists() {
                    println!("Cannot access file at provided path.");
                    return;
                }

                let source_code = fs::read_to_string(input.clone());
                if source_code.is_err() {
                    println!("Cannot read file at provided path.");
                    return;
                }

                let instructions = compiler::handle(source_code.unwrap(), &config).unwrap();
                let output = if let Some(output) = output {
                    output
                } else {
                    let mut output = input;
                    output.set_extension("yaml");

                    output
                };

                write_yaml(&output, instructions).unwrap();
            }
            Command::Repl {} => repl::handle(config).unwrap(),
            Command::Setup {} => setup::handle().unwrap(),
        }
    } else {
        match command {
            Command::Setup {} => setup::handle().unwrap(),
            _ => println!("No configuration found, please run 'bakery setup' first."),
        }
    }
}

pub fn write_yaml<T>(
    path: &PathBuf,
    yaml: T,
) -> Result<(), YError>
where
    T: Serialize,
{
    let contents = serde_yaml::to_string(&yaml)?;

    write(path, &contents);

    Ok(())
}

pub fn write(
    path: &PathBuf,
    contents: &String,
) {
    let directory = path.parent().unwrap();
    fs::create_dir_all(&directory).expect("Couldn't create directories.");

    let mut buffer = File::create(path).expect("Failed to create file.");

    write!(buffer, "{}", contents).unwrap();
}
