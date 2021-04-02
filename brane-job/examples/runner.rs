#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use brane_job::interface::{Command, CommandKind, Event, EventKind};
use bytes::BytesMut;
use clap::Clap;
use dashmap::DashMap;
use dotenv::dotenv;
use futures::TryStreamExt;
use log::LevelFilter;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::Offset;
use rdkafka::{Message as KafkaMesage, TopicPartitionList};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Action {
    Run(RunAction),
    WaitUntil(WaitUntilAction),
}

#[derive(Clone, Debug, Deserialize)]
struct RunAction {
    pub identifier: String,
    pub application: String,
    pub location: String,
    pub image: String,
    pub command: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct WaitUntilAction {
    pub identifier: String,
    pub state: String,
}

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(short, long, default_value = "./resources/noop-app.yml")]
    application: PathBuf,
    /// Topic to send commands to
    #[clap(short, long = "cmd-topic", env = "COMMAND_TOPIC")]
    command_topic: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Topic to receive events from
    #[clap(short, long = "evt-topic", env = "EVENT_TOPIC")]
    event_topic: String,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-job-runner")]
    group_id: String,
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

    let application = opts.application;
    let brokers = opts.brokers;
    let command_topic = opts.command_topic;
    let event_topic = opts.event_topic;
    let group_id = opts.group_id;

    let states: Arc<DashMap<String, String>> = Arc::new(DashMap::new());

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    // Process events in the background
    tokio::spawn(start_event_monitor(brokers, group_id, event_topic, states.clone()));

    let app_reader = BufReader::new(File::open(application)?);
    let app_actions: Vec<Action> = serde_yaml::from_reader(app_reader)?;

    // Start application by execution actions from YAML file.
    for action in app_actions {
        match action {
            Action::Run(action) => {
                execute_run_action(action, &command_topic, &producer).await?;
            }
            Action::WaitUntil(action) => {
                execute_wait_until(action, states.clone())?;
            }
        }
    }

    Ok(())
}

async fn start_event_monitor(
    brokers: String,
    group_id: String,
    topic: String,
    states: Arc<DashMap<String, String>>,
) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &group_id)
        .set("bootstrap.servers", &brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .context("Failed to create Kafka consumer.")?;

    // Restore previous topic/partition offset.
    let mut tpl = TopicPartitionList::new();
    tpl.add_partition(&topic, 0);

    let committed_offsets = consumer.committed_offsets(tpl.clone(), Timeout::Never)?;
    let committed_offsets = committed_offsets.to_topic_map();
    if let Some(offset) = committed_offsets.get(&(topic.clone(), 0)) {
        match offset {
            Offset::Invalid => tpl.set_partition_offset(&topic, 0, Offset::Beginning)?,
            offset => tpl.set_partition_offset(&topic, 0, offset.clone())?,
        };
    }

    info!("Restoring commited offsets: {:?}", &tpl);
    consumer
        .assign(&tpl)
        .context("Failed to manually assign topic, partition, and/or offset to consumer.")?;

    consumer
        .start()
        .try_for_each(|borrowed_message| {
            let owned_message = borrowed_message.detach();
            let owned_states = states.clone();

            async move {
                if let Some(payload) = owned_message.payload() {
                    // Decode payload into a Event message.
                    let event = Event::decode(payload).unwrap();
                    let kind = EventKind::from_i32(event.kind).unwrap();

                    dbg!(&event);

                    let event_id: Vec<_> = event.identifier.split('-').collect();
                    let command_id = event_id.first().unwrap().to_string();

                    match kind {
                        EventKind::Created => {
                            owned_states.insert(command_id, String::from("created"));
                        }
                        EventKind::Started => {
                            owned_states.insert(command_id, String::from("started"));
                        }
                        EventKind::Stopped => {
                            owned_states.insert(command_id, String::from("stopped"));
                        }
                        EventKind::Unknown => {
                            owned_states.insert(command_id, String::from("unkown"));
                        }
                    }
                }

                Ok(())
            }
        })
        .await?;

    Ok(())
}

///
///
///
async fn execute_run_action(
    action: RunAction,
    command_topic: &String,
    producer: &FutureProducer,
) -> Result<()> {
    let command = Command::new(
        CommandKind::Create,
        Some(action.identifier.clone()),
        Some(action.application.clone()),
        Some(action.location.clone()),
        Some(action.image.clone()),
        action.command.clone(),
        None,
    );

    let mut payload = BytesMut::with_capacity(64);
    command.encode(&mut payload)?;

    let message = FutureRecord::to(&command_topic)
        .key(&action.identifier)
        .payload(payload.to_bytes());

    dbg!(&message);

    if let Err(_) = producer.send(message, Timeout::After(Duration::from_secs(5))).await {
        bail!("Failed to send command to '{}' topic.", command_topic);
    }

    Ok(())
}

///
///
///
fn execute_wait_until(
    action: WaitUntilAction,
    states: Arc<DashMap<String, String>>,
) -> Result<()> {
    loop {
        if let Some(state) = states.get(&action.identifier) {
            if state.value() == &action.state {
                return Ok(());
            }
        };

        thread::sleep(Duration::from_secs(1));
    }
}
