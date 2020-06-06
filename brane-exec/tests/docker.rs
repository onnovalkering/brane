use brane_exec::{docker::*, ExecuteInfo};
use serde_json::{json, Value as JValue};
use std::path::PathBuf;
use tokio;

#[tokio::test]
async fn name() {
    let image = String::from("arithmetic:1.0.0");
    let image_file = PathBuf::from("./resources/arithmetic.tar");
    let payload = json!({
        "identifier": "1+1",
        "action": "add",
        "arguments": {
            "a": 1,
            "b": 1,
        },
    });

    let exec_info = ExecuteInfo::new(image, Some(image_file), payload);
    let (stdout, _) = run_and_wait(exec_info).await.unwrap();

    let output: JValue = serde_json::from_str(&stdout).unwrap();
    assert_eq!(output["c"].as_i64().unwrap(), 2);
}
