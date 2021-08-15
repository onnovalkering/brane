use anyhow::Result;
use brane_clb::grpc::{CallbackKind, CallbackRequest, CallbackServiceClient};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = CallbackServiceClient::connect("http://127.0.0.1:50052").await?;

    let request = CallbackRequest {
        kind: CallbackKind::Started.into(),
        job: String::from("job1"),
        application: String::from("app"),
        location: String::from("loc"),
        order: 1,
        payload: vec![],
    };

    let response = client.callback(request).await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}
