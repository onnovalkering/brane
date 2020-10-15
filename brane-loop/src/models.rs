use crate::schema::{invocations, packages, sessions, variables};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::path::PathBuf;
use uuid::Uuid;

type FResult<T> = Result<T, failure::Error>;

#[derive(Clone)]
pub struct Config {
    pub registry_host: String,
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

#[derive(Insertable)]
#[table_name = "invocations"]
pub struct NewInvocation {
    pub session: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub name: Option<String>,
    pub uuid: String,
    // Content
    pub instructions_json: String,
    pub status: String,
}

impl NewInvocation {
    pub fn new(
        session: i32,
        name: Option<String>,
        instructions: &[Instruction],
    ) -> FResult<Self> {
        let created = Utc::now().naive_utc();
        let uuid = Uuid::new_v4().to_string();
        let instructions_json = serde_json::to_string(instructions)?;
        let status = String::from("created");

        Ok(NewInvocation {
            session,
            created,
            name,
            uuid,
            instructions_json,
            status,
        })
    }
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

#[derive(Insertable)]
#[table_name = "packages"]
pub struct NewPackage {
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

impl NewPackage {
    pub fn from_info(
        info: PackageInfo,
        checksum: u32,
        filename: String,
    ) -> Self {
        let functions_json = if let Some(functions) = info.functions {
            let functions = serde_json::to_string(&functions).unwrap();
            Some(functions)
        } else {
            None
        };

        let types_json = if let Some(types) = info.types {
            let types = serde_json::to_string(&types).unwrap();
            Some(types)
        } else {
            None
        };

        NewPackage {
            checksum: checksum as i64,
            created: info.created.naive_utc(),
            description: info.description,
            filename,
            functions_json,
            kind: info.kind,
            name: info.name,
            source: None,
            types_json,
            uploaded: Utc::now().naive_utc(),
            uuid: info.id.to_string(),
            version: info.version,
        }
    }
}

#[derive(Serialize, Queryable, Identifiable)]
#[primary_key(id)]
pub struct Session {
    pub id: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub uuid: String,
    // Content
    pub parent: Option<i32>,
    pub status: String,
}

#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    // Metadata
    pub created: NaiveDateTime,
    pub uuid: String,
    // Content
    pub parent: Option<i32>,
    pub status: String,
}

impl NewSession {
    pub fn new(parent: Option<i32>) -> FResult<Self> {
        let created = Utc::now().naive_utc();
        let uuid = Uuid::new_v4().to_string();
        let status = String::from("active");

        Ok(NewSession {
            created,
            uuid,
            parent,
            status,
        })
    }
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
        content_json: Option<String>,
    ) -> FResult<Self> {
        let created = Utc::now().naive_utc();

        Ok(NewVariable {
            session,
            created,
            name,
            type_,
            content_json,
        })
    }
}
