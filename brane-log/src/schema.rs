use crate::Context;
use async_stream::stream;
use futures::Stream;
use juniper::{EmptyMutation, FieldError, GraphQLObject, RootNode};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, Subscription>;

impl juniper::Context for Context {}

#[derive(Clone, Debug, Deserialize, GraphQLObject, Default, Serialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, GraphQLObject, Default)]
pub struct Event {
    pub application: String,
    pub job: String,
    pub location: String,
    pub category: String,
    pub order: i32,
    pub kind: String,
    pub timestamp: String,
    pub information: Vec<KeyValuePair>,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    ///
    ///
    ///
    async fn applications(_context: &Context) -> Vec<String> {
        // let cassandra = context.cassandra.read().unwrap();

        // let query = stmt!("SELECT DISTINCT application_id FROM application_event.events;");
        // let result = cassandra.execute(&query).wait().unwrap();

        // let as_string = |r: Row| r.get_by_name("application_id").unwrap();

        // result.iter().map(as_string).collect()

        todo!()
    }

    ///
    ///
    ///
    async fn events(
        _application: String,
        _job: Option<String>,
        _kind: Option<String>,
        _context: &Context,
    ) -> Vec<Event> {
        // let session = context.scylla.read().unwrap();

        // let mut events = session.query("SELECT * FROM application_event.events WHERE application_id = ?;", (application.as_str(),)).await.unwrap();

        // let as_event = |r: Row| {
        //     let application = r.get_by_name("application_id").unwrap();
        //     let job = r.get_by_name("job_id").unwrap();
        //     let location = r.get_by_name("location_id").unwrap();
        //     let category = r.get_by_name("category").unwrap();
        //     let order = r.get_by_name("event_id").unwrap();
        //     let kind = r.get_by_name("kind").unwrap();
        //     let information: String = r.get_by_name("information").unwrap();
        //     let information: Vec<KeyValuePair> = serde_json::from_str(&information).unwrap();
        //     let timestamp = r.get_by_name("timestamp").unwrap();
        //     let timestamp = OffsetDateTime::from_unix_timestamp(timestamp).format(Format::Rfc3339);

        //     Event {
        //         application,
        //         job,
        //         location,
        //         category,
        //         order,
        //         kind,
        //         timestamp,
        //         information,
        //     }
        // };

        // let mut events: Vec<Event> = cassandra.execute(&query).wait().unwrap().iter().map(as_event).collect();

        // if let Some(job) = job {
        //     events = events.iter().filter(|e| e.job == job).map(Event::clone).collect();
        // }

        // if let Some(kind) = kind {
        //     events = events.iter().filter(|e| e.kind == kind).map(Event::clone).collect();
        // }

        // // Lastly, sort by timestamp.
        // events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // events

        todo!()
    }
}

pub struct Subscription;

type EventStream = Pin<Box<dyn Stream<Item = Result<Event, FieldError>> + Send>>;

#[graphql_subscription(context = Context)]
impl Subscription {
    async fn events(
        application: String,
        job: Option<String>,
        kind: Option<String>,
        context: &Context,
    ) -> EventStream {
        let mut events_rx = context.events_rx.clone();

        let stream = stream! {
            while events_rx.changed().await.is_ok() {
                let event = events_rx.borrow().clone();

                if event.application != application {
                    continue;
                }
                if let Some(ref job) = job {
                    if &event.job != job {
                        continue;
                    }
                }
                if let Some(ref kind) = kind {
                    if &event.kind != kind {
                        continue;
                    }
                }

                yield Ok(event)
            }
        };

        Box::pin(stream)
    }
}
