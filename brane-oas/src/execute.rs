use crate::{build, resolver};
use anyhow::Result;
use cookie::Cookie as RawCookie;
use cookie_store::{Cookie, CookieStore, CookieStoreRwLock};
use openapiv3::{OpenAPI, Operation, Parameter as OParameter, ReferenceOr, SecurityScheme};
use reqwest::Url;
use specifications::common::Value;
use std::{collections::HashMap, sync::Arc};

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn execute(
    operation_id: &str,
    arguments: &Map<Value>,
    oas_document: &OpenAPI,
) -> Result<String> {
    let mut arguments = arguments.clone();
    debug!("Arguments: {:?}", arguments);

    if let Some(Value::Struct { properties, .. }) = arguments.get("input") {
        debug!("Observed a struct argument with name `input`, expanding..");

        let properties = properties.clone();
        arguments.extend(properties);

        debug!("Arguments: {:?}", arguments);
    }

    let components = oas_document.components.clone();
    let base_url = &oas_document
        .servers
        .first()
        .expect("OAS document requires a server.")
        .url;

    let (path, method, operation) = get_operation(operation_id, &oas_document)?;
    let mut operation_url = format!("{}{}", base_url, path);
    let mut cookies = CookieStore::default();
    let mut headers = vec![];
    let mut query = vec![];

    let base_url = Url::parse(base_url)?;
    for parameter in &operation.parameters {
        let parameter = resolver::resolve_parameter(parameter, &components)?;
        match parameter {
            OParameter::Cookie { parameter_data, .. } => {
                let name = &parameter_data.name;
                let value = arguments.get(name).expect("Missing argument.");

                let cookie = RawCookie::new(name.clone(), value.to_string());
                let cookie = Cookie::try_from_raw_cookie(&cookie, &base_url)?;
                cookies.insert(cookie, &base_url)?;
            }
            OParameter::Header { parameter_data, .. } => {
                let name = &parameter_data.name;
                let value = arguments.get(name).expect("Missing argument.");

                headers.push((name.clone(), value.to_string()));
            }
            OParameter::Path { parameter_data, .. } => {
                let name = &parameter_data.name;
                let value = arguments.get(name).expect("Missing argument.");
                operation_url = operation_url.replace(&format!("{{{}}}", name), &value.to_string());
            }
            OParameter::Query { parameter_data, .. } => {
                let name = &parameter_data.name;
                let value = arguments.get(name).expect("Missing argument.");

                query.push((name.clone(), value.to_string()));
            }
        }
    }

    // Determine input from security schemes.
    if let Some(security_scheme) = &operation.security.first() {
        if let Some(security_scheme) = security_scheme.keys().next() {
            let item = ReferenceOr::Reference::<SecurityScheme> {
                reference: format!("#/components/schemas/{}", security_scheme),
            };

            let security_scheme = resolver::resolve_security_scheme(&item, &components)?;
            match security_scheme {
                SecurityScheme::APIKey { name, location } => {
                    let value = arguments.get(&name).expect("Missing argument.");
                    match location {
                        openapiv3::APIKeyLocation::Query => {
                            query.push((name.clone(), value.to_string()));
                        }
                        openapiv3::APIKeyLocation::Header => {
                            headers.push((name.clone(), value.to_string()));
                        }
                        openapiv3::APIKeyLocation::Cookie => {
                            let cookie = RawCookie::new(name.clone(), value.to_string());
                            let cookie = Cookie::try_from_raw_cookie(&cookie, &base_url)?;
                            cookies.insert(cookie, &base_url)?;
                        }
                    }
                }
                SecurityScheme::HTTP { scheme, .. } => {
                    if scheme.to_lowercase() != *"bearer" {
                        todo!();
                    }

                    let value = arguments.get("token").expect("Missing `token` argument.");
                    headers.push((String::from("Authorization"), format!("Bearer {}", value)));
                }
                _ => todo!(),
            }
        }
    }

    // Build the client.
    let client = reqwest::Client::builder()
        .cookie_provider(Arc::new(CookieStoreRwLock::new(cookies)))
        .user_agent("HTTPie/2.2.0")
        .build()?;

    let mut client = match method.as_str() {
        "delete" => client.delete(&operation_url),
        "get" => client.get(&operation_url),
        "patch" => client.patch(&operation_url),
        "post" => client.post(&operation_url),
        "put" => client.put(&operation_url),
        _ => unreachable!(),
    };

    // Add query and headers to the client.
    client = client.query(&query);
    for (name, value) in headers.iter() {
        client = client.header(name.as_str(), value.to_string());
    }

    if let Some(request_body) = &operation.request_body {
        let request_body = resolver::resolve_request_body(request_body, &components)?;
        let mut json = HashMap::new();

        // Only 'application/json' request bodies are supported
        if let Some(content) = request_body.content.get("application/json") {
            if let Some(schema) = &content.schema {
                let (ref_name, schema) = resolver::resolve_schema(schema, &components)?;
                let mut _types = HashMap::new();
                let properties = build::schema_to_properties(None, &schema, true, &components, &mut _types, ref_name)?;

                for property in properties {
                    if let Some(value) = arguments.get(&property.name) {
                        json.insert(property.name.clone(), value.as_json());
                    }
                }
            }

            debug!("Request body:\n {}", serde_json::to_string_pretty(&json)?);
            client = client.json(&json);
        } else {
            unreachable!()
        }
    }

    dbg!(&client);

    // Perform the request.
    let response = client.send().await?.text().await?;
    Ok(response)
}

///
///
///
pub fn get_operation(
    operation_id: &str,
    oas_document: &OpenAPI,
) -> Result<(String, String, Operation)> {
    let (path, method, operation) = oas_document
        .paths
        .iter()
        .find_map(|(path, path_item)| {
            if let ReferenceOr::Item(path_item) = path_item {
                // Check each method-operation to see if the operation ID matches.
                let check = |op: &Option<Operation>| {
                    if let Some(op) = op {
                        if let Some(id) = &op.operation_id {
                            if operation_id == id.to_lowercase().as_str() {
                                return true;
                            }
                        }
                    }

                    false
                };

                if check(&path_item.delete) {
                    return Some((path, "delete", path_item.delete.clone()));
                }
                if check(&path_item.get) {
                    return Some((path, "get", path_item.get.clone()));
                }
                if check(&path_item.patch) {
                    return Some((path, "patch", path_item.patch.clone()));
                }
                if check(&path_item.post) {
                    return Some((path, "post", path_item.post.clone()));
                }
                if check(&path_item.put) {
                    return Some((path, "put", path_item.put.clone()));
                }
            }

            None
        })
        .expect("Mismatch in operation id");

    Ok((path.clone(), method.to_string(), operation.unwrap()))
}
