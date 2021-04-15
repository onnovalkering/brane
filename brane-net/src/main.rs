use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Clap;
use dotenv::dotenv;
use log::LevelFilter;
use socksx::Socks6Handler;
use tokio::net::TcpListener;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    producer::FutureProducer,
    error::RDKafkaErrorCode,
    ClientConfig,
};

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(short, long, default_value = "127.0.0.1:5080", env = "ADDRESS")]
    /// Service address
    address: String,
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Topic to send callbacks to
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

    // Ensure that the callback topic (output) exists.
    let callback_topic = opts.event_topic.clone();
    ensure_event_topic(&callback_topic, &opts.brokers).await?;

    let _producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &opts.brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    let listener = TcpListener::bind(opts.address).await?;
    let handler = Arc::new(Socks6Handler::new());

    loop {
        let (mut incoming, _) = listener.accept().await?;
        let handler = Arc::clone(&handler);

        dbg!(&incoming);

        tokio::spawn(async move { handler.handle_request(&mut incoming).await });
    }
}


///
///
///
pub async fn ensure_event_topic(
    event_topic: &str,
    brokers: &str,
) -> Result<()> {
    let admin_client: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka admin client.")?;

    let results = admin_client
        .create_topics(
            &[NewTopic::new(event_topic, 1, TopicReplication::Fixed(1))],
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
