#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate cassandra_cpp;
#[macro_use]
extern crate log;
#[macro_use]
extern crate juniper;

use cassandra_cpp::Session;
use tokio::sync::watch::Receiver;
use std::sync::{Arc, RwLock};

pub mod ingestion;
pub mod schema;

pub struct Context {
    pub cassandra: Arc<RwLock<Session>>,
    pub events_rx: Receiver<schema::Event>,
}

pub use schema::Schema;
