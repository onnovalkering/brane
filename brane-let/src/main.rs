use anyhow::Result;
use brane_let::callback::Callback;
use brane_let::redirector;
use clap::Clap;
use dotenv::dotenv;
use log::LevelFilter;
use socksx::{self, Socks6Client};
use std::{future::Future, process, time::Duration};
use tokio::net::{TcpListener, TcpStream};

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
    #[clap(short, long, env = "BRANE_PROXY_ADDRESS")]
    proxy_address: Option<String>,
    /// Prints debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    #[clap(subcommand)]
    sub_command: SubCommand,
}

#[derive(Clap, Clone)]
enum SubCommand {
    /// Don't perform any operation and return nothing.
    #[clap(name = "no-op")]
    NoOp,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let opts = Opts::parse();

    let application_id = opts.application_id.clone();
    let location_id = opts.location_id.clone();
    let job_id = opts.job_id.clone();
    let callback_to = opts.callback_to.clone();
    let proxy_address = opts.proxy_address.clone();

    // Configure logger.
    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if opts.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    // Start redirector in the background, if proxy address is set.
    if proxy_address.is_some() {
        tokio::spawn(redirector::start(proxy_address.unwrap()));
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Callbacks may be called at any time of the execution.
    let callback = Callback::new(&application_id, &location_id, &job_id, &callback_to);

    // Wrap actual execution, so we can always log errors.
    match run(opts.sub_command, callback).await {
        Ok(_) => process::exit(0),
        Err(error) => {
            eprintln!("{:?}", error);
            process::exit(1);
        }
    }
}

///
///
///
async fn run(
    sub_command: SubCommand,
    callback: impl Future<Output = Result<Callback>>,
) -> Result<()> {
    let mut callback = callback.await?;

    // Perform READY callback as soon as possible.
    callback.ready(None).await?;

    // TODO: start hearthbeat background worker.
    callback.heartbeat(None).await?;

    // TODO: perform initialization.
    callback.initialized(None).await?;

    match sub_command {
        SubCommand::NoOp => {
            // TODO: start execution.
            callback.started(None).await?;
        }
    };

    // Perform final FINISHED callback.
    callback.finished(None).await?;

    Ok(())
}
