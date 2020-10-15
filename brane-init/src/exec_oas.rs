use anyhow::{Context, Result};
use openapiv3::{OpenAPI, Operation, Parameter as OParameter, ReferenceOr};
use specifications::common::{Parameter, Type, Value};
use specifications::package::PackageInfo;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn handle(
    func_name: String,
    arguments: Map<Value>,
    working_dir: PathBuf,
) -> Result<Value> {
    debug!("Executing '{}' (OAS) using arguments:\n{:#?}", func_name, arguments);

    let package_info = PackageInfo::from_path(working_dir.join("package.yml"))?;
    let functions = package_info
        .functions
        .expect("Missing `functions` property in package.yml");
    let function = functions
        .get(&func_name)
        .expect(&format!("Function '{}' not found", func_name));

    assert_input(&function.parameters, &arguments)?;
    initialize(&arguments, &working_dir)?;

    // Output variables are captured from the stdout
    let stdout = execute(&func_name, &arguments, &working_dir).await?;
    let output = capture_output(&stdout, &function.return_type, &package_info.types)?;

    if let Some(value) = output {
        Ok(value)
    } else {
        Ok(Value::Unit)
    }
}

///
///
///
fn assert_input(
    parameters: &[Parameter],
    arguments: &Map<Value>,
) -> Result<()> {
    debug!("Asserting input arguments");

    for p in parameters {
        let expected_type = p.data_type.as_str();
        let argument = arguments
            .get(&p.name)
            .with_context(|| format!("Argument not provided: {}", p.name))?;

        let actual_type = argument.data_type();
        if expected_type != actual_type {
            bail!(
                "Type check for '{}' failed. Expecting '{}' but got '{}'.",
                p.name,
                expected_type,
                actual_type
            );
        }
    }

    Ok(())
}

///
///
///
fn initialize(
    _arguments: &Map<Value>,
    _working_dir: &PathBuf,
) -> Result<()> {
    // unimplemented

    Ok(())
}

///
///
///
pub async fn execute(
    operation_id: &String,
    arguments: &Map<Value>,
    working_dir: &PathBuf,
) -> Result<String> {
    let oas_file = working_dir.join("document.yml");
    let oas_reader = BufReader::new(File::open(&oas_file)?);
    let oas_document: OpenAPI = serde_yaml::from_reader(oas_reader)?;

    let base_url = &oas_document
        .servers
        .first()
        .expect("OAS document requires a server.")
        .url;

    let (path, operation) = get_operation(operation_id, &oas_document)?;
    let mut operation_url = format!("{}{}", base_url, path);

    for parameter in &operation.parameters {
        if let ReferenceOr::Item(parameter) = parameter {
            match parameter {
                OParameter::Path { parameter_data, .. } => {
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
    let response = client.get(&operation_url).send().await?.text().await?;
    Ok(response)
}

///
///
///
pub fn get_operation(
    operation_id: &String,
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

///
///
///
fn capture_output(
    stdout: &String,
    return_type: &String,
    c_types: &Option<Map<Type>>,
) -> Result<Option<Value>> {
    let json = serde_json::from_str(stdout)?;
    let output = Value::from_json(&json);

    match &output {
        Value::Struct { properties, .. } => {
            let properties = if let Some(c_types) = c_types {
                let mut filtered = Map::<Value>::new();
                let c_type = c_types
                    .get(return_type)
                    .with_context(|| format!("Cannot find {} in custom types.", return_type))?;

                for p in &c_type.properties {
                    let property = properties
                        .get(&p.name)
                        .with_context(|| format!("Cannot find {} in output (required)", p.name))?;

                    filtered.insert(p.name.to_string(), property.clone());
                }

                filtered
            } else {
                properties.clone()
            };

            Ok(Some(Value::Struct {
                data_type: return_type.clone(),
                properties: properties,
            }))
        }
        Value::Unit => Ok(None),
        _ => Ok(Some(output)),
    }
}
