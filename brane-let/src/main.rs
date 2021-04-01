use anyhow::Result;
use clap::Clap;
use dotenv::dotenv;
use log::LevelFilter;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(short, long, env = "BRANE_APPLICATION_ID")]
    application_id: String,
    #[clap(short, long, env = "BRANE_LOCATION_ID")]
    location_id: String,
    #[clap(short, long, env = "BRANE_JOB_ID")]
    job_id: String,
    #[clap(short, long, env = "BRANE_CALLBACK_TO")]
    callback_to: String,
    /// Prints debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Execute a function, using the ECU package handler
    #[clap(name = "ecu")]
    Ecu {
        /// Name of the function to execute
        #[clap(short, long)]
        function: String,
        /// Arguments as Base64 encoded JSON
        #[clap(short, long)]
        arguments: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let opts = Opts::parse();

    // Configure logger.
    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if opts.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    // TODO...

    Ok(())
}
