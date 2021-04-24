use anyhow::{bail, Context, Result};
use brane_drv::grpc::DriverServiceServer;
use brane_drv::handler::DriverHandler;
use clap::Clap;
use dotenv::dotenv;
use log::{info};
use log::LevelFilter;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    producer::FutureProducer,
    types::RDKafkaError,
    ClientConfig,
};
use tonic::transport::Server;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
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
    ensure_topics(vec![&opts.command_topic, &opts.event_topic], &opts.brokers).await?;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &opts.brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    let handler = DriverHandler { producer };

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
                RDKafkaError::TopicAlreadyExists => {
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
