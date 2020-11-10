#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod hpc;
pub mod kubernetes;
pub mod local;

use anyhow::Result;
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

pub trait System {
    fn clone(&self) -> Box<dyn System>;

    ///
    ///
    ///
    fn create_dir(
        &self,
        name: &str,
        parent: Option<&Url>,
        temp: bool,
    ) -> Result<Url>;

    ///
    ///
    ///
    fn create_file(
        &self,
        name: &str,
        parent: Option<&Url>,
        temp: bool,
    ) -> Result<Url>;

    ///
    ///
    ///
    fn get_session_id(&self) -> Uuid;

    ///
    ///
    ///
    fn get_temp_dir(&self) -> PathBuf;

    ///
    ///
    ///
    fn get_session_dir(&self) -> PathBuf;
}
