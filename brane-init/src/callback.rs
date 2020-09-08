use anyhow::{Context, Result};
use specifications::common::Value;
use serde_json::json;

pub async fn submit(
    callback_url: &String,
    invocation_id: i32,
    value: &Value,
) -> Result<()> {
    let callback_url = format!("{}/act", callback_url);

    let payload = json!({
        "invocationId": invocation_id,
        "value": value,
    });

    let client = reqwest::Client::new();
    client.post(&callback_url)
        .json(&payload)
        .send()
        .await
        .with_context(|| format!("Failed to perform callback to: {}", callback_url))?;

    Ok(())
}