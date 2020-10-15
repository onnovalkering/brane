use anyhow::{Context, Result};
use brane_init::{callback, exec_cwl, exec_ecu, exec_oas};
use log::LevelFilter;
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "init")]
struct CLI {
    #[structopt(short, long, name = "URL", help = "The URL to post the output (callback)")]
    callback_url: Option<String>,
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
    #[structopt(short, long, name = "ID", help = "Identifier of the associated invocation")]
    invocation_id: Option<i32>,
    #[structopt(subcommand)]
    sub_command: SubCommand,
    #[structopt(
        short,
        long,
        name = "PATH",
        help = "Path to working directory",
        default_value = "/opt/wd"
    )]
    working_dir: PathBuf,
}

#[derive(StructOpt)]
enum SubCommand {
    #[structopt(about = "Execute a function, using the CWL package handler")]
    Cwl {
        #[structopt(help = "Name of the function to execute")]
        function: String,
        #[structopt(help = "Arguments as Base64 encoded JSON")]
        arguments: String,
        #[structopt(short, long, name = "PATH", help = "Path to output directory")]
        output_dir: Option<PathBuf>,
    },

    #[structopt(about = "Execute a function, using the ECU package handler")]
    Ecu {
        #[structopt(help = "Name of the function to execute")]
        function: String,
        #[structopt(help = "Arguments as Base64 encoded JSON")]
        arguments: String,
    },

    #[structopt(about = "Execute a function, using the OAS package handler")]
    Oas {
        #[structopt(help = "Name of the function to execute")]
        function: String,
        #[structopt(help = "Arguments as Base64 encoded JSON")]
        arguments: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = CLI::from_args();

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if options.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    match run(options).await {
        Ok(_) => process::exit(0),
        Err(error) => {
            println!("{:?}", error); // Anyhow
            process::exit(1);
        }
    }
}

///
///
///
async fn run(options: CLI) -> Result<()> {
    let output = match options.sub_command {
        SubCommand::Cwl {
            function,
            arguments,
            output_dir,
        } => exec_cwl::handle(function, decode(arguments)?, options.working_dir, output_dir)?,
        SubCommand::Ecu { function, arguments } => exec_ecu::handle(function, decode(arguments)?, options.working_dir)?,
        SubCommand::Oas { function, arguments } => {
            exec_oas::handle(function, decode(arguments)?, options.working_dir).await?
        }
    };

    if let Some(callback_url) = options.callback_url {
        callback::submit(&callback_url, options.invocation_id.unwrap(), &output).await?;
    } else {
        println!("{}", serde_json::to_string(&output)?);
    }

    Ok(())
}

///
///
///
fn decode<T>(input: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let input =
        base64::decode(input).with_context(|| "Decoding failed, encoded input doesn't seem to be Base64 encoded.")?;

    let input = String::from_utf8(input[..].to_vec())
        .with_context(|| "Conversion failed, decoded input doesn't seem to be UTF-8 encoded.")?;

    serde_json::from_str(&input)
        .with_context(|| "Deserialization failed, decoded input doesn't seem to be as expected.")
}
