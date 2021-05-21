use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusInfo {
    pub created: DateTime<Utc>,
    pub instruction_id: i32,
    pub invocation_id: i32,
    pub status: Status,
}

impl StatusInfo {
    pub fn new(
        instruction_id: i32,
        invocation_id: i32,
        status: Status,
    ) -> Self {
        let created = Utc::now();

        StatusInfo {
            created,
            instruction_id,
            invocation_id,
            status,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "variant", rename_all = "camelCase")]
pub enum Status {
    Transfer(TransferStatus),
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferStatus {
    pub geo_source: Option<(f32, f32)>,
    pub geo_destination: Option<(f32, f32)>,
    pub total_files: Option<i32>,
    pub total_size: Option<i32>,
    pub done_files: Option<i32>,
    pub done_size: Option<i32>,
}
