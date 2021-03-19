use anyhow::{Context, Result};
use brane_job::interface::{Command, CommandKind};
use bytes::BytesMut;
use clap::Clap;
use dotenv::dotenv;
use log::LevelFilter;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clap)]
#[clap(version = VERSION)]
struct Opts {
    /// Topic to receive commands from
    #[clap(short, long = "cmd-topic", env = "COMMAND_TOPIC")]
    command_topic: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG")]
    debug: bool,
    /// Topic to send events to
    #[clap(short, long = "evt-topic", env = "EVENT_TOPIC")]
    _event_topic: String,
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

    let brokers = opts.brokers;
    let command_topic = opts.command_topic;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    let command = Command::new(CommandKind::Create, None, Some("node"), Some("busybox"), None, None);
    let mut payload = BytesMut::with_capacity(64);
    command.encode(&mut payload)?;

    let message = FutureRecord::to(&command_topic).key("a").payload(payload.to_bytes());
    match producer.send(message, Timeout::Never).await {
        Ok(_) => println!("Message send!"),
        Err(_) => println!("Failed to send message!"),
    }

    Ok(())
}
