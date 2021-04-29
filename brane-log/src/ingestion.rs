use anyhow::{Context as AContext, Result};
use schema::KeyValuePair;
use time::{Format, OffsetDateTime};
use crate::schema;
use crate::interface::{Event, EventKind};
use cassandra_cpp::Session;
use futures::stream::StreamExt;
use log::info;
use prost::Message;
use rdkafka::{
    config::ClientConfig,
    consumer::{stream_consumer::StreamConsumer, Consumer},
    message::OwnedMessage,
    util::Timeout,
    Message as KafkaMesage, Offset, TopicPartitionList,
};
use tokio::sync::watch::Sender;
use std::sync::{Arc, RwLock};

///
///
///
pub async fn ensure_db_keyspace(cassandra: &Arc<RwLock<Session>>) -> Result<()> {
    let query = stmt!(
        r#"
        CREATE KEYSPACE IF NOT EXISTS application_event
            WITH replication = {'class': 'SimpleStrategy', 'replication_factor' : 3};
    "#
    );

    cassandra
        .read()
        .unwrap()
        .execute(&query)
        .await
        .map(|_| ())
        .map_err(|e| anyhow!("{:?}", e))
        .context("Failed to create 'application_event' keyspace.")
}

///
///
///
pub async fn ensure_db_tables(cassandra: &Arc<RwLock<Session>>) -> Result<()> {
    let query = stmt!(
        r#"
        CREATE TABLE IF NOT EXISTS application_event.events (
            application_id text,
            job_id text,
            location_id text,
            category text,
            event_id int,
            kind text,
            information text,
            timestamp timestamp,
            PRIMARY KEY ((application_id), job_id, location_id, category, event_id)
        )
    "#
    );

    cassandra
        .read()
        .unwrap()
        .execute(&query)
        .await
        .map(|_| ())
        .map_err(|e| anyhow!("{:?}", e))
        .context("Failed to create 'application_event.events' table.")
}

///
///
///
pub async fn start_worker(
    brokers: String,
    group_id: String,
    event_topics: Vec<String>,
    events_tx: Sender<schema::Event>,
    cassandra: Arc<RwLock<Session>>,
) -> Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("group.id", &group_id)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .context("Failed to create Kafka consumer.")?;

    // Restore previous topic/partition offset.
    let mut tpl = TopicPartitionList::new();
    for topic in event_topics.iter() {
        tpl.add_partition(&topic, 0);
    }

    let committed_offsets = consumer.committed_offsets(tpl.clone(), Timeout::Never)?;
    let committed_offsets = committed_offsets.to_topic_map();
    for topic in event_topics.iter() {
        if let Some(offset) = committed_offsets.get(&(topic.clone(), 0)) {
            match offset {
                Offset::Invalid => tpl.set_partition_offset(&topic, 0, Offset::Beginning)?,
                offset => tpl.set_partition_offset(&topic, 0, offset.clone())?,
            };
        }
    }

    info!("Restoring commited offsets: {:?}", &tpl);
    consumer
        .assign(&tpl)
        .context("Failed to manually assign topic, partition, and/or offset to consumer.")?;

    let mut message_stream = consumer.start();

    // Process incoming event messages.
    while let Some(message) = message_stream.next().await {
        match message {
            Ok(borrowed_message) => {
                if let Err(error) = process_message(borrowed_message.detach(), &events_tx, &cassandra) {
                    error!("An error occured while processing a kafka message: {:?}", error);
                }
            }
            Err(error) => error!("An kafka error occured: {:?}", error),
        };
    }

    unreachable!()
}

///
///
///
fn process_message(
    message: OwnedMessage,
    events_tx: &Sender<schema::Event>,
    cassandra: &Arc<RwLock<Session>>,
) -> Result<()> {
    let payload = match message.payload() {
        Some(payload) => payload,
        None => bail!("Received Kafka message without a payload."),
    };

    // Decode payload into a Event message.
    let event = Event::decode(payload).unwrap();
    let kind = EventKind::from_i32(event.kind).unwrap();
    let payload = event.payload.clone();

    dbg!(&event);

    // Additional information, based on kind of event.
    let mut information = vec![];
    match kind {
        EventKind::Created => {
            information.push(KeyValuePair {
                key: String::from("image"),
                value: String::from_utf8(payload)?,
            });
        }
        EventKind::Connected => {
            information.push(KeyValuePair {
                key: String::from("destination"),
                value: String::from_utf8(payload)?,
            });
        }
        EventKind::Disconnected => {
            let (bytes_ab, bytes_ba): (u64, u64) = bincode::deserialize(&payload)?;

            information.push(KeyValuePair {
                key: String::from("bytes_ab"),
                value: bytes_ab.to_string(),
            });
            information.push(KeyValuePair {
                key: String::from("bytes_ba"),
                value: bytes_ba.to_string(),
            });
        }
        _ => {},
    }

    // Prepare for insertion.
    let kind = format!("{:?}", kind).to_lowercase();
    let information_str = serde_json::to_string(&information)?;

    // Insert event
    let mut insert = stmt!(
        r#"
        INSERT INTO application_event.events (application_id, job_id, location_id, category, event_id, kind, information, timestamp)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#
    );

    insert.bind_string(0, event.application.as_str()).unwrap();
    insert.bind_string(1, event.identifier.as_str()).unwrap();
    insert.bind_string(2, event.location.as_str()).unwrap();
    insert.bind_string(3, event.category.as_str()).unwrap();
    insert.bind_int32(4, event.order as i32).unwrap();
    insert.bind_string(5, kind.as_str()).unwrap();
    insert.bind_string(6, information_str.as_str()).unwrap();
    insert.bind_int64(7, event.timestamp).unwrap();

    cassandra
        .read()
        .unwrap()
        .execute(&insert)
        .wait() // TODO: use .await, however this won't compile in the current configuration.
        .map(|_| ())
        .map_err(|e| anyhow!("{:?}", e))
        .with_context(|| format!("Failed to insert event: {:?}", event))
        .unwrap();

    let event = schema::Event {
        application: event.application.clone(),
        job: event.identifier.clone(),
        location: event.location.clone(),
        category: event.category.clone(),
        order: event.order as i32,
        kind,
        information,
        timestamp: OffsetDateTime::from_unix_timestamp(event.timestamp.clone()).format(Format::Rfc3339),
    };

    events_tx.broadcast(event)?;

    Ok(())
}
