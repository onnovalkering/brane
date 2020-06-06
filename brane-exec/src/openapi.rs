use openapiv3::{OpenAPI, Parameter, ReferenceOr};
use serde_json::Value as JValue;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

pub async fn execute(
    operation_id: String,
    arguments: Map<String>,
    oas_document: &OpenAPI,
) -> FResult<JValue> {
    let base_url = &oas_document
        .servers
        .first()
        .expect("OAS document requires a server.")
        .url;
    let (path, path_item) = oas_document
        .paths
        .iter()
        .filter(|(_, path)| {
            if let ReferenceOr::Item(path) = path {
                if let Some(get_operation) = &path.get {
                    if let Some(id) = &get_operation.operation_id {
                        return operation_id == id.to_lowercase();
                    }
                } else {
                    unimplemented!();
                }
            }

            unreachable!();
        })
        .next()
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

    let mut operation_url = format!("{}{}", base_url, path);
    for parameter in &operation.parameters {
        if let ReferenceOr::Item(parameter) = parameter {
            match parameter {
                Parameter::Path { parameter_data, .. } => {
                    let name = &parameter_data.name;
                    let value = arguments.get(name).expect("Missing argument.");
                    operation_url = operation_url.replace(&format!("{{{}}}", name), &value);
                }
                _ => unimplemented!(),
            }
        } else {
            unreachable!();
        }
    }

    let client = reqwest::Client::builder()
        .user_agent("HTTPie/1.0.3")
        .build()?;


    // Perform the requirest
    let response = client.get(&operation_url)
        .send()
        .await?
        .json::<JValue>()
        .await?;

    Ok(response)
}
