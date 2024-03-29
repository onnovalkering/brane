use anyhow::{ensure, Context, Result};
use brane_let::callback::Callback;
use brane_let::exec_code;
use brane_let::exec_oas;
use brane_let::redirector;
use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use serde::de::DeserializeOwned;
use socksx::socks6::options::MetadataOption;
use socksx::socks6::options::SocksOption;
use specifications::common::Value;
use std::path::PathBuf;
use std::process::Command;
use std::{future::Future, process};

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(short, long, env = "BRANE_APPLICATION_ID")]
    application_id: String,
    #[clap(short, long, env = "BRANE_LOCATION_ID")]
    location_id: String,
    #[clap(short, long, env = "BRANE_JOB_ID")]
    job_id: String,
    #[clap(short, long, env = "BRANE_CALLBACK_TO")]
    callback_to: Option<String>,
    #[clap(short, long, env = "BRANE_PROXY_ADDRESS")]
    proxy_address: Option<String>,
    #[clap(short, long, env = "BRANE_MOUNT_DFS")]
    mount_dfs: Option<String>,
    /// Prints debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    #[clap(subcommand)]
    sub_command: SubCommand,
}

#[derive(Parser, Clone)]
enum SubCommand {
    /// Execute arbitrary source code and return output
    #[clap(name = "code")]
    Code {
        /// Function to execute
        function: String,
        /// Input arguments
        arguments: String,
        #[clap(short, long, env = "BRANE_WORKDIR", default_value = "/opt/wd")]
        working_dir: PathBuf,
    },
    /// Don't perform any operation and return nothing
    #[clap(name = "no-op")]
    NoOp,
    /// Call a Web API and return output
    #[clap(name = "oas")]
    WebApi {
        /// Function to execute
        function: String,
        /// Input arguments
        arguments: String,
        #[clap(short, long, env = "BRANE_WORKDIR", default_value = "/opt/wd")]
        working_dir: PathBuf,
    },
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

    // Mount DFS via JuiceFS.
    if let Some(ref mount_dfs) = opts.mount_dfs {
        let status = Command::new("/juicefs")
            .args(vec!["mount", "-d", mount_dfs, "/data"])
            .status()
            .expect("Failed to execute '/juicefs' binary.");

        ensure!(status.success(), "Failed to mount distributed filesystem.");
    }

    // Start redirector in the background, if proxy address is set.
    if let Some(proxy_address) = proxy_address {
        let options = vec![
            MetadataOption::new(1, application_id.clone()),
            MetadataOption::new(2, location_id.clone()),
            MetadataOption::new(3, job_id.clone()),
        ];

        let options = options.into_iter().map(SocksOption::Metadata).collect();
        redirector::start(proxy_address, options).await?;
    }

    // Callbacks may be called at any time of the execution.
    let callback = callback_to.map(|callback_to| Callback::new(application_id, location_id, job_id, callback_to));

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
    callback: Option<impl Future<Output = Result<Callback>>>,
) -> Result<()> {
    // Setup callback channel (gRPC) if enabled.
    let mut callback: Option<Callback> = if let Some(callback) = callback {
        let mut callback = callback.await?;
        callback.ready(None).await?;

        Some(callback)
    } else {
        None
    };

    let output = match sub_command {
        SubCommand::Code {
            function,
            arguments,
            working_dir,
        } => exec_code::handle(function, decode_b64(arguments)?, working_dir, &mut callback.as_mut()).await,
        SubCommand::NoOp => {
            if let Some(callback) = &mut callback.as_mut() {
                callback.initialized(None).await?;
                callback.started(None).await?;
            }

            Ok(Value::Unit)
        }
        SubCommand::WebApi {
            function,
            arguments,
            working_dir,
        } => exec_oas::handle(function, decode_b64(arguments)?, working_dir, &mut callback.as_mut()).await,
    };

    // Perform final FINISHED callback.
    match output {
        Ok(value) => {
            let output = serde_json::to_string(&value)?;

            if let Some(callback) = &mut callback.as_mut() {
                let payload: Vec<u8> = output.into_bytes();
                callback.finished(Some(payload)).await?;
            } else {
                // Otherwise, print output to stdout so it can be parsed by caller.
                println!("{}", base64::encode(&output));
            }

            Ok(())
        }
        Err(error) => {
            if let Some(callback) = &mut callback.as_mut() {
                callback.failed(None).await?;
            }

            Err(error)
        }
    }
}

///
///
///
fn decode_b64<T>(input: String) -> Result<T>
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
