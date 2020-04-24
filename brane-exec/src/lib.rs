#[macro_use]
extern crate log;

pub mod kubernetes;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

type Map<T> = std::collections::HashMap<String, T>;

#[derive(Deserialize)]
pub struct ExecuteRequest {
    #[serde(default = "generate_identifier")]
    pub identifier: String,
    pub image: String,
    #[serde(default = "Map::<String>::new")]
    pub options: Map<String>,
    pub payload: String,
    pub target: String,
}

fn generate_identifier() -> String {
    Uuid::new_v4().to_string().chars().take(8).collect()
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ExecuteResponse {
    ExecuteError { message: String },
    ExecuteToken { identifier: String },
}
