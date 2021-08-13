use crate::packages;
use anyhow::{Context as _, Result};
use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions, StartContainerOptions,
    WaitContainerOptions,
};
use bollard::errors::Error;
use bollard::image::{CreateImageOptions, ImportImageOptions, RemoveImageOptions};
use bollard::models::{DeviceRequest, HostConfig};
use bollard::Docker;
use brane_bvm::executor::VmExecutor;
use futures_util::stream::TryStreamExt;
use futures_util::StreamExt;
use hyper::Body;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use specifications::common::{FunctionExt, Value};
use specifications::package::PackageInfo;
use std::env;
use std::path::PathBuf;
use std::{collections::HashMap, default::Default, path::Path};
use tokio::fs::File as TFile;
use tokio_util::codec::{BytesCodec, FramedRead};
use uuid::Uuid;

lazy_static! {
    static ref DOCKER_NETWORK: String = env::var("DOCKER_NETWORK").unwrap_or_else(|_| String::from("host"));
    static ref DOCKER_GPUS: String = env::var("DOCKER_GPUS").unwrap_or_else(|_| String::from(""));
    static ref DOCKER_PRIVILEGED: String = env::var("DOCKER_PRIVILEGED").unwrap_or_else(|_| String::from(""));
    static ref DOCKER_VOLUME: String = env::var("DOCKER_VOLUME").unwrap_or_else(|_| String::from(""));
    static ref DOCKER_VOLUMES_FROM: String = env::var("DOCKER_VOLUMES_FROM").unwrap_or_else(|_| String::from(""));
}

#[derive(Clone, Default)]
pub struct DockerExecutor {
    pub data: Option<PathBuf>,
}

impl DockerExecutor {
    pub fn new(data: Option<PathBuf>) -> Self {
        Self { data }
    }
}

#[async_trait]
impl VmExecutor for DockerExecutor {
    async fn call(
        &self,
        function: FunctionExt,
        arguments: HashMap<String, Value>,
        location: Option<String>,
    ) -> Result<Value> {
        let package_dir = packages::get_package_dir(&function.package, Some("latest"))?;
        let package_file = package_dir.join("package.yml");
        let package_info = PackageInfo::from_path(package_file)?;

        if let Some(location) = location {
            warn!("Ignoring location '{}', running locally.", location);
        }

        // TODO: update upstream that this isn't required.
        let kind = match package_info.kind.as_str() {
            "ecu" => String::from("code"),
            "oas" => String::from("oas"),
            _ => {
                unreachable!()
            }
        };

        let image = format!("{}:{}", package_info.name, package_info.version);
        let image_file = Some(package_dir.join("image.tar"));

        let command = vec![
            String::from("-d"),
            String::from("--application-id"),
            String::from("test"),
            String::from("--location-id"),
            String::from("localhost"),
            String::from("--job-id"),
            String::from("1"),
            kind,
            function.name.clone(),
            base64::encode(serde_json::to_string(&arguments)?),
        ];

        let mounts = if let Some(data) = &self.data {
            let data = std::fs::canonicalize(data)?;
            if data.exists() {
                Some(vec![format!("{}:/data", data.into_os_string().into_string().unwrap())])
            } else {
                None
            }
        } else {
            None
        };

        let exec = ExecuteInfo::new(image, image_file, mounts, Some(command));

        if function.detached {
            let name = run(exec).await?;
            let address = get_container_address(&name).await?;

            let mut properties = HashMap::default();
            properties.insert(String::from("identifier"), Value::Unicode(name));
            properties.insert(String::from("address"), Value::Unicode(address));

            Ok(Value::Struct {
                data_type: String::from("Service"),
                properties,
            })
        } else {
            let (stdout, stderr) = run_and_wait(exec).await?;
            debug!("stderr: {}", stderr);
            debug!("stdout: {}", stdout);

            let output = stdout.lines().last().unwrap_or_default().to_string();
            decode_b64(output).or_else(|err| {
                error!("{:?}", err);
                Ok(Value::Unit)
            })
        }
    }

    ///
    ///
    ///
    async fn debug(
        &self,
        text: String,
    ) -> Result<()> {
        debug!("{}", text);
        Ok(())
    }

    ///
    ///
    ///
    async fn stderr(
        &self,
        text: String,
    ) -> Result<()> {
        eprintln!("{}", text);
        Ok(())
    }

    ///
    ///
    ///
    async fn stdout(
        &self,
        text: String,
    ) -> Result<()> {
        println!("{}", text);
        Ok(())
    }

    ///
    ///
    ///
    async fn wait_until(
        &self,
        name: String,
        state: brane_bvm::executor::ServiceState,
    ) -> Result<()> {
        if let brane_bvm::executor::ServiceState::Started = state {
            return Ok(());
        }

        let docker = Docker::connect_with_local_defaults()?;
        docker
            .wait_container(&name, None::<WaitContainerOptions<String>>)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }
}

///
///
///
fn decode_b64<T>(input: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let input =
        base64::decode(input).with_context(|| "Decoding failed, encoded input doesn't seem to be Base64 encoded.")?;

    let input = String::from_utf8(input[..].to_vec())
        .with_context(|| "Conversion failed, decoded input doesn't seem to be UTF-8 encoded.")?;

    serde_json::from_str(&input)
        .with_context(|| "Deserialization failed, decoded input doesn't seem to be as expected.")
}

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

///
///
///
pub async fn run(exec: ExecuteInfo) -> Result<String> {
    let docker = Docker::connect_with_local_defaults()?;

    // Either import or pull image, if not already present
    ensure_image(&docker, &exec).await?;

    // Start container, return immediatly
    create_and_start_container(&docker, &exec).await
}

