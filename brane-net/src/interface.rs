use prost::{Enumeration, Message};
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone, PartialEq, Message)]
pub struct NetEvent {
    #[prost(tag = "1", enumeration = "NetEventKind")]
    pub kind: i32,
    #[prost(tag = "2", string)]
    pub application: String,
    #[prost(tag = "3", string)]
    pub location: String,
    #[prost(tag = "4", string)]
    pub job_id: String,
    #[prost(tag = "5", uint32)]
    pub order: u32,
    #[prost(tag = "6", bytes)]
    pub payload: Vec<u8>,
    #[prost(tag = "7", int64)]
    pub timestamp: i64,
}

impl NetEvent {
    ///
    ///
    ///
    pub fn new<S: Into<String> + Clone>(
        kind: NetEventKind,
        application: S,
        location: S,
        job_id: S,
        order: u32,
        payload: Option<Vec<u8>>,
        timestamp: Option<i64>,
    ) -> Self {
        let timestamp = timestamp.unwrap_or_else(|| OffsetDateTime::now_utc().unix_timestamp());

        NetEvent {
            kind: kind as i32,
            application: application.into(),
            location: location.into(),
            job_id: job_id.into(),
            order,
            payload: payload.unwrap_or_default(),
            timestamp,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum NetEventKind {
    Unknown = -1,
    Connected = 0,
    Disconnected = 1,
}

impl fmt::Display for NetEventKind {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}
