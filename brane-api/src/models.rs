use crate::schema::packages;
use serde::Serialize;
use std::path::PathBuf;
use specifications::package::PackageInfo;
use chrono::NaiveDateTime;

#[derive(Serialize, Queryable)]
pub struct Package {
    pub id: i32,
    pub uuid: String,
    pub kind: String,
    pub name: String,
    pub version: String,
    pub created: NaiveDateTime,
    pub description: Option<String>,
    pub functions_json: Option<String>,
    pub types_json: Option<String>,
}

#[derive(Insertable)]
#[table_name = "packages"]
pub struct NewPackage {
    pub uuid: String,
    pub kind: String,
    pub name: String,
    pub version: String,
    pub created: NaiveDateTime,
    pub description: Option<String>,
    pub functions_json: Option<String>,
    pub types_json: Option<String>,
}

impl NewPackage {
    pub fn from_info(info: PackageInfo) -> Self {
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
            uuid: info.id.to_string(),
            kind: info.kind,
            name: info.name,
            version: info.version,
            created: info.created.naive_utc(),
            description: info.description,
            functions_json,
            types_json,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub packges_dir: PathBuf,
    pub temporary_dir: PathBuf,
}
