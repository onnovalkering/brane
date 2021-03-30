use crate::Context;
use cassandra_cpp::{AsRustType, BindRustType, Row};
use juniper::{EmptyMutation, GraphQLObject, RootNode, FieldError};
use time::{Format, OffsetDateTime};
use std::pin::Pin;
use futures::Stream;
use async_stream::stream;

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, Subscription>;

impl juniper::Context for Context {}

#[derive(Clone, Debug, GraphQLObject, Default)]
pub struct Event {
    pub application: String,
    pub job: String,
    pub location: String,
    pub order: i32,
    pub kind: String,
    pub timestamp: String,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    ///
    ///
    ///
    async fn applications(context: &Context) -> Vec<String> {
        let cassandra = context.cassandra.read().unwrap();

        let query = stmt!("SELECT DISTINCT application_id FROM application_event.events;");
        let result = cassandra.execute(&query).wait().unwrap();

        let as_string = |r: Row| r.get_by_name("application_id").unwrap();

        result.iter().map(as_string).collect()
    }

    ///
    ///
    ///
    async fn events(
        application: String,
        job: Option<String>,
        kind: Option<String>,
        context: &Context,
    ) -> Vec<Event> {
        let cassandra = context.cassandra.read().unwrap();

        let mut query = stmt!("SELECT * FROM application_event.events WHERE application_id = ?;");
        query.bind(0, application.as_str()).unwrap();

        let as_event = |r: Row| {
            let application = r.get_by_name("application_id").unwrap();
            let job = r.get_by_name("job_id").unwrap();
            let location = r.get_by_name("location_id").unwrap();
            let order = r.get_by_name("event_id").unwrap();
            let kind = r.get_by_name("kind").unwrap();

            let timestamp = r.get_by_name("timestamp").unwrap();
            let timestamp = OffsetDateTime::from_unix_timestamp(timestamp).format(Format::Rfc3339);

            Event {
                application,
                job,
                location,
                order,
                kind,
                timestamp,
            }
        };

        let mut events: Vec<Event> = cassandra.execute(&query).wait().unwrap().iter().map(as_event).collect();

        if let Some(job) = job {
            events = events.iter().filter(|e| e.job == job).map(Event::clone).collect();
        }

        if let Some(kind) = kind {
            events = events.iter().filter(|e| e.kind == kind).map(Event::clone).collect();
        }

        events
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
        context: &Context
    ) -> EventStream {
        let mut events_rx = context.events_rx.clone();

        let stream = stream! {
            while let Some(event) = events_rx.recv().await {
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
