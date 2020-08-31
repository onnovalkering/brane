use crate::schema::{invocations, packages, sessions};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use specifications::common::Value;
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::path::PathBuf;
use uuid::Uuid;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

#[derive(Serialize, Queryable, Identifiable)]
pub struct Invocation {
    pub id: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub name: Option<String>,
    pub uuid: String,
    // Content
    pub status: String,
    pub arguments_json: String,
    pub instructions_json: String,
}

#[derive(Insertable)]
#[table_name = "invocations"]
pub struct NewInvocation {
    // Metadata
    pub created: NaiveDateTime,
    pub name: Option<String>,
    pub uuid: String,
    // Content
    pub status: String,
    pub arguments_json: String,
    pub instructions_json: String,
}

impl NewInvocation {
    pub fn new(
        name: Option<String>,
        arguments: &Map<Value>,
        instructions: &[Instruction],
    ) -> FResult<Self> {
        let created = Utc::now().naive_utc();
        let uuid = Uuid::new_v4().to_string();
        let status = String::from("created");
        let arguments_json = serde_json::to_string(arguments)?;
        let instructions_json = serde_json::to_string(instructions)?;

        Ok(NewInvocation {
            created,
            name,
            uuid,
            status,
            arguments_json,
            instructions_json,
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
pub struct Session {
    pub id: i32,
    // Metadata
    pub created: NaiveDateTime,
    pub uuid: String,
    // Content
    pub status: String,
}

#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    // Metadata
    pub created: NaiveDateTime,
    pub uuid: String,
    // Content
    pub status: String,
}

impl NewSession {
    pub fn new() -> FResult<Self> {
        let created = Utc::now().naive_utc();
        let uuid = Uuid::new_v4().to_string();
        let status = String::from("active");

        Ok(NewSession {
            created,
            uuid,
            status,
        })
    }
}

#[derive(Clone)]
pub struct Config {
    pub docker_host: String,
    pub packages_dir: PathBuf,
    pub temporary_dir: PathBuf,
}
