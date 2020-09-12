use crate::ExecuteInfo;
use crate::docker;
use anyhow::Result;
use specifications::common::Value;
use specifications::instructions::ActInstruction;
use std::path::PathBuf;
use std::env;

type Map<T> = std::collections::HashMap<String, T>;

lazy_static! {
    static ref API_HOST: String = env::var("API_HOST").unwrap_or_else(|_| String::from("brane-api:8080"));
    static ref CALLBACK_URL: String = format!("http://{}/callback", API_HOST.as_str());
    static ref DOCKER_HOST: String = env::var("DOCKER_HOST").unwrap_or_else(|_| String::from("localhost:5000"));
}

///
///
///
pub async fn cwl(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
) -> Result<()> {
    let mut image = act.meta.get("image").expect("Missing `image` metadata property.").clone();

    let image_file = act.meta.get("image_file").map(PathBuf::from);
    if image_file.is_none() {
        image = format!("{}/library/{}", DOCKER_HOST.as_str(), image);
    }

    let mounts = vec![
        String::from("/var/run/docker.sock:/var/run/docker.sock"),
        String::from("/tmp:/tmp"),
    ];

    let command = vec![
        String::from("-c"),
        String::from(CALLBACK_URL.as_str()),
        String::from("-i"),
        format!("{}", invocation_id),
        String::from("cwl"),
        String::from(&act.name),
        base64::encode(serde_json::to_string(&arguments)?)];

    let exec = ExecuteInfo::new(image.clone(), image_file, Some(mounts), Some(command));

    docker::run(exec).await?;
    Ok(())
}

///
///
///
pub async fn src(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
) -> Result<()> {
    unimplemented!()
}

///
///
///
pub async fn ecu(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
) -> Result<()> {
    let mut image = act.meta.get("image").expect("Missing `image` metadata property.").clone();

    let image_file = act.meta.get("image_file").map(PathBuf::from);
    if image_file.is_none() {
        image = format!("{}/library/{}", DOCKER_HOST.as_str(), image);
    }

    let mounts = vec![
        String::from("/tmp:/tmp"),
    ];    

    let command = vec![
        String::from("-c"),
        String::from(CALLBACK_URL.as_str()),
        String::from("-i"),
        format!("{}", invocation_id),
        String::from("ecu"),
        String::from(&act.name),
        base64::encode(serde_json::to_string(&arguments)?)];

    let exec = ExecuteInfo::new(image.clone(), image_file, Some(mounts), Some(command));

    docker::run(exec).await?;
    Ok(())
}

///
///
///
pub async fn oas(
    act: &ActInstruction,
    arguments: Map<Value>,
    invocation_id: i32,
) -> Result<()> {
    let mut image = act.meta.get("image").expect("Missing `image` metadata property.").clone();

    Ok(())
}