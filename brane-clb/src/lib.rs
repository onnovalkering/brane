#[macro_use]
extern crate log;

#[cfg(feature = "default")]
pub mod callback;
pub mod interface;

pub mod grpc {
    tonic::include_proto!("callback");

    pub use callback_service_client::CallbackServiceClient;
    pub use callback_service_server::CallbackService;
    pub use callback_service_server::CallbackServiceServer;
}
