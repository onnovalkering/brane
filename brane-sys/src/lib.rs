pub mod local;

use anyhow::Result;
use url::Url;
use uuid::Uuid;

pub trait System {
    fn get_session_id(&self) -> Uuid;

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
}
