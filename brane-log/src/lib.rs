#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate cassandra_cpp;
#[macro_use]
extern crate log;
#[macro_use]
extern crate juniper;

pub mod ingestion;
pub mod schema;

pub struct Context {
    pub name: String,
}

pub use schema::Schema;
