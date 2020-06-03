#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prettytable;

pub mod build_api;
pub mod build_cwl;
pub mod build_dsl;
pub mod build_ecu;
pub mod packages;
pub mod registry;
pub mod repl;
pub mod test;
pub mod utils;

use semver::Version;
use std::process::Command;

type FResult<T> = Result<T, failure::Error>;

const MIN_DOCKER_VERSION: &str = "19.0.0";

///
///
///
pub fn check_dependencies() -> FResult<()> {
    let output = Command::new("docker").arg("--version").output()?;
    let version = String::from_utf8_lossy(&output.stdout[15..17]);

    let version = Version::parse(&format!("{}.0.0", version)).unwrap();
    ensure!(version >= Version::parse(MIN_DOCKER_VERSION)?);

    Ok(())
}
