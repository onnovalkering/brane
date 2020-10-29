use crate::ExecuteInfo;
use anyhow::Result;
use std::env;
use xenon_rs::compute::JobDescription;
use xenon_rs::compute::Scheduler;
use xenon_rs::credentials::Credential;
use grpcio::{ChannelBuilder, EnvBuilder};
use rand::{distributions::Alphanumeric, Rng};
use std::sync::Arc;

type Map<T> = std::collections::HashMap<String, T>;

lazy_static! {
    static ref HOSTNAME: String = env::var("HPC_HOSTNAME").unwrap_or_else(|_| String::from("slurm"));
    static ref RUNTIME: String = env::var("HPC_RUNTIME").unwrap_or_else(|_| String::from("singularity"));
    static ref SCHEDULER: String = env::var("HPC_SCHEDULER").unwrap_or_else(|_| String::from("slurm"));
    static ref XENON: String = env::var("HPC_XENON").unwrap_or_else(|_| String::from("localhost:50051"));


    // TODO: fetch credentials from vault (requires some refactoring to avoid circular dependencies).
    static ref USERNAME: String = env::var("HPC_USERNAME").unwrap_or_else(|_| String::from("xenon"));
    static ref PASSWORD: String = env::var("HPC_PASSWORD").unwrap_or_else(|_| String::from("javagat"));
}

///
///
///
pub async fn run(exec: ExecuteInfo) -> Result<()> {
    let scheduler = create_slurm_scheduler()?;
    let (executable, arguments) = determine_command(&exec)?;

    let identifier = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .collect::<String>()
        .to_lowercase();

    let job_description = JobDescription {
        arguments,
        executable,
        working_directory: None,
        environment: None,
        queue: None,
        max_runtime: None,
        stderr: Some(format!("{}-stderr.txt", identifier)),
        stdin: None,
        stdout: Some(format!("{}-stdout.txt", identifier)),
        max_memory: None,
        scheduler_arguments: None,
        tasks: None,
        cores_per_tasks: None,
        tasks_per_node: None,
        start_per_task: None,
        start_time: None,
        temp_space: None,
    };

    // TODO: keep monitoring this job, for now fire-and-forget
    let job = scheduler.submit_batch_job(job_description).ok();
    println!("{:#?}", job);

    Ok(())
}

///
///
///
fn create_slurm_scheduler() -> Result<Scheduler> {
    let env = Arc::new(EnvBuilder::new().build());
    let channel = ChannelBuilder::new(env).connect(XENON.as_str());

    let credential = Credential::new_password(USERNAME.to_string(), PASSWORD.to_string());

    let mut properties = Map::<String>::new();
    properties.insert(
        String::from("xenon.adaptors.schedulers.ssh.strictHostKeyChecking"),
        String::from("false"),
    );

    let scheduler = Scheduler::create(
        SCHEDULER.to_string(),
        channel,
        credential,
        format!("ssh://{}:22", HOSTNAME.as_str()),
        properties,
    ).unwrap();

    Ok(scheduler)
}

///
///
///
fn determine_command(exec: &ExecuteInfo) -> Result<(Option<String>, Option<Vec<String>>)> {
    // Format: singularity run [-B /source:/target] {image} {arguments}
    let executable = Some(RUNTIME.to_string());
    let mut arguments = vec![String::from("run")];

    // Add mount bindings
    if let Some(mounts) = &exec.mounts {
        for mount in mounts {
            arguments.push(String::from("-B"));
            arguments.push(mount.to_string());
        }
    }

    // Add image
    let image = format!("docker://{}", exec.image);
    arguments.push(image);

    // Add arguments
    if let Some(command) = &exec.command {
        arguments.push(String::from("sh"));
        arguments.push(String::from("-c"));
        arguments.push(command.join(" "));
    }

    Ok((executable, Some(arguments)))
}
