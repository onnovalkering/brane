use prost::{Enumeration, Message};
use time::OffsetDateTime;

#[derive(Clone, PartialEq, Message)]
pub struct Event {
    #[prost(tag = "1", enumeration = "EventKind")]
    pub kind: i32,
    #[prost(tag = "2", string)]
    pub identifier: String,
    #[prost(tag = "3", string)]
    pub application: String,
    #[prost(tag = "4", string)]
    pub location: String,
    #[prost(tag = "5", string)]
    pub category: String,
    #[prost(tag = "6", uint32)]
    pub order: u32,
    #[prost(tag = "7", bytes)]
    pub payload: Vec<u8>,
    #[prost(tag = "8", int64)]
    pub timestamp: i64,
}

impl Event {
    ///
    ///
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new<S: Into<String> + Clone>(
        kind: EventKind,
        identifier: S,
        application: S,
        location: S,
        category: S,
        order: u32,
        payload: Option<Vec<u8>>,
        timestamp: Option<i64>,
    ) -> Self {
        let timestamp = timestamp.unwrap_or_else(|| OffsetDateTime::now_utc().unix_timestamp());

        Event {
            kind: kind as i32,
            identifier: identifier.into(),
            application: application.into(),
            location: location.into(),
            category: category.into(),
            order,
            payload: payload.unwrap_or_default(),
            timestamp,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum EventKind {
    Unknown = -1,
    Created = 0,
    Ready = 1,
    Initialized = 2,
    Started = 3,
    Heartbeat = 4,
    Finished = 5,
    Stopped = 6,
    Failed = 7,
    Connected = 8,
    Disconnected = 9,
}
