use crate::{docker, openapi};
use crate::ExecuteInfo;
use anyhow::Result;
use specifications::common::Value;
use specifications::instructions::ActInstruction;
use openapiv3::OpenAPI;
use std::io::BufReader;
use serde_json::json;
use std::fs::File;
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn exec_cwl(
    _act: &ActInstruction,
    _arguments: Map<Value>,
) -> Result<Option<Value>> {
    unimplemented!();
}

///
///
///
pub async fn exec_ecu(
    act: &ActInstruction,
    arguments: Map<Value>,
) -> Result<Option<Value>> {
    let image = act.meta.get("image").expect("Missing `image` metadata property.");
    let image_file = act.meta.get("image_file").map(PathBuf::from);
    let payload = json!({
        "identifier": "test",
        "action": act.name,
        "arguments": arguments,
    });

    let command = vec![String::from("exec"), base64::encode(serde_json::to_string(&payload)?)];
    debug!("{:?}", command);

    let exec = ExecuteInfo::new(
        image.clone(),
        image_file,
        None,
        None,
        Some(command),
    );

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    if stderr.len() > 0 {
        error!("stderr: {}", stderr);
    }

    debug!("stdout: {}", stdout);

    let output: Map<Value> = serde_json::from_str(&stdout)?;
    for (_, value) in output.iter() {
        return Ok(Some(value.clone()))
    }

    Ok(None)
}

///
///
///
pub async fn exec_oas(
    act: &ActInstruction,
    arguments: Map<Value>,
) -> Result<Option<Value>> {
    let oas_file = act.meta.get("oas_file").map(PathBuf::from).expect("Missing OAS document.");
    let oas_reader = BufReader::new(File::open(&oas_file)?);
    let oas_document: OpenAPI = serde_yaml::from_reader(oas_reader)?;

    let json = openapi::execute(&act.name, arguments, &oas_document).await?;
    let output = Value::from_json(&json);

    match &output {
        Value::Unit => Ok(None),
        Value::Struct { properties, .. } => {
            if let Some(data_type) = &act.data_type {
                Ok(Some(Value::Struct { data_type: data_type.clone(), properties: properties.clone() }))
            } else {
                Ok(Some(output))
            }
        },
        _ => Ok(Some(output))
    }
}
