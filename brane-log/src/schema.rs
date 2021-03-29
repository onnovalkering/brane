use crate::Context;
use cassandra_cpp::{AsRustType, BindRustType, Row};
use juniper::{EmptyMutation, EmptySubscription, GraphQLObject, RootNode};
use time::{Format, OffsetDateTime};

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

impl juniper::Context for Context {}

#[derive(Clone, GraphQLObject)]
pub struct Event {
    job: String,
    location: String,
    order: i32,
    kind: String,
    timestamp: String,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    ///
    ///
    ///
    async fn applications(context: &Context) -> Vec<String> {
        let cassandra = context.cassandra.clone();

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
        let cassandra = context.cassandra.clone();

        let mut query = stmt!("SELECT * FROM application_event.events WHERE application_id = ?;");
        query.bind(0, application.as_str()).unwrap();

        let as_event = |r: Row| {
            let job = r.get_by_name("job_id").unwrap();
            let location = r.get_by_name("location_id").unwrap();
            let order = r.get_by_name("event_id").unwrap();
            let kind = r.get_by_name("kind").unwrap();

            let timestamp = r.get_by_name("timestamp").unwrap();
            let timestamp = OffsetDateTime::from_unix_timestamp(timestamp).format(Format::Rfc3339);

            Event {
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
