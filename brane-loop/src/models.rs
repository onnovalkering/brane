use chrono::{NaiveDateTime, Utc};
use crate::schema::{invocations, packages, sessions, variables};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub docker_host: String,
    pub packages_dir: PathBuf,
    pub temporary_dir: PathBuf,
}

#[derive(Serialize, Queryable, Identifiable)]
pub struct Invocation {
    pub id: i32,
    pub session: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub name: Option<String>,
    pub started: Option<NaiveDateTime>,
    pub stopped: Option<NaiveDateTime>,
    pub uuid: String,
    // Content
    pub instructions_json: String,
    pub status: String,
    pub return_json: Option<String>,
}

#[derive(Serialize, Queryable, Identifiable)]
pub struct Package {
    pub id: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub kind: String,
    pub name: String,
    pub uploaded: NaiveDateTime,
    pub uuid: String,
    pub version: String,
    // Content
    pub description: Option<String>,
    pub functions_json: Option<String>,
    pub source: Option<String>,
    pub types_json: Option<String>,
    // File
    pub checksum: i64,
    pub filename: String,
}

#[derive(Serialize, Queryable, Identifiable)]
#[primary_key(id)]
pub struct Session {
    pub id: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub uuid: String,
    // Content
    pub status: String,
}

#[derive(Associations, Serialize, Queryable, Identifiable)]
#[belongs_to(Session, foreign_key = "session")]
pub struct Variable {
    pub id: i32,
    pub session: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    // Content
    pub name: String,
    pub type_: String,
    pub content_json: Option<String>,
}

#[derive(Insertable)]
#[table_name = "variables"]
pub struct NewVariable {
    pub session: i32,
    // Metadata
    pub created: NaiveDateTime,
    // Content
    pub name: String,
    pub type_: String,
    pub content_json: Option<String>,
}

impl NewVariable {
    pub fn new(
        session: i32,
        name: String,
        type_: String,
        content_json: Option<String>
    ) -> Self {
        let created = Utc::now().naive_utc();

        NewVariable {
            session,
            created,
            name,
            type_,
            content_json
        }
    }
}
