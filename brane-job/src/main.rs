use anyhow::{bail, Context, Result};
use brane_cfg::{Infrastructure, Secrets};
use brane_job::cmd_create;
use brane_job::interface::{Command, CommandKind};
use bytes::BytesMut;
use clap::Clap;
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
    message::ToBytes,
    producer::{FutureProducer, FutureRecord},
    types::RDKafkaErrorCode,
    util::Timeout,
    Message as KafkaMesage, TopicPartitionList,
};
use tokio::task::JoinHandle;

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
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Topic to send events to
    #[clap(short, long = "evt-topic", env = "EVENT_TOPIC")]
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

    let infra = Infrastructure::new(opts.infra.clone())?;
    let secrets = Secrets::new(opts.secrets.clone())?;

    // Ensure that the input (commands) topic exists.
    ensure_input_topic(&opts.command_topic, &opts.brokers).await?;

    // Spawn workers, using Tokio tasks and thread pool.
    let workers = (0..opts.num_workers)
        .map(|i| {
            let handle = tokio::spawn(start_worker(
                opts.brokers.clone(),
                opts.group_id.clone(),
                opts.command_topic.clone(),
                opts.event_topic.clone(),
                infra.clone(),
                secrets.clone(),
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
async fn ensure_input_topic(
    input_topic: &str,
    brokers: &str,
) -> Result<()> {
    let admin_client: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .context("Failed to create Kafka admin client.")?;

    let results = admin_client
        .create_topics(
            &[NewTopic::new(input_topic, 1, TopicReplication::Fixed(1))],
            &AdminOptions::new(),
        )
        .await?;

    // Report on the results. Don't consider 'TopicAlreadyExists' an error.
    for result in results {
        match result {
            Ok(topic) => info!("Input topic '{}' created.", topic),
            Err((topic, error)) => match error {
                RDKafkaErrorCode::TopicAlreadyExists => {
                    info!("Input topic '{}' already exists", topic);
                }
                _ => {
                    bail!("Input topic '{}' not created: {:?}", topic, error);
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
    input_topic: String,
    output_topic: String,
    infra: Infrastructure,
    secrets: Secrets,
) -> Result<()> {
    let output_topic = output_topic.as_ref();

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
    tpl.add_partition(&input_topic, 0);

    let committed_offsets = consumer.committed_offsets(tpl, Timeout::Never)?;
    debug!("Restoring commited offsets: {:?}", &committed_offsets);
    consumer
        .assign(&committed_offsets)
        .context("Failed to manually assign topic, partition, and/or offset to consumer.")?;

    // Create the outer pipeline on the message stream.
    let stream_processor = consumer.stream().try_for_each(|borrowed_message| {
        &consumer.commit_message(&borrowed_message, CommitMode::Sync).unwrap();

        let owned_message = borrowed_message.detach();
        let owned_producer = producer.clone();
        let owned_infra = infra.clone();
        let owned_secrets = secrets.clone();

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
                    CommandKind::Create => cmd_create::handle(cmd_key, command, owned_infra, owned_secrets),
                    CommandKind::Cancel => unimplemented!(),
                    CommandKind::Check => unimplemented!(),
                    CommandKind::Unknown => unreachable!(),
                };

                match events {
                    Ok(events) => {
                        for (evt_key, event) in events {
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

    stream_processor
        .await
        .context("Stream processor did not run until completion.")
}
