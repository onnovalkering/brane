use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, WaitContainerOptions, LogsOptions, LogOutput};
use crate::ExecuteInfo;
use uuid::Uuid;
use futures_util::stream::TryStreamExt;

type FResult<T> = Result<T, failure::Error>;

///
///
///
pub async fn run(exec: ExecuteInfo) -> FResult<()> {
    let docker = Docker::connect_with_local_defaults()?;

    let version = docker.version().await?;
    println!("{:?}", version);

    Ok(())
}

///
///
///
pub async fn run_and_wait(exec: ExecuteInfo) -> FResult<()> {
    let docker = Docker::connect_with_local_defaults()?;

    let name = create_and_start_container(&docker, exec.image, exec.payload).await?;

    &docker
        .wait_container(
            &name,
            None::<WaitContainerOptions<String>>,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let logs_options = Some(LogsOptions {
        stdout: true,
        ..Default::default()
    });

    let log_outputs = &docker.logs(&name, logs_options)
        .try_collect::<Vec<LogOutput>>()
        .await?;

    println!("{:?}", log_outputs);

    Ok(())
}

///
///
///
async fn create_and_start_container(docker: &Docker, image: String, payload: String) -> FResult<String> {
    let name = generate_identifier();

    let create_options = CreateContainerOptions {
        name: &name
    };

    let create_config = Config {
        image: Some(image),
        cmd: Some(vec!["uname".to_string()]),
        ..Default::default()
    };

    docker.create_container(Some(create_options), create_config).await?;
    docker.start_container(&name, None::<StartContainerOptions<String>>).await?;

    Ok(name)
}

fn generate_identifier() -> String {
    Uuid::new_v4().to_string().chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn name() {
        let exec_info = ExecuteInfo::new(String::from("ubuntu"), String::new());
        run_and_wait(exec_info).await.unwrap();
    }
}
