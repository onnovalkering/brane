use diesel::pg::PgConnection;
use diesel::{r2d2, r2d2::ConnectionManager};
use dotenv::dotenv;
use futures::StreamExt;
use log::{info, warn};
use log::LevelFilter;
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;
use std::env;
use structopt::StructOpt;

const DEF_DATABASE_URL: &str = "postgres://postgres:postgres@postgres/postgres";
const DEF_KAFKA_BROKERS: &str = "kafka:9092";
const TOPIC_CONTROL: &str = "control";

#[derive(StructOpt)]
#[structopt(name = "brane-loop", about = "The Brane loop service.")]
struct CLI {
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
}

#[tokio::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    let options = CLI::from_args();
    if options.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    // Create a database pool
    let database_url = env::var("DATABASE_URL").unwrap_or(String::from(DEF_DATABASE_URL));
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    // Create Kafka consumer
    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or(String::from(DEF_KAFKA_BROKERS));
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "brane-loop")
        .set("bootstrap.servers", &kafka_brokers)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&vec![TOPIC_CONTROL])
        .expect("Failed to subscribe");

    let mut message_stream = consumer.start();

    while let Some(message) = message_stream.next().await {
        match message {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for i in 0..headers.count() {
                        let header = headers.get(i).unwrap();
                        info!("  Header {:#?}: {:?}", header.0, header.1);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }

    Ok(())
}
