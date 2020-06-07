#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prettytable;

pub mod build_cwl;
pub mod build_dsl;
pub mod build_ecu;
pub mod build_oas;
pub mod packages;
pub mod registry;
pub mod repl;
pub mod test;
pub mod utils;

use anyhow::Result;
use semver::Version;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

const MIN_DOCKER_VERSION: &str = "19.0.0";

///
///
///
pub fn check_dependencies() -> Result<()> {
    let output = Command::new("docker").arg("--version").output()?;
    let version = String::from_utf8_lossy(&output.stdout[15..17]);

    let version = Version::parse(&format!("{}.0.0", version))?;
    let minimum = Version::parse(MIN_DOCKER_VERSION)?;

    if version < minimum {
        return Err(anyhow!("Installed Docker doesn't meet the minimum requirement."));
    }

    Ok(())
}

///
///
///
pub fn determine_kind(
    context: &PathBuf,
    file: &PathBuf,
) -> Result<String> {
    let file = String::from(file.as_os_str().to_string_lossy());

    if file.starts_with("container.y") {
        return Ok(String::from("ecu"));
    }

    if file.ends_with(".bk") {
        return Ok(String::from("dsl"));
    }

    // For CWL and OAS we need to look inside the file
    let mut file = File::open(context.join(file))?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;

    if file_content.contains("cwlVersion") {
        return Ok(String::from("cwl"));
    }

    if file_content.contains("openapi") {
        return Ok(String::from("oas"));
    }

    Err(anyhow!(
        "Cannot determine target package kind based on: {:?}. Please use the --kind option.",
        file
    ))
}
