#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use brane_job::interface::{Command, CommandKind};
use brane_job::{cmd_cancel, cmd_check, cmd_create};
use bytes::BytesMut;
use clap::Clap;
use dotenv::dotenv;
use futures::stream::FuturesUnordered;
use futures::{StreamExt, TryStreamExt};
use log::LevelFilter;
use prost::Message;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::producer::FutureProducer;
use rdkafka::util::Timeout;
use rdkafka::{config::ClientConfig, Message as KafkaMesage};
use rdkafka::{message::ToBytes, producer::FutureRecord};
use tokio::task::JoinHandle;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clap)]
#[clap(version = VERSION)]
struct Opts {
    /// Topic to receive commands from
    #[clap(long = "cmd-topic", env = "COMMAND_TOPIC")]
    command_topic: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Topic to send events to
    #[clap(long = "evt-topic", env = "EVENT_TOPIC")]
    event_topic: String,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-job", env = "GROUP_ID")]
    group_id: String,
    /// Number of workers
    #[clap(short = 'w', long, default_value = "1", env = "NUM_WORKERS")]
    num_workers: u8,
}

#[tokio::main]
async fn main() {
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

    // Spawn workers, using Tokio tasks and thread pool.
    let workers = (0..opts.num_workers)
        .map(|_| {
            tokio::spawn(start_worker(
                opts.brokers.clone(),
                opts.group_id.clone(),
                opts.command_topic.clone(),
                opts.event_topic.clone(),
            ))
        })
        .collect::<FuturesUnordered<JoinHandle<_>>>();

    // Wait for workers to finish, print any errors.
    workers
        .map(|r| r.unwrap())
        .for_each(|r| async {
            if let Err(error) = r {
                error!("{:?}", error);
            };
        })
        .await;
}

///
///
///
pub async fn start_worker(
    brokers: String,
    group_id: String,
    input_topic: String,
    output_topic: String,
) -> Result<()> {
    let output_topic = output_topic.as_ref();

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &group_id)
        .set("bootstrap.servers", &brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .context("Failed to create Kafka consumer.")?;

    consumer
        .subscribe(&[&input_topic])
        .context(format!("Failed to subscribe to command topic: {}", input_topic))?;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    // Create the outer pipeline on the message stream.
    let stream_processor = consumer.stream().try_for_each(|borrowed_message| {
        let owned_message = borrowed_message.detach();
        let owned_producer = producer.clone();

        async move {
            let cmd_key = owned_message
                .key()
                .map(String::from_utf8_lossy)
                .map(String::from)
                .unwrap_or_default();

            if cmd_key.is_empty() {
                warn!("Received message without a key. Ignoring it.");
                return Ok(());
            }

            if let Some(payload) = owned_message.payload() {
                // Decode payload into a command message.
                let command = Command::decode(payload).unwrap();
                let kind = CommandKind::from_i32(command.kind).unwrap();

                // Ignore unkown commands, as we can't dispatch it.
                if kind == CommandKind::Unknown {
                    warn!("Received UNKOWN command (key: {}). Ignoring it.", cmd_key);
                    return Ok(());
                }

                info!("Received {} command (key: {}).", kind, cmd_key);
                debug!("{:?}", command);

                // Dispatch command message to appropriate handlers.
                let events = match kind {
                    CommandKind::Create => cmd_create::handle(command),
                    CommandKind::Cancel => cmd_cancel::handle(command),
                    CommandKind::Check => cmd_check::handle(command),
                    CommandKind::Unknown => unreachable!(),
                };

                match events {
                    Ok(events) => {
                        for (i, event) in events.iter().enumerate() {
                            let evt_key = format!("{}-evt{}", cmd_key, i);

                            // Encode event message into a payload (bytes)
                            let mut payload = BytesMut::with_capacity(64);
                            event.encode(&mut payload).unwrap();

                            // Send event on output topic
                            let message = FutureRecord::to(&output_topic)
                                .key(&evt_key)
                                .payload(payload.to_bytes());

                            if let Err(error) = owned_producer.send(message, Timeout::Never).await {
                                error!("Failed to send event (key: {}): {:?}", evt_key, error);
                            }
                        }
                    }
                    Err(error) => error!("{:?}", error),
                };
            } else {
                info!("Received message without payload (key: {}).", cmd_key);
            }

            Ok(())
        }
    });

    stream_processor.await.context("Processor failed.")
}
