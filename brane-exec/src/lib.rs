#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod delegate;
pub mod docker;
pub mod openapi;

///
///
///
#[derive(Deserialize, Serialize)]
pub struct ExecuteInfo {
    pub command: Option<Vec<String>>,
    pub image: String,
    pub image_file: Option<PathBuf>,
    pub mounts: Option<Vec<String>>,
    pub working_dir: Option<String>,
}

impl ExecuteInfo {
    ///
    ///
    ///
    pub fn new(
        image: String,
        image_file: Option<PathBuf>,
        mounts: Option<Vec<String>>,
        working_dir: Option<String>,
        command: Option<Vec<String>>,
    ) -> Self {
        ExecuteInfo {
            command,
            image,
            image_file,
            mounts,
            working_dir,
        }
    }
}
