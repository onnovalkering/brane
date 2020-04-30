use crate::common::{Argument, FunctionNotation, Type};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use serde_yaml;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

type Map<T> = std::collections::HashMap<String, T>;
type FResult<T> = Result<T, failure::Error>;

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

    pub fn from_path(path: PathBuf) -> FResult<PackageInfo> {
        let contents = fs::read_to_string(path)?;

        PackageInfo::from_string(contents)
    }

    pub fn from_string(contents: String) -> FResult<PackageInfo> {
        let result = serde_yaml::from_str(&contents)?;

        Ok(result)
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    pub arguments: Vec<Argument>,
    pub notation: Option<FunctionNotation>,
    pub return_type: String,
}

#[allow(unused)]
impl Function {
    pub fn new(
        arguments: Vec<Argument>,
        notation: Option<FunctionNotation>,
        return_type: String,
    ) -> Function {
        Function {
            arguments,
            notation,
            return_type,
        }
    }
}
