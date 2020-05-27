use base64::{decode, encode};
use brane_init::exec;
use log::LevelFilter;
use specifications::container::ContainerInfo;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

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

    match options.sub_command {
        SubCommand::Exec { payload } => {
            let payload = decode(payload).unwrap()[..].to_vec(); // Decode Base64 to byte vector (UTF-8)
            let payload = serde_json::from_str(&String::from_utf8(payload)?)?; // Convert bytes to String, and deserialize as JSON

            let output = exec::handle(container_info, payload, working_dir).await?;
            let output = encode(serde_json::to_string(&output)?); // Serialize output as JSON, and encode as Base64

            println!("{}", output);
        }
        SubCommand::Test => {
            unimplemented!();
        }
    }

    process::exit(0);
}
