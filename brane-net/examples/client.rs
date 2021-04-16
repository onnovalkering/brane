use std::{net::SocketAddr, str::FromStr};

use anyhow::Result;
use socksx::Socks6Client;
use tokio::io::AsyncWriteExt;


#[tokio::main]
async fn main() -> Result<()> {
    let client = Socks6Client::new("localhost:5081", None).await?;
    let socket_addr = SocketAddr::from_str("192.168.1.3:8080")?;

    let (mut socket, _) = client.connect(socket_addr, None, None).await?;
    socket.write(String::from("asdfasdf").as_bytes()).await?;

    Ok(())
}
