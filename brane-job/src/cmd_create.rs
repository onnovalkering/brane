use crate::interface::{Command, Event};
use anyhow::Result;

pub fn handle(_command: Command) -> Result<Vec<Event>> {
    Ok(vec![])
}
