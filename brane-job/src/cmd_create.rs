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
use serde_json::{json, Value as JValue};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter;
use xenon::compute::{JobDescription, Scheduler};
use xenon::credentials::Credential;
use std::process::Command as OsCommand;

// Names of environment variables.
const BRANE_APPLICATION_ID: &str = "BRANE_APPLICATION_ID";
const BRANE_LOCATION_ID: &str = "BRANE_LOCATION_ID";
const BRANE_JOB_ID: &str = "BRANE_JOB_ID";
const BRANE_CALLBACK_TO: &str = "BRANE_CALLBACK_TO";

///
///
///
pub async fn handle(
    key: &String,
    command: Command,
    infra: Infrastructure,
    secrets: Secrets,
    xenon_channel: Channel,
) -> Result<Vec<(String, Event)>> {
    let context = || format!("CREATE command failed or is invalid (key: {}).", key);

    validate_command(&command).with_context(context)?;
    let application = command.application.clone().unwrap();
    let correlation_id = command.identifier.clone().unwrap();

    // Retreive location metadata and credentials.
    let location_id = command.location.clone().unwrap();
    let location = infra.get_location_metadata(&location_id).with_context(context)?;

    // Generate job identifier.
    let job_id = format!("{}-{}", correlation_id, get_random_identifier());

    // Branch into specific handlers based on the location kind.
    match location {
        Location::Kube {
            address,
            callback_to,
            namespace,
            credentials,
        } => {
            let environment = construct_environment(&application, &location_id, &job_id, &callback_to)?;
            let credentials = credentials.resolve_secrets(&secrets);

            handle_k8s(command, &job_id, environment, address, namespace, credentials).await?
        }
        Location::Local {
            callback_to,
            network,
        } => {
            let environment = construct_environment(&application, &location_id, &job_id, &callback_to)?;
            handle_local(command, &job_id, environment, network)?
        }
        Location::Slurm {
            address,
            callback_to,
            runtime,
            credentials,
        } => {
            let environment = construct_environment(&application, &location_id, &job_id, &callback_to)?;
            let credentials = credentials.resolve_secrets(&secrets);

            handle_xenon(
                command,
                &job_id,
                environment,
                address,
                "slurm",
                runtime,
                credentials,
                xenon_channel,
            )?
        }
        Location::Vm {
            address,
            callback_to,
            runtime,
            credentials,
        } => {
            let environment = construct_environment(&application, &location_id, &job_id, &callback_to)?;
            let credentials = credentials.resolve_secrets(&secrets);

            handle_xenon(
                command,
                &job_id,
                environment,
                address,
                "ssh",
                runtime,
                credentials,
                xenon_channel,
            )?
        }
    };

    info!(
        "Created job '{}' at location '{}' as part of application '{}'.",
        job_id, location_id, application
    );

    let order = 0; // A CREATE event is always the first, thus order=0.
    let key = format!("{}#{}", job_id, order);
    let event = Event::new(EventKind::Created, job_id, application, location_id, order, None, None);

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
fn construct_environment<S: Into<String>>(
    application_id: S,
    location_id: S,
    job_id: S,
    callback_to: S,
) -> Result<HashMap<String, String>> {
    let environment = hashmap! {
        BRANE_APPLICATION_ID.to_string() => application_id.into(),
        BRANE_LOCATION_ID.to_string() => location_id.into(),
        BRANE_JOB_ID.to_string() => job_id.into(),
        BRANE_CALLBACK_TO.to_string() => callback_to.into(),
    };

    Ok(environment)
}

///
///
///
async fn handle_k8s(
    command: Command,
    job_id: &String,
    environment: HashMap<String, String>,
    _address: String,
    namespace: String,
    credentials: LocationCredentials,
) -> Result<()> {
    // Create Kubernetes client based on config credentials
    let client = if let LocationCredentials::Config { file } = credentials {
        let config = construct_k8s_config(file).await?;
        KubeClient::try_from(config)?
    } else {
        bail!("Cannot create KubeClient from non-config credentials.");
    };

    let job_description = create_k8s_job_description(&job_id, &command, environment)?;

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

    Ok(())
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
    job_id: &String,
    command: &Command,
    environment: HashMap<String, String>,
) -> Result<Job> {
    let command = command.clone();
    let environment: Vec<JValue> = environment.iter().map(|(k, v)| json!({ "name": k, "value": v })).collect();

    let job_description = serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_id,
        },
        "spec": {
            "backoffLimit": 3,
            "ttlSecondsAfterFinished": 120,
            "template": {
                "spec": {
                    "containers": [{
                        "name": job_id,
                        "image": command.image.expect("unreachable!"),
                        "args": command.command,
                        "env": environment,
                        "securityContext": {
                            "capabilities": {
                                "drop": ["all"],
                                "add": ["NET_BIND_SERVICE", "NET_ADMIN"]
                            },
                        }
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
fn handle_local(
    command: Command,
    job_id: &String,
    environment: HashMap<String, String>,
    network: String
) -> Result<()> {
    let job_description = create_docker_job_description(&command, &job_id, environment, Some(network))?;

    let process = OsCommand::new("docker")
        .args(job_description.arguments.unwrap_or_default())
        .spawn();

    dbg!(&process);

    Ok(())
}


///
///
///
fn handle_xenon(
    command: Command,
    job_id: &String,
    environment: HashMap<String, String>,
    address: String,
    adaptor: &str,
    runtime: String,
    credentials: LocationCredentials,
    xenon_channel: Channel,
) -> Result<()> {
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
        "singularity" => create_singularity_job_description(&command, &job_id, environment)?,
        "docker" => create_docker_job_description(&command, &job_id, environment, None)?,
        _ => unreachable!(),
    };

    let _job = scheduler.submit_batch_job(job_description)?;
    scheduler.close()?;

    Ok(())
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
fn create_docker_job_description(
    command: &Command,
    job_id: &String,
    environment: HashMap<String, String>,
    network: Option<String>,
) -> Result<JobDescription> {
    let command = command.clone();

    // Format: docker run [-v /source:/target] {image} {arguments}
    let executable = String::from("docker");
    let mut arguments = vec![
        String::from("run"),
        String::from("--name"),
        String::from(job_id.clone()),
        String::from("--cap-drop"),
        String::from("ALL"),
        String::from("--cap-add"),
        String::from("NET_BIND_SERVICE"),
        String::from("--cap-add"),
        String::from("NET_ADMIN"),
    ];

    if let Some(network) = network {
        arguments.push(String::from("--network"));
        arguments.push(network);
    }

    // Add environment variables
    for (name, value) in environment {
        arguments.push(String::from("--env"));
        arguments.push(format!("{}={}", name, value));
    }

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
fn create_singularity_job_description(
    command: &Command,
    _job_id: &String,
    environment: HashMap<String, String>,
) -> Result<JobDescription> {
    let command = command.clone();

    // Format: singularity run [-B /source:/target] {image} {arguments}
    let executable = String::from("singularity");
    let mut arguments = vec![
        String::from("run"),
        String::from("--drop-caps"),
        String::from("ALL"),
        String::from("--add-caps"),
        String::from("CAP_NET_BIND_SERVICE,CAP_NET_ADMIN"),
    ];

    // Add environment variables
    for (name, value) in environment {
        arguments.push(String::from("--env"));
        arguments.push(format!("{}={}", name, value));
    }

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
