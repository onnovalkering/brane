use crate::grpc;
use anyhow::Result;
use brane_dsl::{Compiler, CompilerOptions};
use brane_bvm::{VM, VmResult, VmCall};
use brane_bvm::values::Value;
use brane_job::interface::{Command, CommandKind};
use rdkafka::producer::{FutureRecord, FutureProducer};
use tonic::{Request, Response, Status};
use specifications::package::PackageIndex;
use uuid::Uuid;
use prost::Message as _;
use bytes::BytesMut;
use rdkafka::util::Timeout;
use std::time::Duration;
use std::collections::HashMap;
use specifications::common::Value as SpecValue;
use rdkafka::message::ToBytes;

#[derive(Clone)]
pub struct DriverHandler {
    pub producer: FutureProducer,
    pub command_topic: String,
    pub package_index: PackageIndex,
}

#[tonic::async_trait]
impl grpc::DriverService for DriverHandler {
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
    ) -> Result<Response<grpc::ExecuteReply>, Status> {
        let request = request.into_inner();

        let options = CompilerOptions::new();
        let mut compiler = Compiler::new(options, self.package_index.clone());

        let function = compiler.compile(request.input)
            .map_err(|_| Status::invalid_argument("Compilation error."))?;

        let mut vm = VM::new(self.package_index.clone());
        vm.call(function, 1);

        loop {
            match vm.run(None) {
                VmResult::Call(call) => {
                    vm.result(make_function_call(call, &self.command_topic, &self.producer, &request.uuid).await.unwrap());
                },
                VmResult::Ok(value) => {
                    let output = value.map(|v| format!("{:?}", v)).unwrap_or_default();
                    return Ok(Response::new(grpc::ExecuteReply { output }));
                },
                VmResult::RuntimeError => {
                    return Err(Status::invalid_argument("Runtime error."))
                }
            }
        }
    }
}

///
///
///
async fn make_function_call(
    call: VmCall,
    command_topic: &String,
    producer: &FutureProducer,
    session: &String,
) -> Result<Value> {
    let image = format!("{}:{}", call.package, call.version);
    let command = vec![
        String::from("code"),
        call.function.to_string(),
        base64::encode(serde_json::to_string(&call.arguments)?),
    ];

    let correlation_id = format!("{}-random!", session);

    let command = Command::new(
        CommandKind::Create,
        Some(correlation_id.clone()),
        Some(session.clone()),
        Some(String::from("node1")),
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

    Ok(Value::Unit)
}
