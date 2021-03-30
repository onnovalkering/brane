use anyhow::{Context, Result};
use clap::Clap;
use dotenv::dotenv;
use log::LevelFilter;
use rdkafka::{producer::FutureProducer, ClientConfig};
use tonic::{transport::Server, Request, Response, Status};

use callback::callback_service_server::{CallbackService, CallbackServiceServer};
use callback::{CallbackReply, CallbackRequest};

pub mod callback {
    tonic::include_proto!("callback");
}

pub struct MyCallback {
    callback_topic: String,
    producer: FutureProducer,
}

#[tonic::async_trait]
impl CallbackService for MyCallback {
    async fn callback(
        &self,
        request: Request<CallbackRequest>,
    ) -> Result<Response<CallbackReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CallbackReply {
            status: format!("Hello {}!", request.into_inner().job).into(),
        };

        Ok(Response::new(reply))
    }
}

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Topic to send callbacks to
    #[clap(short, long = "clb-topic", env = "CALLBACK_TOPIC")]
    callback_topic: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let opts = Opts::parse();

    // Configure logger.
    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if opts.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &opts.brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Failed to create Kafka producer.")?;

    let addr = "[::1]:50052".parse()?;
    let my_callback = MyCallback {
        callback_topic: opts.callback_topic.clone(),
        producer,
    };

    Server::builder()
        .add_service(CallbackServiceServer::new(my_callback))
        .serve(addr)
        .await?;

    Ok(())
}
