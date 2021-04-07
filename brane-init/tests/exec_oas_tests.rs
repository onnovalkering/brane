use std::collections::HashMap;
use anyhow::Result;
use brane_init::exec_oas;
use std::path::PathBuf;
use specifications::common::Value;
use serde_json::Value as JValue;

#[tokio::test]
async fn test_delete_method() -> Result<()> {
    let operation_id = String::from("deleteanything");
    let mut arguments = HashMap::new();
    arguments.insert(String::from("anything"), Value::Unicode(String::from("1")));
    let working_dir = PathBuf::from("./resources/httpbin");

    let result = exec_oas::execute(&operation_id, &arguments, &working_dir).await;
    assert!(result.is_ok());

    let result: JValue = serde_json::from_str(&result.unwrap())?;
    assert_eq!(result["method"].as_str().unwrap(), "DELETE");

    Ok(())
}

#[tokio::test]
async fn test_get_method() -> Result<()> {
    let operation_id = String::from("getanything");
    let mut arguments = HashMap::new();
    arguments.insert(String::from("anything"), Value::Unicode(String::from("1")));
    let working_dir = PathBuf::from("./resources/httpbin");

    let result = exec_oas::execute(&operation_id, &arguments, &working_dir).await;
    assert!(result.is_ok());

    let result: JValue = serde_json::from_str(&result.unwrap())?;
    assert_eq!(result["method"].as_str().unwrap(), "GET");

    Ok(())
}


#[tokio::test]
async fn test_patch_method() -> Result<()> {
    let operation_id = String::from("patchanything");
    let mut arguments = HashMap::new();
    arguments.insert(String::from("anything"), Value::Unicode(String::from("1")));
    let working_dir = PathBuf::from("./resources/httpbin");

    let result = exec_oas::execute(&operation_id, &arguments, &working_dir).await;
    assert!(result.is_ok());

    let result: JValue = serde_json::from_str(&result.unwrap())?;
    assert_eq!(result["method"].as_str().unwrap(), "PATCH");

    Ok(())
}


#[tokio::test]
async fn test_post_method() -> Result<()> {
    let operation_id = String::from("postanything");
    let mut arguments = HashMap::new();
    arguments.insert(String::from("anything"), Value::Unicode(String::from("1")));
    let working_dir = PathBuf::from("./resources/httpbin");

    let result = exec_oas::execute(&operation_id, &arguments, &working_dir).await;
    assert!(result.is_ok());

    let result: JValue = serde_json::from_str(&result.unwrap())?;
    assert_eq!(result["method"].as_str().unwrap(), "POST");

    Ok(())
}

#[tokio::test]
async fn test_put_method() -> Result<()> {
    let operation_id = String::from("putanything");
    let mut arguments = HashMap::new();
    arguments.insert(String::from("anything"), Value::Unicode(String::from("1")));
    let working_dir = PathBuf::from("./resources/httpbin");

    let result = exec_oas::execute(&operation_id, &arguments, &working_dir).await;
    assert!(result.is_ok());

    let result: JValue = serde_json::from_str(&result.unwrap())?;
    assert_eq!(result["method"].as_str().unwrap(), "PUT");

    Ok(())
}
