use anyhow::Result;
use socksx::{self, Socks6Client};
use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use tokio::net::{TcpListener, TcpStream};

///
///
///
pub async fn start(proxy_address: String) -> Result<()> {
    let proxy_ip = proxy_address.parse::<SocketAddr>()?.ip();
    configure_iptables(&proxy_ip)?;

    let listener = TcpListener::bind("127.0.0.1:42000").await?;
    let client = Socks6Client::new(proxy_address, None).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(redirect(stream, client.clone()));
    }
}

///
///
///
fn configure_iptables(proxy_ip: &IpAddr) -> Result<()> {
    let proxy_ip = proxy_ip.to_string();

    let args = "-t nat -A OUTPUT ! -d $PROXY_IP/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000";
    let args: Vec<&str> = args.split_ascii_whitespace().collect();

    let output = Command::new("iptables")
        .env("PROXY_IP", proxy_ip)
        .args(&args)
        .output()?;

    ensure!(
        output.status.success(),
        "Failed to configure IPTables for network traffic interception."
    );

    Ok(())
}

/// Redirect an incoming TCP stream through a SOCKS6
/// proxy. The original destination of the stream has
/// been preserved, by iptables, as an socket option.
async fn redirect(
    incoming: TcpStream,
    client: Socks6Client,
) -> Result<()> {
    let mut incoming = incoming;

    let dst_addr = socksx::get_original_dst(&incoming)?;
    let initial_data = socksx::try_read_initial_data(&mut incoming).await?;
    let (mut outgoing, _) = client.connect(dst_addr, initial_data, None).await?;

    socksx::bidirectional_copy(&mut incoming, &mut outgoing).await?;

    Ok(())
}
