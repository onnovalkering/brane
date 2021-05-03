use anyhow::{bail, Context, Result};
use brane_drv::grpc::DriverServiceServer;
use brane_drv::handler::DriverHandler;
use brane_job::interface::{Event, EventKind};
use brane_bvm::values::Value;
use brane_bvm::VmState;
use clap::Clap;
use dotenv::dotenv;
use log::{info};
use log::LevelFilter;
use prost::Message as _;
use futures::TryStreamExt;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    producer::FutureProducer,
    consumer::{Consumer, StreamConsumer},
    error::RDKafkaErrorCode,
    ClientConfig,
    TopicPartitionList,
    Offset,
    util::Timeout,
    Message as _
};
use tonic::transport::Server;
use specifications::common::Value as SpecValue;
use std::sync::Arc;
use dashmap::DashMap;


#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(short, long, default_value = "http://brane-api:8080/packages", env = "PACKAGE_INDEX_URL")]
    package_index_url: String,
    #[clap(short, long, default_value = "127.0.0.1:50053", env = "ADDRESS")]
    /// Service address
    address: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Topic to send commands to
    #[clap(short, long = "cmd-topic", env = "COMMAND_TOPIC")]
    command_topic: String,
    /// Topic to recieve events from
    #[clap(short, long = "evt-topic", env = "EVENT_TOPIC")]
    event_topic: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-drv")]
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

    // Ensure that the input/output topics exists.
    let command_topic = opts.command_topic.clone();
    ensure_topics(vec![&command_topic, &opts.event_topic], &opts.brokers).await?;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &opts.brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    // Start event monitor in the background.
    let states: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
    let results: Arc<DashMap<String, Value>> = Arc::new(DashMap::new());

    tokio::spawn(start_event_monitor(
        opts.brokers.clone(),
        opts.group_id.clone(),
        opts.event_topic.clone(),
        states.clone(),
        results.clone(),
    ));

    let package_index_url = opts.package_index_url.clone();
    let sessions: Arc<DashMap<String, VmState>> = Arc::new(DashMap::new());
    let handler = DriverHandler {
        package_index_url,
        producer,
        command_topic,
        states,
        results,
        sessions,
    };

    // Start gRPC server with callback service.
    Server::builder()
        .add_service(DriverServiceServer::new(handler))
        .serve(opts.address.parse()?)
        .await
        .context("Failed to start callback gRPC server.")
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

async fn start_event_monitor(
    brokers: String,
    group_id: String,
    topic: String,
    states: Arc<DashMap<String, String>>,
    results: Arc<DashMap<String, Value>>,
) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
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
        .stream()
        .try_for_each(|borrowed_message| {
            let owned_message = borrowed_message.detach();
            let owned_states = states.clone();
            let owned_results = results.clone();

            async move {
                if let Some(payload) = owned_message.payload() {
                    // Decode payload into a Event message.
                    let event = Event::decode(payload).unwrap();
                    let kind = EventKind::from_i32(event.kind).unwrap();

                    dbg!(&event);

                    let event_id: Vec<_> = event.identifier.split('-').collect();
                    let correlation_id = event_id.first().unwrap().to_string();

                    match kind {
                        EventKind::Unknown => {
                            owned_states.insert(correlation_id, String::from("unkown"));
                        }
                        EventKind::Created => {
                            owned_states.insert(correlation_id, String::from("created"));
                        }
                        EventKind::Ready => {
                            owned_states.insert(correlation_id, String::from("created"));
                        }
                        EventKind::Initialized => {
                            owned_states.insert(correlation_id, String::from("initialized"));
                        }
                        EventKind::Started => {
                            owned_states.insert(correlation_id, String::from("started"));
                        }
                        EventKind::Finished => {
                            let payload = String::from_utf8_lossy(&event.payload).to_string();
                            let value: SpecValue = serde_json::from_str(&payload).unwrap();
                            let value = Value::from(value);

                            // Using these two hashmaps is not ideal, they lock and we're dependend on polling (from call future).
                            // NOTE: for now we have to make sure the results are inserted before the state becomes "finished" to prevent race conditions.
                            owned_results.insert(correlation_id.clone(), value);
                            owned_states.insert(correlation_id.clone(), String::from("finished"));
                            dbg!(&owned_results);
                        }
                        EventKind::Stopped => {
                            owned_states.insert(correlation_id, String::from("stopped"));
                        }
                        EventKind::Failed => {
                            owned_states.insert(correlation_id, String::from("failed"));
                        }
                        _ => {
                            unreachable!();
                        }
                    }

                    dbg!(&owned_states);
                }

                Ok(())
            }
        })
        .await?;

    Ok(())
}
