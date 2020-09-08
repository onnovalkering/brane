#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};
use specifications::common::Value;

pub mod callback;
pub mod exec;

type Map<T> = std::collections::HashMap<String, T>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub action: String,
    pub arguments: Map<Value>,
    pub callback_url: Option<String>,
    pub invocation_id: Option<i32>,
}