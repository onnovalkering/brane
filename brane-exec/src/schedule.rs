use crate::ExecuteInfo;
use crate::{docker, kubernetes};
use anyhow::{Context, Result};
use brane_sys::System;
use reqwest::Client;
use serde_json::{json, Value as JValue};
use specifications::common::Value;
use specifications::instructions::{ActInstruction, Instruction};
use std::env;
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;

lazy_static! {
    static ref API_HOST: String = env::var("API_HOST").unwrap_or_else(|_| String::from("brane-api:8080"));
    static ref REGISTRY_HOST: String = env::var("REGISTRY_HOST").unwrap_or_else(|_| String::from("localhost:5000"));
    static ref SYSTEM: String = env::var("SYSTEM").unwrap_or_else(|_| String::from("local"));
}

///
///
///
pub async fn cwl(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
    system: &Box<dyn System>,
) -> Result<()> {
    let (image, image_file) = determine_image(&act)?;
    let mounts = determine_mounts(vec!["/var/run/docker.sock:/var/run/docker.sock", "/tmp:/tmp"], system);
    let command = determine_cwl_command(invocation_id, "cwl", &act.name, &arguments, system)?;

    let exec = ExecuteInfo::new(image, image_file, mounts, command);
    run(exec).await
}

///
///
///
pub async fn dsl(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
) -> Result<()> {
    let name = act.meta.get("name").expect("No `name` property in metadata.");
    let version = act.meta.get("version").expect("No `version` property in metadata.");

    let client = Client::new();

    // Retreive package source (i.e. instructions)
    let package_source_url = format!("http://{}/packages/{}/{}/source", API_HOST.as_str(), name, version);
    let package_source = client
        .get(&package_source_url)
        .send()
        .await
        .with_context(|| "Failed to perform GET to retreive package instructions.")?
        .text()
        .await?;

    let instructions: Vec<Instruction> = serde_yaml::from_str(&package_source)
        .with_context(|| "Failed to parse package source as instructions (YAML).")?;

    // Create child process, with arguments
    let process_creation_url = format!("http://{}/sessions", API_HOST.as_str());
    let payload = json!({
        "invocationId": invocation_id,
        "arguments": arguments,
    });

    let process: JValue = client
        .post(&process_creation_url)
        .json(&payload)
        .send()
        .await
        .with_context(|| "Failed to perform POST to create child process.")?
        .json()
        .await
        .with_context(|| "Failed to parse POST response from child process creation.")?;

    let process_uuid = process["uuid"].as_str().expect("Missing `uuid` property.");

    // Create invocation
    let invocation_creation_url = format!("http://{}/invocations", API_HOST.as_str());
    let payload = json!({
        "session": process_uuid,
        "instructions": instructions,
    });

    client
        .post(&invocation_creation_url)
        .json(&payload)
        .send()
        .await
        .with_context(|| "Failed to perform POST to invocation creation endpoint.")?;

    Ok(())
}

///
///
///
pub async fn ecu(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
    system: &Box<dyn System>,
) -> Result<()> {
    let (image, image_file) = determine_image(&act)?;
    let mounts = determine_mounts(vec![], system);
    let command = determine_command(invocation_id, "ecu", &act.name, &arguments)?;

    let exec = ExecuteInfo::new(image, image_file, mounts, command);
    run(exec).await
}

///
///
///
pub async fn oas(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
    system: &Box<dyn System>,
) -> Result<()> {
    let (image, image_file) = determine_image(&act)?;
    let mounts = determine_mounts(vec![], system);
    let command = determine_command(invocation_id, "oas", &act.name, &arguments)?;

    let exec = ExecuteInfo::new(image, image_file, mounts, command);
    run(exec).await
}

///
///
///
fn determine_image(act: &ActInstruction) -> Result<(String, Option<PathBuf>)> {
    let mut image = act
        .meta
        .get("image")
        .expect("Missing `image` metadata property.")
        .clone();

    let image_file = act.meta.get("image_file").map(PathBuf::from);
    if image_file.is_none() {
        image = format!("{}/library/{}", REGISTRY_HOST.as_str(), image);
    }

    Ok((image, image_file))
}

///
///
///
fn determine_mounts(
    mounts: Vec<&str>,
    system: &Box<dyn System>,
) -> Option<Vec<String>> {
    let temp_dir = system.get_temp_dir();
    let session_dir = system.get_session_dir();

    let default = vec![
        format!("{0}:{0}", temp_dir.into_os_string().into_string().unwrap()),
        format!("{0}:{0}", session_dir.into_os_string().into_string().unwrap()),
    ];

    let mut mounts: Vec<String> = mounts.iter().map(|m| m.to_string()).collect();

    mounts.extend(default);

    Some(mounts)
}

///
///
///
fn determine_command(
    invocation_id: i32,
    kind: &str,
    function: &str,
    arguments: &Map<Value>,
) -> Result<Option<Vec<String>>> {
    let arguments = base64::encode(serde_json::to_string(&arguments)?);
    let callback_url = format!("http://{}/callback", API_HOST.as_str());

    let command = vec![
        String::from("-d"),
        String::from("-c"),
        String::from(callback_url),
        String::from("-i"),
        format!("{}", invocation_id),
        kind.to_string(),
        function.to_string(),
        arguments,
    ];

    Ok(Some(command))
}

///
///
///
fn determine_cwl_command(
    invocation_id: i32,
    kind: &str,
    function: &str,
    arguments: &Map<Value>,
    system: &Box<dyn System>,
) -> Result<Option<Vec<String>>> {
    let temp_dir = system.get_temp_dir();

    let arguments = base64::encode(serde_json::to_string(&arguments)?);
    let callback_url = format!("http://{}/callback", API_HOST.as_str());

    let command = vec![
        String::from("-d"),
        String::from("-c"),
        String::from(callback_url),
        String::from("-i"),
        format!("{}", invocation_id),
        kind.to_string(),
        String::from("-o"),
        String::from(temp_dir.as_os_str().to_string_lossy()),
        function.to_string(),
        arguments,
    ];

    Ok(Some(command))
}

///
///
///
async fn run(exec: ExecuteInfo) -> Result<()> {
    match SYSTEM.as_str() {
        "local" => docker::run(exec).await?,
        "kubernetes" => kubernetes::run(exec).await?,
        _ => unimplemented!(),
    }

    Ok(())
}
