use chrono::{DateTime, Utc};
use juniper::{EmptyMutation, EmptySubscription, GraphQLObject, RootNode};
use uuid::Uuid;
use crate::Context;

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;
// pub type Stream<T> = std::pin::Pin<Box<dyn futures::Stream<Item = Result<T, juniper::FieldError>> + Send>>;

impl juniper::Context for Context {}

#[derive(Clone, Debug, GraphQLObject)]
pub struct Package {
    pub created: DateTime<Utc>,
    pub description: Option<String>,
    pub detached: bool,
    pub owners: Vec<String>,
    pub id: Uuid,
    pub kind: String,
    pub name: String,
    pub version: String,
    pub functions_as_json: Option<String>,
    pub types_as_json: Option<String>,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    ///
    ///
    ///
    async fn apiVersion() -> &str {
        env!("CARGO_PKG_VERSION")
    }

    ///
    ///
    ///
    async fn packages(
        name: Option<String>,
        version: Option<String>,
        _term: Option<String>,
        _context: &Context,
    ) -> Vec<Package> {
        vec![Package {
            created: Utc::now(),
            description: Some(String::new()),
            detached: false,
            owners: vec![],
            id: Uuid::new_v4(),
            kind: String::new(),
            name: name.unwrap_or_default(),
            version: version.unwrap_or_default(),
            functions_as_json: None,
            types_as_json: None,
        }]
    }
}
