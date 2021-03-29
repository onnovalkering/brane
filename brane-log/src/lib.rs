#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate cassandra_cpp;
#[macro_use]
extern crate log;
#[macro_use]
extern crate juniper;

use cassandra_cpp::Session;
use std::sync::Arc;

pub mod ingestion;
pub mod schema;

pub struct Context {
    pub cassandra: Arc<Session>,
}

pub use schema::Schema;
