use crate::interface::{Command, Event};
use anyhow::{Context, Result};
use brane_cfg::infrastructure::{Location, LocationCredentials};
use brane_cfg::{Infrastructure, Secrets};

///
///
///
pub fn handle(
    key: String,
    command: Command,
    infra: Infrastructure,
    secrets: Secrets,
) -> Result<Vec<(String, Event)>> {
    let context = || format!("CREATE command failed or is invalid (key: {}).", key);

    validate_command(&command).with_context(context)?;

    // Retreive location metadata and credentials
    let location = infra
        .get_location_metadata(command.location.clone().unwrap())
        .with_context(context)?;

    let credentials = location.credentials.resolve_secrets(&secrets).with_context(context)?;

    // Branch into specific handlers based on the location kind.
    match location.kind.as_str() {
        "k8s" => unimplemented!(),
        "vm" => handle_vm_job(command, location, credentials).with_context(context),
        _ => unreachable!(),
    }
}

///
///
///
fn validate_command(command: &Command) -> Result<()> {
    ensure!(command.location.is_some(), "Location is not specified");
    ensure!(command.image.is_some(), "Image is not specified");

    Ok(())
}

///
///
///
fn handle_vm_job(
    _command: Command,
    _location: Location,
    _credentials: LocationCredentials,
) -> Result<Vec<(String, Event)>> {
    bail!("unimplemented");
}
