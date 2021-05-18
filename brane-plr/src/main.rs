use anyhow::{bail, Context, Result};
use brane_cfg::infrastructure::Location;
use brane_cfg::Infrastructure;
use brane_job::interface::{Command, CommandKind};
use bytes::{Bytes, BytesMut};
use clap::Clap;
use dotenv::dotenv;
use futures::stream::FuturesUnordered;
use futures::{StreamExt, TryStreamExt};
use log::LevelFilter;
use log::{error, info, warn};
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

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Topic to receive commands from
    #[clap(short = 'o', long = "cmd-from-topic", env = "COMMAND_FROM_TOPIC")]
    command_from_topic: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Topic to send commands to
    #[clap(short, long = "cmd-to-topic", env = "COMMAND_TO_TOPIC")]
    command_to_topic: String,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-job", env = "GROUP_ID")]
    group_id: String,
    /// Infra metadata store
    #[clap(short, long, default_value = "./infra.yml", env = "INFRA")]
    infra: String,
    /// Number of workers
    #[clap(short = 'w', long, default_value = "1", env = "NUM_WORKERS")]
    num_workers: u8,
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
    ensure_topics(vec![&opts.command_from_topic, &opts.command_to_topic], &opts.brokers).await?;

    let infra = Infrastructure::new(opts.infra.clone())?;
    infra.validate()?;

    // Spawn workers, using Tokio tasks and thread pool.
    let workers = (0..opts.num_workers)
        .map(|i| {
            let handle = tokio::spawn(start_worker(
                opts.brokers.clone(),
                opts.group_id.clone(),
                opts.command_from_topic.clone(),
                opts.command_to_topic.clone(),
                infra.clone(),
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
async fn start_worker(
    brokers: String,
    group_id: String,
    cmd_from_topic: String,
    cmd_to_topic: String,
    infra: Infrastructure,
) -> Result<()> {
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

    // Restore previous topic/partition offset.
    let mut tpl = TopicPartitionList::new();
    tpl.add_partition(&cmd_from_topic, 0);

    let committed_offsets = consumer.committed_offsets(tpl.clone(), Timeout::Never)?;
    let committed_offsets = committed_offsets.to_topic_map();
    if let Some(offset) = committed_offsets.get(&(cmd_from_topic.clone(), 0)) {
        match offset {
            Offset::Invalid => tpl.set_partition_offset(&cmd_from_topic, 0, Offset::Beginning)?,
            offset => tpl.set_partition_offset(&cmd_from_topic, 0, offset.clone())?,
        };
    }

    info!("Restoring commited offsets: {:?}", &tpl);
    consumer
        .assign(&tpl)
        .context("Failed to manually assign topic, partition, and/or offset to consumer.")?;

    // Create the outer pipeline on the message stream.
    let stream_processor = consumer.stream().try_for_each(|borrowed_message| {
        &consumer.commit_message(&borrowed_message, CommitMode::Sync).unwrap();

        // Shadow with owned clones.
        let owned_message = borrowed_message.detach();
        let producer = producer.clone();
        let infra = infra.clone();
        let cmd_to_topic = cmd_to_topic.clone();

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

            let processing = process_cmd_message(msg_payload, infra).await;
            match processing {
                Ok(payload) => {
                    // Send event on output topic
                    let message = FutureRecord::to(&cmd_to_topic)
                        .key(&msg_key)
                        .payload(payload.to_bytes());

                    if let Err(error) = producer.send(message, Timeout::Never).await {
                        error!("Failed to send command (key: {}): {:?}", msg_key, error);
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
async fn process_cmd_message(
    payload: &[u8],
    infra: Infrastructure,
) -> Result<Bytes> {
    use rand::seq::SliceRandom;

    // Decode payload into a command message.
    let mut command = Command::decode(payload).unwrap();
    let kind = CommandKind::from_i32(command.kind).unwrap();

    // Returns an empty string if location is None.
    if command.location() == "" {
        if kind == CommandKind::Create {
            let locations = infra.get_locations()?;

            // Choose a random location
            let location = locations.choose(&mut rand::thread_rng());
            let location = location.unwrap().clone();

            info!("Assigned command '{}' to location '{}'.", command.identifier(), location);

            let metadata = infra.get_location_metadata(&location)?;
            command.location = Some(location);

            match metadata {
                Location::Kube { registry, .. } | Location::Slurm { registry, .. } | Location::Vm { registry, .. } | Location::Local { registry, ..} => {
                    let image = command.image.unwrap();
                    command.image = Some(format!("{}/library/{}", registry, image));
                }
            }
        }
    }

    // Encode command message into a payload.
    let mut payload = BytesMut::with_capacity(64);
    command.encode(&mut payload).unwrap();

    Ok(Bytes::from(payload))
}
