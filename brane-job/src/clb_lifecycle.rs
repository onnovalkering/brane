use crate::interface::{Callback, Event, EventKind, CallbackKind};
use anyhow::Result;

pub fn handle(callback: Callback) -> Result<Vec<(String, Event)>> {
    let job_id = callback.job.clone();
    let application = callback.application.clone();
    let location_id = callback.location.clone();
    let order = callback.order;

    let kind = match &callback.kind() {
        CallbackKind::Unknown => {
            debug!("Received Unkown callback: {:?}", callback);
            return Ok(vec![]);
        },
        CallbackKind::Ready => EventKind::Ready,
        CallbackKind::Initialized => EventKind::Initialized,
        CallbackKind::Started => EventKind::Started,
        CallbackKind::Heartbeat => unreachable!(),
        CallbackKind::Finished => EventKind::Finished,
        CallbackKind::Stopped => EventKind::Stopped,
        CallbackKind::Failed => EventKind::Failed,
    };

    let key = format!("{}#{}", job_id, order);
    let payload = callback.payload;
    let category = String::from("job");
    let event = Event::new(kind, job_id, application, location_id, category, order as u32, Some(payload), None);

    Ok(vec![(key, event)])
}
