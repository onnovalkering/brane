use crate::common::{CallPattern, Parameter, Type};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInfo {
    pub actions: Map<Action>,
    pub base: Option<String>,
    pub contributors: Option<Vec<String>>,
    pub description: Option<String>,
    pub entrypoint: Entrypoint,
    pub environment: Option<Map<String>>,
    pub dependencies: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub initialize: Option<Vec<String>>,
    pub install: Option<Vec<String>>,
    pub kind: String,
    pub name: String,
    pub types: Option<Map<Type>>,
    pub version: String,
}

#[allow(unused)]
impl ContainerInfo {
    pub fn from_path(path: PathBuf) -> Result<ContainerInfo> {
        let contents = fs::read_to_string(path)?;

        ContainerInfo::from_string(contents)
    }

    pub fn from_reader<R: Read>(r: R) -> Result<ContainerInfo> {
        let result = serde_yaml::from_reader(r)?;
        Ok(result)
    }

    pub fn from_string(contents: String) -> Result<ContainerInfo> {
        let result = serde_yaml::from_str(&contents)?;
        Ok(result)
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub command: Option<ActionCommand>,
    pub description: Option<String>,
    pub endpoint: Option<ActionEndpoint>,
    pub pattern: Option<CallPattern>,
    pub input: Option<Vec<Parameter>>,
    pub output: Option<Vec<Parameter>>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionCommand {
    pub args: Vec<String>,
    pub capture: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEndpoint {
    pub method: Option<String>,
    pub path: String,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entrypoint {
    pub kind: String,
    pub exec: String,
    pub content: Option<String>,
    pub delay: Option<u64>,
}
