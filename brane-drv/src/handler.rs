use crate::executor::JobExecutor;
use crate::{grpc, packages};
use anyhow::Result;
use brane_bvm::vm::{Vm, VmOptions, VmState};
use brane_cfg::Infrastructure;
use brane_dsl::{Compiler, CompilerOptions, Lang};
use brane_shr::jobs::JobStatus;
use dashmap::DashMap;
use rdkafka::producer::FutureProducer;
use specifications::common::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[derive(Clone)]
pub struct DriverHandler {
    pub command_topic: String,
    pub graphql_url: String,
    pub producer: FutureProducer,
    pub results: Arc<DashMap<String, Value>>,
    pub sessions: Arc<DashMap<String, VmState>>,
    pub states: Arc<DashMap<String, JobStatus>>,
    pub locations: Arc<DashMap<String, String>>,
    pub infra: Infrastructure,
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
        let package_index = packages::get_package_index(&self.graphql_url).await.unwrap();
        let sessions = self.sessions.clone();

        // Prepare gRPC stream between client and (this) driver.
        let (tx, rx) = mpsc::channel::<Result<grpc::ExecuteReply, Status>>(10);

        let executor = JobExecutor {
            client_tx: tx.clone(),
            command_topic: self.command_topic.clone(),
            producer: self.producer.clone(),
            session_uuid: request.uuid.clone(),
            states: self.states.clone(),
            results: self.results.clone(),
            locations: self.locations.clone(),
            infra: self.infra.clone(),
        };

        let vm_state = sessions.get(&request.uuid).as_deref().cloned();
        tokio::spawn(async move {
            let options = CompilerOptions::new(Lang::BraneScript);
            let mut compiler = Compiler::new(options, package_index.clone());

            // Compile input and send update to client.
            let function = match compiler.compile(request.input) {
                Ok(function) => function,
                Err(error) => {
                    let status = Status::invalid_argument(error.to_string());
                    tx.send(Err(status)).await.unwrap();
                    return;
                }
            };

            // Restore VM state corresponding to the session, if any.
            let mut vm = if let Some(vm_state) = vm_state {
                debug!("Restore VM with state:\n{:?}", vm_state);
                Vm::new_with_state(executor, Some(package_index), vm_state)
            } else {
                debug!("No VM state to restore, creating new VM.");
                let options = VmOptions {
                    clear_after_main: true,
                    ..Default::default()
                };
                Vm::new_with(executor, Some(package_index), Some(options))
            };

            // TEMP: needed because the VM is not completely `send`.
            futures::executor::block_on(vm.main(function));
            let vm_state = vm.capture_state();
            sessions.insert(request.uuid, vm_state);
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
