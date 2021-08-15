use crate::grpc;
use anyhow::Result;
use async_trait::async_trait;
use brane_bvm::executor::VmExecutor;
use brane_cfg::Infrastructure;
use brane_job::interface::{Command, CommandKind};
use brane_shr::jobs::JobStatus;
use bytes::BytesMut;
use dashmap::DashMap;
use prost::Message as _;
use rand::distributions::Alphanumeric;
use rand::{self, Rng};
use rdkafka::message::ToBytes;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use specifications::common::{FunctionExt, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc::Sender;
use tonic::Status;
use uuid::Uuid;

///
///
///
#[derive(Clone)]
pub struct JobExecutor {
    pub client_tx: Sender<Result<grpc::ExecuteReply, Status>>,
    pub command_topic: String,
    pub producer: FutureProducer,
    pub session_uuid: String,
    pub states: Arc<DashMap<String, JobStatus>>,
    pub results: Arc<DashMap<String, Value>>,
    pub locations: Arc<DashMap<String, String>>,
    pub infra: Infrastructure,
}

impl JobExecutor {
    ///
    ///
    ///
    fn get_random_identifier(&self) -> String {
        let mut rng = rand::thread_rng();

        let identifier: String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(6)
            .collect();

        identifier.to_lowercase()
    }
}

#[async_trait]
impl VmExecutor for JobExecutor {
    ///
    ///
    ///
    async fn call(
        &self,
        function: FunctionExt,
        arguments: HashMap<String, Value>,
        location: Option<String>,
    ) -> Result<Value> {
        let image = format!("{}:{}", function.package, function.version);
        let command = vec![
            function.kind.to_string(),
            function.name.to_string(),
            base64::encode(serde_json::to_string(&arguments)?),
        ];

        let session_uuid = Uuid::parse_str(&self.session_uuid)?;
        let session_uuid_simple = session_uuid.to_simple().to_string();

        let random_id = self.get_random_identifier();
        let correlation_id = format!("A{}R{}", &session_uuid_simple[..8], random_id);

        let command = Command::new(
            CommandKind::Create,
            Some(correlation_id.clone()),
            Some(self.session_uuid.clone()),
            location,
            Some(image),
            command,
            None,
        );

        let mut payload = BytesMut::with_capacity(64);
        command.encode(&mut payload)?;

        let message = FutureRecord::to(&self.command_topic)
            .key(&correlation_id)
            .payload(payload.to_bytes());

        let timeout = Timeout::After(Duration::from_secs(5));
        if self.producer.send(message, timeout).await.is_err() {
            bail!("Failed to send command to '{}' topic.", self.command_topic);
        }

        if function.detached {
            // Wait until "created" (address known ?)
            let created = WaitUntil {
                at_least: JobStatus::Created,
                correlation_id: correlation_id.clone(),
                states: self.states.clone(),
            };

            info!("Waiting until (detached) job '{}' is created...", correlation_id);
            created.await;
            info!("OK, job '{}' has been created", correlation_id);

            let location = self
                .locations
                .get(&correlation_id)
                .map(|s| s.clone())
                .unwrap_or_default();

            let location = self.infra.get_location_metadata(location)?;

            let mut properties = HashMap::default();
            properties.insert(String::from("identifier"), Value::Unicode(correlation_id));
            properties.insert(String::from("address"), Value::Unicode(location.get_address()));

            Ok(Value::Struct {
                data_type: String::from("Service"),
                properties,
            })
        } else {
            let finished = WaitUntil {
                at_least: JobStatus::Finished,
                correlation_id: correlation_id.clone(),
                states: self.states.clone(),
            };

            info!("Waiting until job '{}' is finished...", correlation_id);
            finished.await;
            info!("OK, job '{}' has been finished", correlation_id);

            let (_, value) = self.results.remove(&correlation_id).unwrap();
            self.states.remove(&correlation_id);

            debug!("RESULT: {:?}", value);
            Ok(value)
        }
    }

    ///
    ///
    ///
    async fn debug(
        &self,
        text: String,
    ) -> Result<()> {
        let reply = grpc::ExecuteReply {
            close: false,
            debug: Some(text),
            stderr: None,
            stdout: None,
        };

        self.client_tx.send(Ok(reply)).await.map(|_| ()).map_err(|e| {
            error!("{:?}", e);
            anyhow!("Failed to send gRPC (print) message to client.")
        })
    }

    ///
    ///
    ///
    async fn stderr(
        &self,
        text: String,
    ) -> Result<()> {
        let reply = grpc::ExecuteReply {
            close: false,
            debug: None,
            stderr: Some(text),
            stdout: None,
        };

        self.client_tx.send(Ok(reply)).await.map(|_| ()).map_err(|e| {
            error!("{:?}", e);
            anyhow!("Failed to send gRPC (print) message to client.")
        })
    }

    ///
    ///
    ///
    async fn stdout(
        &self,
        text: String,
    ) -> Result<()> {
        let reply = grpc::ExecuteReply {
            close: false,
            debug: None,
            stderr: None,
            stdout: Some(text),
        };

        self.client_tx.send(Ok(reply)).await.map(|_| ()).map_err(|e| {
            error!("{:?}", e);
            anyhow!("Failed to send gRPC (print) message to client.")
        })
    }

    ///
    ///
    ///
    async fn wait_until(
        &self,
        _service: String,
        _state: brane_bvm::executor::ServiceState,
    ) -> Result<()> {
        Ok(())
    }
}

///
///
///
struct Wait {
    correlation_id: String,
    states: Arc<DashMap<String, JobStatus>>,
    results: Arc<DashMap<String, Value>>,
}

impl Future for Wait {
    type Output = Value;

    ///
    ///
    ///
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let state = self.states.get(&self.correlation_id);
        if let Some(state) = state {
            let state = state.value();
            match state {
                JobStatus::Failed => {
                    unimplemented!();
                }
                JobStatus::Finished => {
                    let (_, value) = self.results.remove(&self.correlation_id).unwrap();
                    self.states.remove(&self.correlation_id);

                    debug!("Job finished! Returning result");
                    return Poll::Ready(value);
                }
                JobStatus::Stopped => {
                    unimplemented!();
                }
                _ => {}
            }
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

///
///
///
struct WaitUntil {
    at_least: JobStatus,
    correlation_id: String,
    states: Arc<DashMap<String, JobStatus>>,
}

impl Future for WaitUntil {
    type Output = ();

    ///
    ///
    ///
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let state = self.states.get(&self.correlation_id);
        if let Some(state) = state {
            if *state.value() >= self.at_least {
                return Poll::Ready(());
            } else {
                debug!("{:?}", state.value());
            }
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
