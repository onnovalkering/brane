#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::{r2d2, r2d2::ConnectionManager};
use dotenv::dotenv;
use futures::StreamExt;
use futures::*;
use log::LevelFilter;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use redis::Client;
use std::env;
use structopt::StructOpt;

mod inv_handler;
mod models;
mod schema;

const DEF_DATABASE_URL: &str = "postgres://postgres:postgres@postgres/postgres";
const DEF_KAFKA_BROKERS: &str = "kafka:9092";
const DEF_REDIS_URL: &str = "redis://redis";
const TOPIC_CONTROL: &str = "control";

#[derive(StructOpt)]
#[structopt(name = "brane-loop", about = "The Brane loop service.")]
struct CLI {
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
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
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| String::from(DEF_DATABASE_URL));
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    // Create a redis client
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| String::from(DEF_REDIS_URL));
    let rd = Client::open(redis_url)?;

    // Create Kafka streams
    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| String::from(DEF_KAFKA_BROKERS));
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "brane-loop")
        .set("bootstrap.servers", &kafka_brokers)
        .create()
        .expect("Couldn't create a Kafka consumer.");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Couldn't create a Kafka producer.");

    // Start consuming messages
    consumer.subscribe(&[TOPIC_CONTROL]).expect("Failed to subscribe");
    let mut message_stream = consumer.start();

    while let Some(message) = message_stream.next().await {
        match message {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let key = String::from_utf8(m.key().expect("Message doesn't have a key.").to_vec())?;
                let payload = match m.payload_view::<str>() {
                    None => serde_json::from_str("{}")?,
                    Some(Ok(s)) => serde_json::from_str(s)?,
                    _ => unreachable!(),
                };

                consumer.commit_message(&m, CommitMode::Async)?;

                debug!(
                    "key: '{:?}', payload: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    key,
                    payload,
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );

                let events = match &key[..3] {
                    "inv" => inv_handler::handle(key, payload, &db, &rd).await?,
                    _ => unimplemented!(),
                };

                for event in events {
                    trigger_event(&producer, &event.0, &event.1).await?;
                }
            }
        };
    }

    Ok(())
}

///
///
///
async fn trigger_event(
    producer: &FutureProducer,
    context: &String,
    payload: &String,
) -> Result<()> {
    let message = FutureRecord::to(TOPIC_CONTROL).key(context).payload(payload);

    info!("Going to trigger event within context '{}': {}", context, payload);

    let _ = producer
        .send(message, 0)
        .map(|delivery| match delivery {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(error)) => {
                error!("Unable to trigger event within context '{}':\n{:#?}", context, error);
                Err(())
            }
            Err(error) => {
                error!("Unable to trigger event within context '{}':\n{:#?}", context, error);
                Err(())
            }
        })
        .await;

    Ok(())
}
