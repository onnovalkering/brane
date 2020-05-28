#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};
use serde_json::Value as JValue;
use std::path::PathBuf;

pub mod docker;

///
///
///
#[derive(Deserialize, Serialize)]
pub struct ExecuteInfo {
    pub image: String,
    pub image_file: Option<PathBuf>,
    pub payload: JValue,
}

impl ExecuteInfo {
    ///
    ///
    ///
    pub fn new(
        image: String,
        image_file: Option<PathBuf>,
        payload: JValue,
    ) -> Self {
        ExecuteInfo {
            image,
            image_file,
            payload,
        }
    }
}
