use std::sync::Arc;

use anyhow::{bail, Context, Result};
use brane_cfg::{Infrastructure, Secrets};
use brane_job::{
    clb_heartbeat, clb_lifecycle,
    interface::{Callback, CallbackKind, Command, CommandKind},
};
use brane_job::{cmd_create, interface::Event};
use brane_shr::utilities;
use bytes::BytesMut;
use clap::Clap;
use dashmap::{lock::RwLock, DashMap};
use dotenv::dotenv;
use futures::stream::FuturesUnordered;
use futures::{StreamExt, TryStreamExt};
use log::LevelFilter;
use log::{debug, error, info, warn};
use prost::Message;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    config::ClientConfig,
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    error::RDKafkaErrorCode,
    message::ToBytes,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
    Message as KafkaMesage, Offset, TopicPartitionList,
};
use tokio::task::JoinHandle;
use xenon::compute::Scheduler;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Topic to receive callbacks from
    #[clap(short, long = "clb-topic", default_value = "clb", env = "CALLBACK_TOPIC")]
    callback_topic: String,
    /// Topic to receive commands from
    #[clap(short = 'o', long = "cmd-topic", default_value = "plr-cmd", env = "COMMAND_TOPIC")]
    command_topic: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "127.0.0.1:9092", env = "BROKERS")]
    brokers: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Topic to send events to
    #[clap(short, long = "evt-topic", default_value = "job-evt", env = "EVENT_TOPIC")]
    event_topic: String,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-job", env = "GROUP_ID")]
    group_id: String,
    /// Infra metadata store
    #[clap(short, long, default_value = "./infra.yml", env = "INFRA")]
    infra: String,
    /// Number of workers
    #[clap(short = 'w', long, default_value = "1", env = "NUM_WORKERS")]
    num_workers: u8,
    /// Secrets store
    #[clap(short, long, default_value = "./secrets.yml", env = "SECRETS")]
    secrets: String,
    /// Xenon gRPC endpoint
    #[clap(short, long, default_value = "http://127.0.0.1:50051", env = "XENON")]
    xenon: String,
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

    // Ensure that the input/output topics exists.
    ensure_topics(
        vec![&opts.callback_topic, &opts.command_topic, &opts.event_topic],
        &opts.brokers,
    )
    .await?;

    let infra = Infrastructure::new(opts.infra.clone())?;
    infra.validate()?;

    let secrets = Secrets::new(opts.secrets.clone())?;
    secrets.validate()?;

    let xenon_schedulers = Arc::new(DashMap::<String, Arc<RwLock<Scheduler>>>::new());
    let xenon_endpoint = utilities::ensure_http_schema(&opts.xenon, !opts.debug)?;

    // Spawn workers, using Tokio tasks and thread pool.
    let workers = (0..opts.num_workers)
        .map(|i| {
            let handle = tokio::spawn(start_worker(
                opts.brokers.clone(),
                opts.group_id.clone(),
                opts.callback_topic.clone(),
                opts.command_topic.clone(),
                opts.event_topic.clone(),
                infra.clone(),
                secrets.clone(),
                xenon_endpoint.clone(),
                xenon_schedulers.clone(),
            ));

            info!("Spawned asynchronous worker #{}.", i + 1);
            handle
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

    Ok(())
}

///
///
///
async fn ensure_topics(
    topics: Vec<&str>,
    brokers: &str,
) -> Result<()> {
    let admin_client: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .context("Failed to create Kafka admin client.")?;

    let topics: Vec<NewTopic> = topics
        .iter()
        .map(|t| NewTopic::new(t, 1, TopicReplication::Fixed(1)))
        .collect();

    let results = admin_client.create_topics(topics.iter(), &AdminOptions::new()).await?;

    // Report on the results. Don't consider 'TopicAlreadyExists' an error.
    for result in results {
        match result {
            Ok(topic) => info!("Kafka topic '{}' created.", topic),
            Err((topic, error)) => match error {
                RDKafkaErrorCode::TopicAlreadyExists => {
                    info!("Kafka topic '{}' already exists", topic);
                }
                _ => {
                    bail!("Kafka topic '{}' not created: {:?}", topic, error);
                }
            },
        }
    }

    Ok(())
}

///
///
///
#[allow(clippy::too_many_arguments)]
async fn start_worker(
    brokers: String,
    group_id: String,
    clb_topic: String,
    cmd_topic: String,
    evt_topic: String,
    infra: Infrastructure,
    secrets: Secrets,
    xenon_endpoint: String,
    xenon_schedulers: Arc<DashMap<String, Arc<RwLock<Scheduler>>>>,
) -> Result<()> {
    let output_topic = evt_topic.as_ref();

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &group_id)
        .set("bootstrap.servers", &brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .context("Failed to create Kafka consumer.")?;

    // TODO: make use of transactions / exactly-once semantics (EOS)

    // Restore previous topic/partition offset.
    let mut tpl = TopicPartitionList::new();
    tpl.add_partition(&clb_topic, 0);
    tpl.add_partition(&cmd_topic, 0);

    let committed_offsets = consumer.committed_offsets(tpl.clone(), Timeout::Never)?;
    let committed_offsets = committed_offsets.to_topic_map();
    if let Some(offset) = committed_offsets.get(&(clb_topic.clone(), 0)) {
        match offset {
            Offset::Invalid => tpl.set_partition_offset(&clb_topic, 0, Offset::Beginning)?,
            offset => tpl.set_partition_offset(&clb_topic, 0, *offset)?,
        };
    }
    if let Some(offset) = committed_offsets.get(&(cmd_topic.clone(), 0)) {
        match offset {
            Offset::Invalid => tpl.set_partition_offset(&cmd_topic, 0, Offset::Beginning)?,
            offset => tpl.set_partition_offset(&cmd_topic, 0, *offset)?,
        };
    }

    info!("Restoring commited offsets: {:?}", &tpl);
    consumer
        .assign(&tpl)
        .context("Failed to manually assign topic, partition, and/or offset to consumer.")?;

    // Create the outer pipeline on the message stream.
    let stream_processor = consumer.stream().try_for_each(|borrowed_message| {
        consumer.commit_message(&borrowed_message, CommitMode::Sync).unwrap();

        let owned_message = borrowed_message.detach();
        let owned_producer = producer.clone();
        let owned_infra = infra.clone();
        let owned_secrets = secrets.clone();
        let owned_xenon_endpoint = xenon_endpoint.clone();
        let owned_xenon_schedulers = xenon_schedulers.clone();
        let clb_topic = clb_topic.clone();
        let cmd_topic = cmd_topic.clone();

        async move {
            let msg_key = owned_message
                .key()
                .map(String::from_utf8_lossy)
                .map(String::from)
                .unwrap_or_default();

            if msg_key.is_empty() {
                warn!("Received message without a key. Ignoring it.");
                return Ok(());
            }

            let msg_payload = owned_message.payload().unwrap_or_default();
            if msg_payload.is_empty() {
                warn!("Received message without a payload (key: {}). Ignoring it.", msg_key);
                return Ok(());
            }

            let topic = owned_message.topic();
            let events = if topic == clb_topic {
                handle_clb_message(msg_key, msg_payload)
            } else if topic == cmd_topic {
                handle_cmd_message(
                    msg_key,
                    msg_payload,
                    owned_infra,
                    owned_secrets,
                    owned_xenon_endpoint,
                    owned_xenon_schedulers,
                )
                .await
            } else {
                unreachable!()
            };

            match events {
                Ok(events) => {
                    for (evt_key, event) in events {
                        // Encode event message into a payload (bytes)
                        let mut payload = BytesMut::with_capacity(64);
                        event.encode(&mut payload).unwrap();

                        // Send event on output topic
                        let message = FutureRecord::to(output_topic).key(&evt_key).payload(payload.to_bytes());

                        if let Err(error) = owned_producer.send(message, Timeout::Never).await {
                            error!("Failed to send event (key: {}): {:?}", evt_key, error);
                        }
                    }
                }
                Err(error) => error!("{:?}", error),
            };

            Ok(())
        }
    });

    stream_processor
        .await
        .context("Stream processor did not run until completion.")
}

///
///
///
fn handle_clb_message(
    key: String,
    payload: &[u8],
) -> Result<Vec<(String, Event)>> {
    // Decode payload into a callback message.
    let callback = Callback::decode(payload).unwrap();
    let kind = CallbackKind::from_i32(callback.kind).unwrap();

    // Ignore unkown callbacks, as we can't dispatch it.
    if kind == CallbackKind::Unknown {
        warn!("Received UNKOWN command (key: {}). Ignoring it.", key);
        return Ok(vec![]);
    }

    info!("Received {} callback (key: {}).", kind, key);
    debug!("{:?}", callback);

    match kind {
        CallbackKind::Heartbeat => clb_heartbeat::handle(callback),
        _ => clb_lifecycle::handle(callback),
    }
}

///
///
///
async fn handle_cmd_message(
    key: String,
    payload: &[u8],
    infra: Infrastructure,
    secrets: Secrets,
    xenon_endpoint: String,
    xenon_schedulers: Arc<DashMap<String, Arc<RwLock<Scheduler>>>>,
) -> Result<Vec<(String, Event)>> {
    // Decode payload into a command message.
    let command = Command::decode(payload).unwrap();
    let kind = CommandKind::from_i32(command.kind).unwrap();

    // Ignore unkown commands, as we can't dispatch it.
    if kind == CommandKind::Unknown {
        warn!("Received UNKOWN command (key: {}). Ignoring it.", key);
        return Ok(vec![]);
    }

    info!("Received {} command (key: {}).", kind, key);
    debug!("{:?}", command);

    // Dispatch command message to appropriate handlers.
    match kind {
        CommandKind::Create => {
            cmd_create::handle(&key, command, infra, secrets, xenon_endpoint, xenon_schedulers).await
        }
        CommandKind::Stop => unimplemented!(),
        CommandKind::Unknown => unreachable!(),
    }
}
