#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod docker;
pub mod kubernetes;
pub mod schedule;

///
///
///
#[derive(Deserialize, Serialize)]
pub struct ExecuteInfo {
    pub command: Option<Vec<String>>,
    pub image: String,
    pub image_file: Option<PathBuf>,
    pub mounts: Option<Vec<String>>,
}

impl ExecuteInfo {
    ///
    ///
    ///
    pub fn new(
        image: String,
        image_file: Option<PathBuf>,
        mounts: Option<Vec<String>>,
        command: Option<Vec<String>>,
    ) -> Self {
        ExecuteInfo {
            command,
            image,
            image_file,
            mounts,
        }
    }
}
