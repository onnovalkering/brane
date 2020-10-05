use crate::ExecuteInfo;
use anyhow::Result;
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, PostParams};
use kube::client::APIClient;
use kube::config;
use serde_json::json;
use std::env;
use rand::{Rng, distributions::Alphanumeric};

lazy_static! {
    static ref NAMESPACE: String = env::var("K8S_NAMESPACE").unwrap_or_else(|_| String::from("default"));
    static ref CONFIG: String = env::var("K8S_CONFIG").unwrap_or_else(|_| String::from("kubeconfig"));
}

///
///
///
pub async fn run(exec: ExecuteInfo) -> Result<()> {
    let image = exec.image;
    let command = exec.command.unwrap_or_default();

    let identifier = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .collect::<String>()
        .to_lowercase();

    info!("Scheduling '{}' on Kubernetes (namespace: {})", image, NAMESPACE.as_str());

    let resource = serde_json::to_vec(&json!({
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
                        "image": image,
                        "command": command,
                    }],
                    "restartPolicy": "Never",
                }
            }
        }
    }))?;

    let config = if CONFIG.as_str() == "incluster" {
        config::incluster_config()?
    } else {
        config::load_kube_config().await?
    };

    let client = APIClient::new(config);

    let jobs: Api<Job> = Api::namespaced(client, NAMESPACE.as_str());
    jobs.create(&PostParams::default(), resource).await?;

    Ok(())
}
