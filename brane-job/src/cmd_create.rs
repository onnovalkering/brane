use crate::interface::{Command, Event, EventKind};
use anyhow::{Context, Result};
use base64;
use brane_cfg::infrastructure::{Location, LocationCredentials};
use brane_cfg::{Infrastructure, Secrets};
use grpcio::Channel;
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{Api, PostParams};
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Client as KubeClient, Config as KubeConfig};
use rand::distributions::Alphanumeric;
use rand::{self, Rng};
use serde_json::json;
use std::convert::TryFrom;
use std::iter;
use xenon::compute::{JobDescription, Scheduler};
use xenon::credentials::Credential;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

///
///
///
pub async fn handle(
    key: &String,
    command: Command,
    infra: Infrastructure,
    secrets: Secrets,
    xenon_channel: Channel,
    event_counter: Arc<AtomicU32>,
) -> Result<Vec<(String, Event)>> {
    let context = || format!("CREATE command failed or is invalid (key: {}).", key);

    validate_command(&command).with_context(context)?;
    let application = command.application.clone().unwrap();

    // Retreive location metadata and credentials.
    let location_id = command.location.clone().unwrap();
    let location = infra.get_location_metadata(&location_id).with_context(context)?;

    // Branch into specific handlers based on the location kind.
    let identifier = match location {
        Location::Kube {
            address,
            namespace,
            credentials,
        } => {
            let credentials = credentials.resolve_secrets(&secrets);
            handle_k8s(command, address, namespace, credentials).await?
        }
        Location::Slurm {
            address,
            runtime,
            credentials,
        } => {
            let credentials = credentials.resolve_secrets(&secrets);
            handle_xenon(command, address, "slurm", runtime, credentials, xenon_channel)?
        }
        Location::Vm {
            address,
            runtime,
            credentials,
        } => {
            let credentials = credentials.resolve_secrets(&secrets);
            handle_xenon(command, address, "ssh", runtime, credentials, xenon_channel)?
        }
    };

    info!("Created job '{}' at location '{}'.", identifier, location_id);

    let key = format!("{}#1", identifier);
    let count = event_counter.fetch_add(1, Ordering::Release);

    let event = Event::new(EventKind::Created, identifier, application, location_id, count, None);

    Ok(vec![(key, event)])
}

///
///
///
fn validate_command(command: &Command) -> Result<()> {
    ensure!(command.identifier.is_some(), "Identifier is not specified");
    ensure!(command.application.is_some(), "Application is not specified");
    ensure!(command.location.is_some(), "Location is not specified");
    ensure!(command.image.is_some(), "Image is not specified");

    Ok(())
}

///
///
///
async fn handle_k8s(
    command: Command,
    _address: String,
    namespace: String,
    credentials: LocationCredentials,
) -> Result<String> {
    // Create Kubernetes client based on config credentials
    let client = if let LocationCredentials::Config { file } = credentials {
        let config = construct_k8s_config(file).await?;
        KubeClient::try_from(config)?
    } else {
        bail!("Cannot create KubeClient from non-config credentials.");
    };

    let job_id = get_random_identifier();
    let job_description = create_k8s_job_description(&job_id, &command)?;

    let jobs: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let result = jobs.create(&PostParams::default(), &job_description).await;

    // Try again if job creation failed because of missing namespace.
    if let Err(error) = result {
        match error {
            kube::Error::Api(error) => {
                if error.message.starts_with("namespaces") && error.reason.as_str() == "NotFound" {
                    warn!(
                        "Failed to create k8s job because namespace '{}' didn't exist.",
                        namespace
                    );

                    // First create namespace
                    let namespaces: Api<Namespace> = Api::all(client.clone());
                    let new_namespace = create_k8s_namespace(&namespace)?;
                    let result = namespaces.create(&PostParams::default(), &new_namespace).await;

                    // Only try again if namespace creation succeeded.
                    if result.is_ok() {
                        info!("Created k8s namespace '{}'. Trying again to create k8s job.", namespace);
                        jobs.create(&PostParams::default(), &job_description).await?;
                    }
                }
            }
            _ => bail!(error),
        }
    }

    let command_id = command.identifier.clone().unwrap();
    Ok(format!("{}+{}", command_id, job_id))
}

