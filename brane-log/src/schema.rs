use crate::Context;
use juniper::{EmptyMutation, EmptySubscription, GraphQLObject, RootNode};

impl juniper::Context for Context {}

#[derive(GraphQLObject)]
pub struct Event {
    identifier: String,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn events(context: &Context) -> Vec<Event> {
        vec![Event {
            identifier: context.name.clone(),
        }]
    }
}

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;
