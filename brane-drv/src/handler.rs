use crate::grpc;
use anyhow::Result;
use brane_dsl::{Compiler, CompilerOptions, Lang};
use brane_bvm::{VM, VmCall, VmExecutor, VmOptions, VmResult, VmState};
use brane_bvm::values::Value;
use brane_bvm::bytecode::Function;
use brane_job::interface::{Command, CommandKind};
use rdkafka::producer::{FutureRecord, FutureProducer};
use tonic::{Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use specifications::package::PackageIndex;
use uuid::Uuid;
use prost::Message as _;
use bytes::BytesMut;
use rdkafka::util::Timeout;
use std::time::Duration;
use rand::distributions::Alphanumeric;
use rand::{self, Rng};
use rdkafka::message::ToBytes;
use std::iter;
use std::sync::Arc;
use dashmap::DashMap;

#[derive(Clone)]
pub struct DriverHandler {
    pub producer: FutureProducer,
    pub command_topic: String,
    pub package_index_url: String,
    pub states: Arc<DashMap<String, String>>,
    pub results: Arc<DashMap<String, Value>>,
    pub sessions: Arc<DashMap<String, VmState>>,
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
        let packages = reqwest::get(&self.package_index_url).await.unwrap().json().await.unwrap();
        let package_index = PackageIndex::from_value(packages).unwrap();
        let sessions = self.sessions.clone();

        let executor = Executor {
            command_topic: self.command_topic.clone(),
            producer: self.producer.clone(),
            request_uuid: request.uuid.clone(),
            states: self.states.clone(),
            results: self.results.clone(),
        };

        let (tx, rx) = mpsc::channel::<Result<grpc::ExecuteReply, Status>>(10);

        tokio::spawn(async move {
            let options = CompilerOptions::new(Lang::BraneScript);
            let mut compiler = Compiler::new(options, package_index.clone());

            let function = compiler.compile(request.input)
                .map_err(|e| Status::invalid_argument(e.to_string()));

            if function.is_err() {
                tx.send(Err(function.unwrap_err())).await.unwrap();
                return;
            }

            let function = function.unwrap();

            // Disassemble bytecode to representative format
            let bytecode = if let Function::UserDefined { chunk, .. } = &function {
                chunk.disassemble().unwrap() // Infallible
            } else {
                unreachable!()
            };

            let reply = grpc::ExecuteReply { output: String::new(), bytecode: bytecode.clone(), close: false };
            tx.send(Ok(reply)).await.unwrap();

            let options = VmOptions { always_return: true };
            let mut vm = if let Some(session) = sessions.get(&request.uuid) {
                VM::new(&request.uuid, package_index.clone(), Some(session.clone()), Some(options), executor)
            } else {
                VM::new(&request.uuid, package_index.clone(), None, Some(options), executor)
            };

            if let Function::UserDefined { chunk, .. } = function {
                vm.call(chunk, 0);
            }

            loop {
                match vm.run(None).await {
                    Ok(VmResult::Ok(value)) => {
                        let output = value.map(|v| format!("{:?}", v)).unwrap_or_default();

                        sessions.insert(request.uuid.clone(), vm.state.clone());
                        let reply = grpc::ExecuteReply { output, bytecode: String::new(), close: true };
                        tx.send(Ok(reply)).await.unwrap();
                        break;
                    },
                    Ok(VmResult::RuntimeError) => {
                        tx.send(Err(Status::invalid_argument("Runtime error."))).await.unwrap();
                    }
                    _ => unreachable!()
                }
            }

            return;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[derive(Clone)]
struct Executor {
    pub command_topic: String,
    pub producer: FutureProducer,
    pub request_uuid: String,
    pub states: Arc<DashMap<String, String>>,
    pub results: Arc<DashMap<String, Value>>,
}

#[async_trait::async_trait]
impl VmExecutor for Executor {
    async fn execute(&self, call: VmCall) -> Result<Value> {
        make_function_call(
            call,
            self.command_topic.clone(),
            self.producer.clone(),
            self.request_uuid.clone(),
            self.states.clone(),
            self.results.clone(),
        ).await
    }
}

///
///
///
async fn make_function_call(
    call: VmCall,
    command_topic: String,
    producer: FutureProducer,
    session: String,
    states: Arc<DashMap<String, String>>,
    results: Arc<DashMap<String, Value>>,
) -> Result<Value> {
    let image = format!("{}:{}", call.package, call.version);
    let command = vec![
        call.kind.to_string(),
        call.function.to_string(),
        base64::encode(serde_json::to_string(&call.arguments)?),
    ];

    let session_uuid = Uuid::parse_str(&session)?;
    let session_uuid_simple = session_uuid.to_simple().to_string();

    let random_id = get_random_identifier();
    let correlation_id = format!("A{}R{}", &session_uuid_simple[..8], random_id);

    let command = Command::new(
        CommandKind::Create,
        Some(correlation_id.clone()),
        Some(session.clone()),
        call.location,
        Some(image),
        command,
        None,
    );

    let mut payload = BytesMut::with_capacity(64);
    command.encode(&mut payload)?;

    let message = FutureRecord::to(&command_topic)
        .key(&correlation_id)
        .payload(payload.to_bytes());

    dbg!(&message);

    if let Err(_) = producer.send(message, Timeout::After(Duration::from_secs(5))).await {
        bail!("Failed to send command to '{}' topic.", command_topic);
    }

    // TODO: await value to be in states & results.
    let call = Call { correlation_id: correlation_id.clone(), states: states.clone(), results: results.clone() };
    Ok(call.await)
}

///
///
///
fn get_random_identifier() -> String {
    let mut rng = rand::thread_rng();

    let identifier: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(6)
        .collect();

    identifier.to_lowercase()
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
        cx: &mut Context<'_>
    ) -> Poll<Self::Output> {
        let state = self.states.get(&self.correlation_id);
        if state.is_none() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        let state = state.unwrap().clone();
        if state == String::from("finished") {
            let (_, value) = self.results.remove(&self.correlation_id).unwrap();
            let value = value.clone();

            self.states.remove(&self.correlation_id);
            return Poll::Ready(value);
        }

        cx.waker().wake_by_ref();
        return Poll::Pending;
    }
}