///
///
///
pub async fn run_and_wait(exec: ExecuteInfo) -> Result<(String, String)> {
    let docker = Docker::connect_with_local_defaults()?;

    // Either import or pull image, if not already present
    ensure_image(&docker, &exec).await?;

    // Start container and wait for completion
    let name = create_and_start_container(&docker, &exec).await?;
    docker
        .wait_container(&name, None::<WaitContainerOptions<String>>)
        .try_collect::<Vec<_>>()
        .await?;

    // Get stdout and stderr logs from container
    let logs_options = Some(LogsOptions::<String> {
        stdout: true,
        stderr: true,
        ..Default::default()
    });

    let log_outputs = &docker.logs(&name, logs_options).try_collect::<Vec<LogOutput>>().await?;

    let mut stderr = String::new();
    let mut stdout = String::new();

    for log_output in log_outputs {
        match log_output {
            LogOutput::StdErr { message } => stderr.push_str(String::from_utf8_lossy(message).as_ref()),
            LogOutput::StdOut { message } => stdout.push_str(String::from_utf8_lossy(message).as_ref()),
            _ => unreachable!(),
        }
    }

    // Don't leave behind any waste: remove container
    remove_container(&docker, &name).await?;

    Ok((stdout, stderr))
}

///
///
///
async fn create_and_start_container(
    docker: &Docker,
    exec: &ExecuteInfo,
) -> Result<String> {
    // Generate unique (temporary) container name
    let name = Uuid::new_v4().to_string().chars().take(8).collect::<String>();
    let create_options = CreateContainerOptions { name: &name };

    let device_requests = if DOCKER_GPUS.as_str() != "" {
        let device_request = DeviceRequest {
            driver: None,
            count: Some(-1),
            device_ids: None,
            capabilities: Some(vec![vec![String::from("gpu")]]),
            options: None,
        };

        Some(vec![device_request])
    } else {
        None
    };

    let volumes_from = if DOCKER_VOLUMES_FROM.as_str() != "" {
        Some(vec![DOCKER_VOLUMES_FROM.to_string()])
    } else {
        None
    };

    let mut binds = if DOCKER_VOLUME.as_str() != "" {
        vec![format!("{}:/brane", DOCKER_VOLUME.as_str())]
    } else {
        exec.mounts.clone().unwrap_or_default()
    };

    binds.push(String::from("/var/run/docker.sock:/var/run/docker.sock"));

    let host_config = HostConfig {
        binds: Some(binds),
        network_mode: Some(DOCKER_NETWORK.to_string()),
        privileged: Some(DOCKER_PRIVILEGED.as_str() == "true"),
        volumes_from,
        device_requests,
        ..Default::default()
    };

    let create_config = Config {
        image: Some(exec.image.clone()),
        cmd: exec.command.clone(),
        host_config: Some(host_config),
        ..Default::default()
    };

    docker.create_container(Some(create_options), create_config).await?;
    docker
        .start_container(&name, None::<StartContainerOptions<String>>)
        .await?;

    Ok(name)
}

///
///
///
async fn ensure_image(
    docker: &Docker,
    exec: &ExecuteInfo,
) -> Result<()> {
    // Abort, if image is already loaded
    if docker.inspect_image(&exec.image).await.is_ok() {
        debug!("Image already exists in Docker deamon.");
        return Ok(());
    }

    if let Some(image_file) = &exec.image_file {
        debug!("Image doesn't exist in Docker deamon: importing...");
        import_image(docker, image_file).await
    } else {
        debug!("Image '{}' doesn't exist in Docker deamon: pulling...", exec.image);
        pull_image(docker, exec.image.clone()).await
    }
}

///
///
///
async fn import_image(
    docker: &Docker,
    image_file: &Path,
) -> Result<()> {
    let options = ImportImageOptions { quiet: true };

    let file = TFile::open(image_file).await?;
    let byte_stream = FramedRead::new(file, BytesCodec::new()).map(|r| {
        let bytes = r.unwrap().freeze();
        Ok::<_, Error>(bytes)
    });

    let body = Body::wrap_stream(byte_stream);
    docker.import_image(options, body, None).try_collect::<Vec<_>>().await?;

    Ok(())
}

///
///
///
async fn pull_image(
    docker: &Docker,
    image: String,
) -> Result<()> {
    let options = Some(CreateImageOptions {
        from_image: image,
        ..Default::default()
    });

    docker.create_image(options, None, None).try_collect::<Vec<_>>().await?;

    Ok(())
}

///
///
///
async fn remove_container(
    docker: &Docker,
    name: &str,
) -> Result<()> {
    let remove_options = Some(RemoveContainerOptions {
        force: true,
        ..Default::default()
    });

    docker.remove_container(name, remove_options).await?;

    Ok(())
}

///
///
///
pub async fn remove_image(name: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;

    let image = docker.inspect_image(name).await;
    if image.is_err() {
        return Ok(());
    }

    let remove_options = Some(RemoveImageOptions {
        force: true,
        ..Default::default()
    });

    let image = image.unwrap();
    docker.remove_image(&image.id, remove_options, None).await?;

    Ok(())
}

///
///
///
pub async fn get_container_address(name: &str) -> Result<String> {
    let docker = Docker::connect_with_local_defaults()?;

    let container = docker.inspect_container(name, None).await?;
    let networks = container
        .network_settings
        .map(|n| n.networks)
        .flatten()
        .unwrap_or_default();

    let address = if let Some(network) = networks.values().next() {
        let ip = network.ip_address.clone().unwrap_or_default();

        if ip.is_empty() {
            String::from("127.0.0.1")
        } else {
            ip
        }
    } else {
        String::new()
    };

    Ok(address)
}
