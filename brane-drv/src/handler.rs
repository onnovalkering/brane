use crate::grpc::{self, *};
use anyhow::Result;
use rdkafka::producer::FutureProducer;
use tonic::{Request, Response, Status};

pub struct DriverHandler {
    pub producer: FutureProducer,
}

#[tonic::async_trait]
impl grpc::DriverService for DriverHandler {
    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        todo!();
    }

    async fn close_session(
        &self,
        request: Request<CloseSessionRequest>,
    ) -> Result<Response<CloseSessionReply>, Status> {
        todo!();
    }

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteReply>, Status> {
        todo!();
    }
}
