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
pub struct GroupMeta {
    pub created: DateTime<Utc>,
    pub contributors: Option<Vec<String>>,
    pub description: Option<String>,
    pub functions: Map<Function>,
    pub id: Uuid,
    pub image_id: Option<String>,
    pub kind: String,
    pub license: Option<String>,
    pub name: String,
    pub types: Option<Map<Type>>,
    pub version: String,
}

#[allow(unused)]
impl GroupMeta {
    pub fn new_action(
        name: String,
        version: String,
        description: Option<String>,
        contributors: Option<Vec<String>>,
        license: Option<String>,
        functions: Map<Function>,
        image_id: Option<String>,
    ) -> GroupMeta {
        GroupMeta {
            created: Utc::now(),
            contributors,
            description,
            functions,
            id: Uuid::new_v4(),
            image_id,
            kind: "action".to_string(),
            license,
            name,
            types: None,
            version,
        }
    }

    pub fn new_activity(
        name: String,
        version: String,
        description: Option<String>,
        contributors: Option<Vec<String>>,
        license: Option<String>,
        functions: Map<Function>,
        types: Option<Map<Type>>,
    ) -> GroupMeta {
        GroupMeta {
            created: Utc::now(),
            contributors,
            description,
            functions,
            id: Uuid::new_v4(),
            image_id: None,
            kind: "activity".to_string(),
            license,
            name,
            types,
            version,
        }
    }

    pub fn from_path(path: PathBuf) -> FResult<GroupMeta> {
        let contents = fs::read_to_string(path)?;

        GroupMeta::from_string(contents)
    }

    pub fn from_string(contents: String) -> FResult<GroupMeta> {
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
