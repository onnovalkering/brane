#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};
use specifications::common::Value;

type Map<T> = std::collections::HashMap<String, T>;

pub mod exec;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub action: String,
    pub arguments: Map<Value>,
    pub callback_url: Option<String>,
    pub identifier: String,
    pub monitor_url: Option<String>,
}
