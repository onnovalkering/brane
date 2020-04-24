use crate::ExecuteRequest;
use failure::Error;
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, PostParams};
use kube::client::APIClient;
use kube::config;
use serde_json::json;
use std::env;

///
///
///
pub async fn schedule(request: ExecuteRequest) -> Result<String, Error> {
    let identifier = request.identifier;
    let image = request.image;
    let options = request.options;
    let payload = request.payload;

    let namespace = if let Some(namespace) = options.get("namespace") {
        String::from(namespace)
    } else {
        env::var("K8S_NAMESPACE").unwrap_or_else(|_| "default".into())
    };

    info!("Scheduling '{}' on Kubernetes (namespace: {})", image, namespace);

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
                        "command": [payload],
                    }],
                    "restartPolicy": "Never",
                }
            }
        }
    }))?;

    let config = if env::var_os("K8S_IN_CLUSTER").is_some() {
        config::incluster_config()?
    } else {
        config::load_kube_config().await?
    };

    let client = APIClient::new(config);

    let jobs: Api<Job> = Api::namespaced(client, &namespace);
    jobs.create(&PostParams::default(), resource).await?;

    Ok(identifier)
}
