use crate::schema::packages;
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use specifications::package::PackageInfo;
use std::path::PathBuf;

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
            types_json,
            uploaded: Utc::now().naive_utc(),
            uuid: info.id.to_string(),
            version: info.version,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub packages_dir: PathBuf,
    pub temporary_dir: PathBuf,
}
