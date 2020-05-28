use base64::decode;
use brane_init::{exec, Payload};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input as Prompt, Select};
use log::LevelFilter;
use specifications::common::{Literal::*, Value};
use specifications::container::ContainerInfo;
use std::path::PathBuf;
use std::process;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use structopt::StructOpt;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

#[derive(StructOpt)]
#[structopt(name = "init")]
struct CLI {
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
    #[structopt(
        short,
        long = "container",
        name = "container.yml",
        help = "Path to container.yml file",
        default_value = "container.yml"
    )]
    container_info: PathBuf,
    #[structopt(
        short,
        long,
        name = "directory",
        help = "Path to working directory",
        default_value = "/opt/wd"
    )]
    working_dir: PathBuf,
    #[structopt(subcommand)]
    sub_command: SubCommand,
}

#[derive(StructOpt)]
enum SubCommand {
    #[structopt(name = "exec", about = "Execute an action specified in container.yml")]
    Exec {
        #[structopt(help = "Payload as Base64 encoded JSON")]
        payload: String,
    },
    #[structopt(name = "test", about = "Interactivly test an action specified in container.yml")]
    Test,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let options = CLI::from_args();

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if options.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    let container_info = ContainerInfo::from_path(options.container_info)?;
    let working_dir = options.working_dir;

    let payload = match options.sub_command {
        SubCommand::Exec { payload } => {
            let payload = decode(payload).unwrap()[..].to_vec(); // Decode Base64 to byte vector (UTF-8)
            serde_json::from_str(&String::from_utf8(payload)?)? // Convert bytes to String, and deserialize as JSON
        }
        SubCommand::Test => build_payload(&container_info)?,
    };

    let output = exec::handle(container_info, payload, working_dir).await?;
    let output = serde_json::to_string(&output)?; // Serialize output as JSON

    println!("{}", output);
    process::exit(0);
}

///
///
///
fn build_payload(container_info: &ContainerInfo) -> FResult<Payload> {
    let actions: Vec<String> = container_info.actions.keys().map(|k| k.to_string()).collect();

    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("The action the execute")
        .default(0)
        .items(&actions[..])
        .interact()?;

    let name = &actions[index];
    let action = &container_info.actions[name];

    println!("\nPlease provide input for the chosen action:\n");

    let mut arguments = Map::<Value>::new();
    for i in &action.input {
        let data_type = i.data_type.as_str();
        let literal = match data_type {
            "boolean" => Boolean(prompt(&i.name, data_type)?),
            "integer" => Integer(prompt(&i.name, data_type)?),
            "real" => Decimal(prompt(&i.name, data_type)?),
            "string" => Str(prompt(&i.name, data_type)?),
            _ => continue,
        };

        let value = Value::Literal(literal);
        arguments.insert(i.name.clone(), value);
    }

    let payload = Payload {
        action: name.clone(),
        arguments,
        identifier: String::from("ID"),
        callback_url: None,
        monitor_url: None,
    };

    Ok(payload)
}

///
///
///
fn prompt<T>(
    name: &str,
    data_type: &str,
) -> std::io::Result<T>
where
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    Prompt::with_theme(&ColorfulTheme::default())
        .with_prompt(&format!("{} ({})", name, data_type))
        .interact()
}