///
///
///
async fn construct_k8s_config(config_file: String) -> Result<KubeConfig> {
    let base64_symbols = ['+', '/', '='];

    // Remove any whitespace and/or newlines.
    let config_file: String = config_file
        .chars()
        .filter(|c| c.is_alphanumeric() || base64_symbols.contains(c))
        .collect();

    // Decode and parse as YAML.
    let config_file = String::from_utf8(base64::decode(config_file)?)?;
    let config_file: Kubeconfig = serde_yaml::from_str(&config_file)?;

    KubeConfig::from_custom_kubeconfig(config_file, &KubeConfigOptions::default())
        .await
        .context("Failed to construct Kubernetes configuration object.")
}

///
///
///
fn create_k8s_job_description(
    identifier: &String,
    command: &Command,
) -> Result<Job> {
    let command = command.clone();

    let job_description = serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": identifier,
        },
        "spec": {
            "backoffLimit": 3,
            "ttlSecondsAfterFinished": 120,
            "template": {
                "spec": {
                    "containers": [{
                        "name": identifier,
                        "image": command.image.expect("unreachable!"),
                        "command": command.command,
                    }],
                    "restartPolicy": "Never",
                }
            }
        }
    }))?;

    Ok(job_description)
}

///
///
///
fn create_k8s_namespace(namespace: &String) -> Result<Namespace> {
    let namespace = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {
            "name": namespace,
        }
    }))?;

    Ok(namespace)
}

///
///
///
fn handle_xenon(
    command: Command,
    address: String,
    adaptor: &str,
    runtime: String,
    credentials: LocationCredentials,
    xenon_channel: Channel,
) -> Result<String> {
    let credentials = match credentials {
        LocationCredentials::SshCertificate {
            username,
            certificate,
            passphrase,
        } => Credential::new_certificate(certificate, username, passphrase),
        LocationCredentials::SshPassword { username, password } => Credential::new_password(username, password),
        LocationCredentials::Config { .. } => unreachable!(),
    };

    let mut scheduler = create_xenon_scheduler(address, adaptor, credentials, xenon_channel)?;

    let job_description = match runtime.to_lowercase().as_str() {
        "singularity" => create_singularity_job_description(&command)?,
        "docker" => create_docker_job_description(&command)?,
        _ => unreachable!(),
    };

    let job = scheduler.submit_batch_job(job_description)?;
    scheduler.close()?;

    let command_id = command.identifier.clone().unwrap();
    Ok(format!("{}+{}", command_id, job.id))
}

///
///
///
fn create_xenon_scheduler<S1: Into<String>, S2: Into<String>>(
    address: S1,
    adaptor: S2,
    xenon_credential: Credential,
    xenon_channel: Channel,
) -> Result<Scheduler> {
    let address = address.into();
    let adaptor = adaptor.into();

    let properties = hashmap! {
        "xenon.adaptors.schedulers.ssh.strictHostKeyChecking" => "false"
    };

    // A SLURM scheduler requires the protocol scheme in the address.
    let address = if adaptor == String::from("slurm") {
        format!("ssh://{}", address)
    } else {
        address
    };

    let scheduler = Scheduler::create(adaptor, xenon_channel, xenon_credential, address, Some(properties))?;

    Ok(scheduler)
}

///
///
///
fn create_docker_job_description(command: &Command) -> Result<JobDescription> {
    let command = command.clone();

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
fn create_singularity_job_description(command: &Command) -> Result<JobDescription> {
    let command = command.clone();

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

///
///
///
fn get_random_identifier() -> String {
    let mut rng = rand::thread_rng();

    let identifier: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(10)
        .collect();

    identifier.to_lowercase()
}
