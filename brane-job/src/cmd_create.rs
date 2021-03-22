use crate::interface::{Command, Event};
use anyhow::{Context, Result};
use brane_cfg::infrastructure::{Location, LocationCredentials};
use brane_cfg::{Infrastructure, Secrets};
use grpcio::Channel;
use xenon::compute::{JobDescription, Scheduler};
use xenon::credentials::Credential;

///
///
///
pub fn handle(
    key: &String,
    command: Command,
    infra: Infrastructure,
    secrets: Secrets,
    xenon_channel: Channel,
) -> Result<Vec<(String, Event)>> {
    let context = || format!("CREATE command failed or is invalid (key: {}).", key);

    validate_command(&command).with_context(context)?;

    // Retreive location metadata and credentials.
    let location = infra
        .get_location_metadata(command.location.clone().unwrap())
        .with_context(context)?;

    let credentials = location.credentials.resolve_secrets(&secrets).with_context(context)?;

    // Branch into specific handlers based on the location kind.
    match location.kind.as_str() {
        "k8s" => handle_k8s(command, location, credentials).with_context(context),
        "slurm" | "vm" => handle_xenon(command, location, credentials, xenon_channel).with_context(context),
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
fn handle_k8s(
    _command: Command,
    _location: Location,
    _credentials: LocationCredentials,
) -> Result<Vec<(String, Event)>> {
    unimplemented!()
}

///
///
///
fn handle_xenon(
    command: Command,
    location: Location,
    credentials: LocationCredentials,
    xenon_channel: Channel,
) -> Result<Vec<(String, Event)>> {
    let credentials = Credential::new_password(credentials.username, credentials.password);

    let mut scheduler = match location.kind.to_lowercase().as_str() {
        "vm" => create_vm_scheduler(&location.address, credentials, xenon_channel)?,
        "slurm" => create_slurm_scheduler(&location.address, credentials, xenon_channel)?,
        _ => unreachable!(),
    };

    let job_description = match location.runtime.to_lowercase().as_str() {
        "singularity" => create_singularity_job_description(command)?,
        "docker" => create_docker_job_description(command)?,
        _ => unreachable!(),
    };

    let job = scheduler.submit_batch_job(job_description)?;
    debug!("{:?}", job);

    scheduler.close()?;

    Ok(vec![])
}

///
///
///
fn create_vm_scheduler(
    location_address: &String,
    xenon_credential: Credential,
    xenon_channel: Channel,
) -> Result<Scheduler> {
    let properties = hashmap! {
        "xenon.adaptors.schedulers.ssh.strictHostKeyChecking" => "false"
    };

    let scheduler = Scheduler::create(
        "ssh",
        xenon_channel,
        xenon_credential,
        location_address,
        Some(properties),
    )?;

    Ok(scheduler)
}

///
///
///
fn create_slurm_scheduler(
    location_address: &String,
    xenon_credential: Credential,
    xenon_channel: Channel,
) -> Result<Scheduler> {
    let properties = hashmap! {
        "xenon.adaptors.schedulers.ssh.strictHostKeyChecking" => "false"
    };

    let scheduler = Scheduler::create(
        "slurm",
        xenon_channel,
        xenon_credential,
        format!("ssh://{}", location_address),
        Some(properties),
    )?;

    Ok(scheduler)
}

///
///
///
fn create_docker_job_description(command: Command) -> Result<JobDescription> {
    // Format: docker run [-v /source:/target] {image} {arguments}
    let executable = String::from("docker");
    let mut arguments = vec![String::from("run")];

    // Add mount bindings
    for mount in command.mounts {
        arguments.push(String::from("-v"));
        arguments.push(format!("{}:{}", mount.source, mount.destination));
    }

    // Add image
    arguments.push(command.image.expect("unreachable!"));

    // Add command
    arguments.extend(command.command);

    let job_description = JobDescription {
        arguments: Some(arguments),
        executable: Some(executable),
        ..Default::default()
    };

    Ok(job_description)
}

///
///
///
fn create_singularity_job_description(command: Command) -> Result<JobDescription> {
    // Format: singularity run [-B /source:/target] {image} {arguments}
    let executable = String::from("singularity");
    let mut arguments = vec![String::from("run")];

    // Add mount bindings
    for mount in command.mounts {
        arguments.push(String::from("-B"));
        arguments.push(format!("{}:{}", mount.source, mount.destination));
    }

    // Add image
    arguments.push(format!("docker://{}", command.image.expect("unreachable!")));

    // Add command
    arguments.extend(command.command);

    let job_description = JobDescription {
        arguments: Some(arguments),
        executable: Some(executable),
        ..Default::default()
    };

    Ok(job_description)
}
