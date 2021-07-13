use crate::grpc;
use anyhow::Result;
use async_trait::async_trait;
use brane_bvm::vm::{VmOptions, VmState};
use brane_bvm::{executor::VmExecutor, vm::Vm};
use brane_dsl::{Compiler, CompilerOptions, Lang};
use brane_job::interface::{Command, CommandKind};
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
use specifications::common::{Value, FunctionExt};
use specifications::package::PackageIndex;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[derive(Clone)]
pub struct DriverHandler {
    pub command_topic: String,
    pub package_index_url: String,
    pub producer: FutureProducer,
    pub results: Arc<DashMap<String, Value>>,
    pub sessions: Arc<DashMap<String, VmState>>,
    pub states: Arc<DashMap<String, String>>,
}

#[tonic::async_trait]
impl grpc::DriverService for DriverHandler {
    type ExecuteStream = ReceiverStream<Result<grpc::ExecuteReply, Status>>;

    ///
    ///
    ///
    async fn create_session(
        &self,
        _request: Request<grpc::CreateSessionRequest>,
    ) -> Result<Response<grpc::CreateSessionReply>, Status> {
        let uuid = Uuid::new_v4().to_string();

        let reply = grpc::CreateSessionReply { uuid };
        Ok(Response::new(reply))
    }

    ///
    ///
    ///
    async fn execute(
        &self,
        request: Request<grpc::ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let request = request.into_inner();
        let package_index = PackageIndex::from_url(&self.package_index_url).await.unwrap();

        let executor = JobExecutor {
            command_topic: self.command_topic.clone(),
            producer: self.producer.clone(),
            session_uuid: request.uuid.clone(),
            states: self.states.clone(),
            results: self.results.clone(),
        };

        let vm_state = self.sessions.get(&request.uuid).as_deref().cloned();

        let (tx, rx) = mpsc::channel::<Result<grpc::ExecuteReply, Status>>(10);
        tokio::spawn(async move {
            let options = CompilerOptions::new(Lang::BraneScript);
            let mut compiler = Compiler::new(options, package_index.clone());

            // Compile input and send update to client.
            let function = match compiler.compile(request.input) {
                Ok(function) => {
                    let reply = grpc::ExecuteReply {
                        output: String::new(),
                        bytecode: String::from("TODO"),
                        close: false,
                    };

                    tx.send(Ok(reply)).await.unwrap();
                    function
                }
                Err(error) => {
                    let status = Status::invalid_argument(error.to_string());
                    tx.send(Err(status)).await.unwrap();
                    return;
                }
            };

            // Restore VM state corresponding to the session, if any.
            let mut vm = if let Some(vm_state) = vm_state {
                Vm::new_with_state(executor, Some(package_index), vm_state)
            } else {
                let options = VmOptions { clear_after_main: true, ..Default::default() };
                Vm::new_with(executor, Some(package_index), Some(options))
            };

            // TEMP: needed because the VM is not completely `send`.
            futures::executor::block_on(vm.main(function));
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[derive(Clone)]
struct JobExecutor {
    pub command_topic: String,
    pub producer: FutureProducer,
    pub session_uuid: String,
    pub states: Arc<DashMap<String, String>>,
    pub results: Arc<DashMap<String, Value>>,
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

        dbg!(&message);

        let timeout = Timeout::After(Duration::from_secs(5));
        if self.producer.send(message, timeout).await.is_err() {
            bail!("Failed to send command to '{}' topic.", self.command_topic);
        }

        // TODO: await value to be in states & results.
        let call = Call {
            correlation_id: correlation_id.clone(),
            states: self.states.clone(),
            results: self.results.clone(),
        };

        Ok(call.await)
    }

    ///
    ///
    ///
    async fn wait_until(&self, _service: String, _state: brane_bvm::executor::ServiceState) -> Result<()> {
        Ok(())
    }
}

struct Call {
    correlation_id: String,
    states: Arc<DashMap<String, String>>,
    results: Arc<DashMap<String, Value>>,
}

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

impl Future for Call {
    type Output = Value;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let state = self.states.get(&self.correlation_id);
        if state.is_none() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        let state = state.unwrap().clone();
        if state == *"finished" {
            let (_, value) = self.results.remove(&self.correlation_id).unwrap();

            self.states.remove(&self.correlation_id);
            return Poll::Ready(value);
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
