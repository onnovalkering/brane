use crate::common::{Function, Type};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

type Map<T> = std::collections::HashMap<String, T>;

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    pub created: DateTime<Utc>,
    pub description: Option<String>,
    pub functions: Option<Map<Function>>,
    pub id: Uuid,
    pub kind: String,
    pub name: String,
    pub types: Option<Map<Type>>,
    pub version: String,
}

#[allow(unused)]
impl PackageInfo {
    pub fn new(
        name: String,
        version: String,
        description: Option<String>,
        kind: String,
        functions: Option<Map<Function>>,
        types: Option<Map<Type>>,
    ) -> PackageInfo {
        let id = Uuid::new_v4();
        let created = Utc::now();

        PackageInfo {
            created,
            description,
            functions,
            id,
            kind,
            name,
            types,
            version,
        }
    }

    pub fn from_path(path: PathBuf) -> Result<PackageInfo> {
        let contents = fs::read_to_string(path)?;

        PackageInfo::from_string(contents)
    }

    pub fn from_string(contents: String) -> Result<PackageInfo> {
        let result = serde_yaml::from_str(&contents)?;

        Ok(result)
    }
}
