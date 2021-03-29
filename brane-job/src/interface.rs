use prost::{Enumeration, Message};
use time::OffsetDateTime;
use std::fmt;

#[derive(Clone, PartialEq, Message)]
pub struct Command {
    #[prost(tag = "1", enumeration = "CommandKind")]
    pub kind: i32,
    #[prost(tag = "2", optional, string)]
    pub identifier: Option<String>,
    #[prost(tag = "3", optional, string)]
    pub application: Option<String>,
    #[prost(tag = "4", optional, string)]
    pub location: Option<String>,
    #[prost(tag = "5", optional, string)]
    pub image: Option<String>,
    #[prost(tag = "6", repeated, string)]
    pub command: Vec<String>,
    #[prost(tag = "7", repeated, message)]
    pub mounts: Vec<Mount>,
}

impl Command {
    pub fn new<S: Into<String> + Clone>(
        kind: CommandKind,
        identifier: Option<S>,
        application: Option<S>,
        location: Option<S>,
        image: Option<S>,
        command: Vec<S>,
        mounts: Option<Vec<Mount>>,
    ) -> Self {
        Command {
            kind: kind as i32,
            identifier: identifier.map(S::into),
            application: application.map(S::into),
            location: location.map(S::into),
            image: image.map(S::into),
            command: command.iter().map(S::clone).map(S::into).collect(),
            mounts: mounts.unwrap_or_default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum CommandKind {
    Unknown = 0,
    Create = 1,
    Stop = 3,
}

impl fmt::Display for CommandKind {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}

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
    #[prost(tag = "5", uint32)]
    pub order: u32,
    #[prost(tag = "6", int64)]
    pub timestamp: i64,
}

impl Event {
    ///
    ///
    ///
    pub fn new<S: Into<String> + Clone>(
        kind: EventKind,
        identifier: S,
        application: S,
        location: S,
        order: u32,
        timestamp: Option<i64>,
    ) -> Self {
        let timestamp = timestamp.unwrap_or_else(|| OffsetDateTime::now_utc().unix_timestamp());

        Event {
            kind: kind as i32,
            identifier: identifier.into(),
            application: application.into(),
            location: location.into(),
            order,
            timestamp,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum EventKind {
    Unknown = 0,
    Created = 1,
    Stopped = 2,
}

#[derive(Clone, PartialEq, Message)]
pub struct Mount {
    #[prost(tag = "1", string)]
    pub source: String,
    #[prost(tag = "2", string)]
    pub destination: String,
}

impl Mount {
    pub fn new<S: Into<String>>(
        source: S,
        destination: S,
    ) -> Self {
        Mount {
            source: source.into(),
            destination: destination.into(),
        }
    }
}
