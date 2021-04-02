use crate::interface::{Callback, Event, EventKind, CallbackKind};
use anyhow::Result;

pub fn handle(callback: Callback) -> Result<Vec<(String, Event)>> {
    let job_id = callback.job.clone();
    let application = callback.application.clone();
    let location_id = callback.location.clone();
    let order = callback.order;

    let kind = match &callback.kind() {
        CallbackKind::Started => EventKind::Started,
        CallbackKind::Stopped => EventKind::Stopped,
        kind => {
            debug!("Received {} callback: {:?}", kind, callback);
            return Ok(vec![]);
        }
    };

    let key = format!("{}#{}", job_id, order);
    let event = Event::new(kind, job_id, application, location_id, order as u32, None);

    Ok(vec![(key, event)])
}
