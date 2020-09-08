use anyhow::Result;
use base64::decode;
use brane_init::{callback, exec, Payload, relay};
use log::LevelFilter;
use specifications::container::ContainerInfo;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;
use std::thread;

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

    let container_info = ContainerInfo::from_path(options.container_info)?;
    let working_dir = options.working_dir;

    let payload: Payload = match options.sub_command {
        SubCommand::Exec { payload } => {
            let payload = decode(payload).unwrap()[..].to_vec(); // Decode Base64 to byte vector (UTF-8)
            serde_json::from_str(&String::from_utf8(payload)?)? // Convert bytes to String, and deserialize as JSON
        }
    };

    let invocation_id = payload.invocation_id.clone();
    if let Some(callback_url) = payload.callback_url.clone() {
        thread::spawn(move || {
            relay::start(callback_url, invocation_id).unwrap();
        });
    }

    let output = exec::handle(&payload.action, &payload.arguments, container_info, working_dir).await?;
    if let Some(callback_url) = payload.callback_url {
        callback::submit(&callback_url, invocation_id, &output).await?;
    }

    process::exit(0);
}
