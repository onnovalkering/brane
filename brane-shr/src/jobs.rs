#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum JobStatus {
    Unknown = 0,
    Created = 1,
    Ready = 2,
    Initialized = 3,
    Started = 4,
    Finished = 5,
    Stopped = 6,
    Failed = 7,
}
