use anyhow::Result;
use openapiv3::{OpenAPI, Operation, Parameter, ReferenceOr};
use serde_json::Value as JValue;
use specifications::common::Value;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn execute(
    operation_id: &str,
    arguments: Map<Value>,
    oas_document: &OpenAPI,
) -> Result<JValue> {
    let base_url = &oas_document
        .servers
        .first()
        .expect("OAS document requires a server.")
        .url;

    let (path, operation) = get_operation(operation_id, oas_document)?;
    let mut operation_url = format!("{}{}", base_url, path);

    for parameter in &operation.parameters {
        if let ReferenceOr::Item(parameter) = parameter {
            match parameter {
                Parameter::Path { parameter_data, .. } => {
                    let name = &parameter_data.name;
                    let value = arguments.get(name).expect("Missing argument.");
                    match value {
                        Value::Unicode(value) => {
                            operation_url = operation_url.replace(&format!("{{{}}}", name), &value);
                        }
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            }
        } else {
            unreachable!();
        }
    }

    let client = reqwest::Client::builder().user_agent("HTTPie/1.0.3").build()?;

    // Perform the requirest
    let response = client.get(&operation_url).send().await?.json::<JValue>().await?;

    Ok(response)
}

///
///
///
pub fn get_operation(
    operation_id: &str,
    oas_document: &OpenAPI,
) -> Result<(String, Operation)> {
    let (path, path_item) = oas_document
        .paths
        .iter()
        .find(|(_, path)| {
            if let ReferenceOr::Item(path) = path {
                if let Some(get_operation) = &path.get {
                    if let Some(id) = &get_operation.operation_id {
                        return operation_id == id.to_lowercase().as_str();
                    }
                } else {
                    unimplemented!();
                }
            }

            unreachable!();
        })
        .expect("Mismatch in operation id");

    let operation = if let ReferenceOr::Item(path_item) = path_item {
        if let Some(operation) = &path_item.get {
            operation
        } else {
            unreachable!();
        }
    } else {
        unreachable!();
    };

    Ok((path.clone(), operation.clone()))
}
