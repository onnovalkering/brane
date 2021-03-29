use anyhow::{Context as AContext, Result};
use brane_job::interface::{Event, EventKind};
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
use std::sync::Arc;

///
///
///
pub async fn ensure_db_keyspace(cassandra: &Arc<Session>) -> Result<()> {
    let query = stmt!(
        r#"
        CREATE KEYSPACE IF NOT EXISTS application_event
            WITH replication = {'class': 'SimpleStrategy', 'replication_factor' : 3};
    "#
    );

    cassandra
        .execute(&query)
        .await
        .map(|_| ())
        .map_err(|e| anyhow!("{:?}", e))
        .context("Failed to create 'application_event' keyspace.")
}

///
///
///
pub async fn ensure_db_tables(cassandra: &Arc<Session>) -> Result<()> {
    let query = stmt!(
        r#"
        CREATE TABLE IF NOT EXISTS application_event.events (
            application_id text,
            job_id text,
            location_id text,
            event_id int,
            kind text,
            timestamp timestamp,
            PRIMARY KEY ((application_id), job_id, location_id, event_id)
        )
    "#
    );

    cassandra
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
    event_topic: String,
    cassandra: Arc<Session>,
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
    tpl.add_partition(&event_topic, 0);

    let committed_offsets = consumer.committed_offsets(tpl.clone(), Timeout::Never)?;
    let committed_offsets = committed_offsets.to_topic_map();
    if let Some(offset) = committed_offsets.get(&(event_topic.clone(), 0)) {
        match offset {
            Offset::Invalid => tpl.set_partition_offset(&event_topic, 0, Offset::Beginning)?,
            offset => tpl.set_partition_offset(&event_topic, 0, offset.clone())?,
        };
    }

    info!("Restoring commited offsets: {:?}", &tpl);
    consumer
        .assign(&tpl)
        .context("Failed to manually assign topic, partition, and/or offset to consumer.")?;

    let mut message_stream = consumer.stream();

    // Process incoming event messages.
    while let Some(message) = message_stream.next().await {
        match message {
            Ok(borrowed_message) => {
                if let Err(error) = process_message(borrowed_message.detach(), &cassandra) {
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
    cassandra: &Session,
) -> Result<()> {
    let payload = match message.payload() {
        Some(payload) => payload,
        None => bail!("Received Kafka message without a payload."),
    };

    // Decode payload into a Event message.
    let event = Event::decode(payload).unwrap();
    let kind = EventKind::from_i32(event.kind).unwrap();
    let kind_txt = format!("{:?}", kind).to_lowercase();

    dbg!(&event);

    // Insert event
    let mut insert = stmt!(
        r#"
        INSERT INTO application_event.events (application_id, job_id, location_id, event_id, kind, timestamp)
        VALUES (?, ?, ?, ?, ?, ?)
    "#
    );

    insert.bind_string(0, event.application.as_str()).unwrap();
    insert.bind_string(1, event.identifier.as_str()).unwrap();
    insert.bind_string(2, event.location.as_str()).unwrap();
    insert.bind_int32(3, 1).unwrap();
    insert.bind_string(4, kind_txt.as_str()).unwrap();
    insert.bind_int64(5, event.timestamp).unwrap();

    cassandra
        .execute(&insert)
        .wait() // TODO: use .await, however this won't compile in the current configuration.
        .map(|_| ())
        .map_err(|e| anyhow!("{:?}", e))
        .with_context(|| format!("Failed to insert event: {:?}", event))
        .unwrap();

    Ok(())
}
