use anyhow::{Context, Result};
use specifications::status::{Status, StatusInfo};
use reqwest::blocking::Client;

pub fn start(
    callback_url: String,
    invocation_id: i32,
) -> Result<()> {
    let callback_url = format!("{}/status", callback_url);

    let context = zmq::Context::new();
    let socket = context.socket(zmq::REP)?;

    socket.bind("tcp://*:8888")?;
    loop {
        let data = socket.recv_string(0)?;
        socket.send("", 0)?;

        if let Ok(data) = data {
            let status: Status = serde_json::from_str(&data)?;
            let payload = StatusInfo::new(0, invocation_id, status);
            
            let client = Client::new();
            client.post(&callback_url)
                .json(&payload)
                .send()
                .with_context(|| format!("Failed to perform callback to: {}", callback_url))?;
        }
    }
  }