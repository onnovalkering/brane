use anyhow::{Context, Result};
use brane_clb::{callback::CallbackHandler, grpc::CallbackServiceServer};
use clap::Clap;
use dotenv::dotenv;
use log::LevelFilter;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    error::RDKafkaErrorCode,
    producer::FutureProducer,
    ClientConfig,
};
use tonic::transport::Server;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(short, long, default_value = "0.0.0.0:50052", env = "ADDRESS")]
    /// Service address
    address: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "127.0.0.1:9092", env = "BROKERS")]
    brokers: String,
    /// Topic to send callbacks to
    #[clap(short, long = "clb-topic", default_value = "clb", env = "CALLBACK_TOPIC")]
    callback_topic: String,
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

    // Ensure that the callback topic (output) exists.
    let callback_topic = opts.callback_topic.clone();
    ensure_callback_topic(&callback_topic, &opts.brokers).await?;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &opts.brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    let handler = CallbackHandler {
        callback_topic,
        producer,
    };

    // Start gRPC server with callback service.
    Server::builder()
        .add_service(CallbackServiceServer::new(handler))
        .serve(opts.address.parse()?)
        .await
        .context("Failed to start callback gRPC server.")
}

///
///
///
pub async fn ensure_callback_topic(
    callback_topic: &str,
    brokers: &str,
) -> Result<()> {
    let admin_client: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka admin client.")?;

    let results = admin_client
        .create_topics(
            &[NewTopic::new(callback_topic, 1, TopicReplication::Fixed(1))],
            &AdminOptions::new(),
        )
        .await?;

    // Report on the results. Don't consider 'TopicAlreadyExists' an error.
    for result in results {
        match result {
            Ok(topic) => log::info!("Kafka topic '{}' created.", topic),
            Err((topic, error)) => match error {
                RDKafkaErrorCode::TopicAlreadyExists => {
                    log::info!("Kafka topic '{}' already exists", topic);
                }
                _ => {
                    anyhow::bail!("Kafka topic '{}' not created: {:?}", topic, error);
                }
            },
        }
    }

    Ok(())
}
