use crate::interface::{Callback, CallbackKind};
use anyhow::Result;
use bytes::BytesMut;
use prost::Message;
use rdkafka::message::ToBytes;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use tonic::{Request, Response, Status};

pub mod grpc {
    tonic::include_proto!("callback");

    pub use callback_service_server::CallbackService;
    pub use callback_service_server::CallbackServiceServer;
}

pub struct CallbackHandler {
    pub callback_topic: String,
    pub producer: FutureProducer,
}

#[tonic::async_trait]
impl grpc::CallbackService for CallbackHandler {
    async fn callback(
        &self,
        request: Request<grpc::CallbackRequest>,
    ) -> Result<Response<grpc::CallbackReply>, Status> {
        let message = request.into_inner();

        let kind = CallbackKind::from_i32(message.kind).unwrap();
        let application = message.application;
        let job = message.job;
        let location = message.location;
        let order = message.order;
        let payload = message.payload;

        info!(
            "Received '{:?}' callback for job '{}' at location '{}', with payload size: {} (bytes).",
            kind,
            job,
            location,
            payload.len(),
        );

        // Turn callback into a Kafka message
        let msg_key = format!("{}+{}", job, order);
        let callback = Callback::new(kind, job, application, location, order, payload);
        let mut msg_payload = BytesMut::with_capacity(64);
        callback.encode(&mut msg_payload).unwrap();

        // Send event on output topic
        let message = FutureRecord::to(&self.callback_topic)
            .key(&msg_key)
            .payload(msg_payload.to_bytes());

        let (status, message) = if let Err(error) = self.producer.send(message, Timeout::Never).await {
            error!("Failed to send event (key: {}): {:?}", msg_key, error);
            (String::from("500"), String::new())
        } else {
            (String::from("202"), String::new())
        };

        let reply = grpc::CallbackReply { status, message };
        Ok(Response::new(reply))
    }
}
