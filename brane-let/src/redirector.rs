use anyhow::Result;
use socksx::socks6::options::SocksOption;
use socksx::{self, Socks6Client};
use std::net::IpAddr;
use std::process::Command;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};

const REDIRECTOR_ADDRESS: &str = "127.0.0.1:42000";

///
///
///
pub async fn start(
    proxy_address: String,
    options: Vec<SocksOption>,
) -> Result<()> {
    let proxy_ip = socksx::resolve_addr(&proxy_address).await?.ip();
    debug!("Going to setup network redirection to proxy with IP: {}.", proxy_ip);

    // Turn interception on as quickly as possible.
    configure_iptables(&proxy_ip)?;

    // Create a TCP listener that will receive intercepted connections.
    let listener: TcpListener = TcpListener::bind(REDIRECTOR_ADDRESS).await?;
    let client = Socks6Client::new(proxy_address, None).await?;

    tokio::spawn(async move {
        debug!("Started redirector service on: {}", REDIRECTOR_ADDRESS);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(redirect(stream, client.clone(), options.clone()));
                }
                Err(err) => {
                    error!("An error occured while trying to redirect a connection: {:?}", err);
                    break;
                }
            }
        }
    });

    // Everything *should* be instantaneous, but give it some time to be sure.
    tokio::time::sleep(Duration::from_millis(256)).await;

    Ok(())
}

///
///
///
fn configure_iptables(proxy_ip: &IpAddr) -> Result<()> {
    let proxy_ip = proxy_ip.to_string();

    let args = format!(
        "-t nat -A OUTPUT ! -d {}/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000",
        proxy_ip
    );
    let args: Vec<&str> = args.split_ascii_whitespace().collect();

    let output = Command::new("iptables").args(&args).output()?;

    // Stop execution if we can't properly configure IPTables.
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr[..]);
        error!("{}", stderr);

        bail!("Failed to configure IPTables for network traffic interception.");
    }

    debug!("Configured IPTables to intercept all outgoing TCP connections.");

    Ok(())
}

///
///
///
async fn redirect(
    incoming: TcpStream,
    client: Socks6Client,
    options: Vec<SocksOption>,
) -> Result<()> {
    let mut incoming = incoming;
    let dst_addr = socksx::get_original_dst(&incoming)?;

    debug!("Intercepted connection to: {:?}.", dst_addr);

    let (mut outgoing, _) = client.connect(dst_addr, None, Some(options)).await?;
    tokio::io::copy_bidirectional(&mut incoming, &mut outgoing).await?;

    Ok(())
}
