use crate::ExecuteInfo;
use crate::docker;
use anyhow::Result;
use serde_json::{json, Value as JValue};
use specifications::common::Value;
use specifications::instructions::ActInstruction;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::env;

type Map<T> = std::collections::HashMap<String, T>;

lazy_static! {
    static ref CALLBACK_URL: String = env::var("CALLBACK_URL").unwrap_or_else(|_| String::from("brane-api:8080/callback"));
}

///
///
///
pub async fn cwl(
    act: &ActInstruction,
    arguments: Map<Value>,
) -> Result<Option<Value>> {
    // Create (temporary) working directory
    let working_dir = tempfile::tempdir()?.into_path();
    let working_dir_str = working_dir.to_string_lossy().to_string();

    // Copy CWL document to working directory
    if let Some(cwl_file) = act.meta.get("cwl_file").map(PathBuf::from) {
        fs::copy(cwl_file, working_dir.join("document.cwl"))?;
    } else {
        // Try to get it from the registry
        debug!("Not implemented!");
        unimplemented!();
    }

    // Create file containing input arguments
    let mut input = Map::<JValue>::new();
    for (name, value) in arguments.iter() {
        input.insert(name.clone(), value.as_json());
    }

    let input_path = working_dir.join("input.json");
    let mut input_file = File::create(input_path)?;
    writeln!(input_file, "{}", &serde_json::to_string(&input)?)?;

    // Setup execution
    let image = String::from("commonworkflowlanguage/cwltool");
    let mounts = vec![
        String::from("/var/run/docker.sock:/var/run/docker.sock"),
        String::from("/tmp:/tmp"),
        format!("{}:{}", working_dir_str, working_dir_str),
    ];

    let command = vec!["--quiet", "document.cwl", "input.json"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let exec = ExecuteInfo::new(image, None, Some(mounts), Some(working_dir_str), Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    if !stderr.is_empty() {
        warn!("{}", stderr);
    }

    // Determine data type
    let data_type = if let Some(data_type) = &act.data_type {
        if data_type == "unit" {
            return Ok(None);
        }

        data_type
    } else {
        return Ok(None);
    };

    let output: Map<JValue> = serde_json::from_str(&stdout)?;
    if let Some((_, value)) = output.iter().next() {
        let mut value_file = File::open(value["path"].as_str().unwrap())?;
        let mut value = String::new();
        value_file.read_to_string(&mut value)?;

        let value = match data_type.as_str() {
            "boolean" => Value::Boolean(value.parse()?),
            "integer" => Value::Integer(value.parse()?),
            "real" => Value::Real(value.parse()?),
            "string" => Value::Unicode(value),
            _ => {
                debug!("Data type: {}", data_type);
                unimplemented!()
            }
        };

        return Ok(Some(value));
    }

    unreachable!();
}

///
///
///
pub async fn ecu(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
) -> Result<()> {
    let image = act.meta.get("image").expect("Missing `image` metadata property.");
    let image_file = act.meta.get("image_file").map(PathBuf::from);
    let payload = json!({
        "identifier": "test",
        "action": act.name,
        "arguments": arguments,
        "callback_url": CALLBACK_URL.as_str(),
        "invocation_id": invocation_id,
    });

    let command = vec![String::from("exec"), base64::encode(serde_json::to_string(&payload)?)];
    debug!("{:?}", command);

    let exec = ExecuteInfo::new(image.clone(), image_file, None, None, Some(command));

    docker::run(exec).await?;
    Ok(())
}
