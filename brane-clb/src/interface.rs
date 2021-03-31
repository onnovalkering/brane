use prost::{Enumeration, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Callback {
    #[prost(tag = "1", enumeration = "CallbackKind")]
    pub kind: i32,
    #[prost(tag = "2", string)]
    pub job: String,
    #[prost(tag = "3", string)]
    pub application: String,
    #[prost(tag = "4", string)]
    pub location: String,
    #[prost(tag = "5", int32)]
    pub order: i32,
    #[prost(tag = "6", bytes)]
    pub payload: Vec<u8>,
}

impl Callback {
    ///
    ///
    ///
    pub fn new<S, B>(
        kind: CallbackKind,
        job: S,
        application: S,
        location: S,
        order: i32,
        payload: B,
    ) -> Self
    where
        S: Into<String> + Clone,
        B: Into<Vec<u8>>,
    {
        Callback {
            kind: kind.into(),
            job: job.into(),
            application: application.into(),
            location: location.into(),
            order,
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum CallbackKind {
    Unknown = 0,
    Ready = 1,
    Initialized = 2,
    Started = 3,
    Heartbeat = 4,
    Finished = 5,
    Stopped = 6,
    Failed = 7,
}
