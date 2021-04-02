use crate::interface::{Callback, Event};
use anyhow::Result;

pub fn handle(callback: Callback) -> Result<Vec<(String, Event)>> {
    debug!("Received heartbeat callback: {:?}", callback);
    Ok(vec![])
}
