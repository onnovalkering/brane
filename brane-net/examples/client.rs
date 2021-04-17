use std::{net::SocketAddr, str::FromStr};

use anyhow::Result;
use socksx::options::MetadataOption;
use socksx::Socks6Client;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Socks6Client::new("localhost:5081", None).await?;
    let socket_addr = SocketAddr::from_str("192.168.1.3:8080")?;

    let options = vec![
        MetadataOption::new(1, String::from("application")),
        MetadataOption::new(2, String::from("location")),
        MetadataOption::new(3, String::from("job-id")),
        MetadataOption::new(4, String::from("1")),
    ];

    let (mut socket, _) = client.connect(socket_addr, None, Some(options)).await?;
    socket.write(String::from("Hello, world!\n").as_bytes()).await?;

    Ok(())
}
