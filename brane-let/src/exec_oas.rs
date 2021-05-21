use crate::callback::Callback;
use anyhow::{Context, Result};
use specifications::common::{Parameter, Type, Value};
use specifications::package::PackageInfo;
use std::path::{Path, PathBuf};

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn handle(
    function: String,
    arguments: Map<Value>,
    working_dir: PathBuf,
    callback: &mut Option<&mut Callback>,
) -> Result<Value> {
    debug!("Executing '{}' (Web API) using arguments:\n{:#?}", function, arguments);

    let package_info = PackageInfo::from_path(working_dir.join("package.yml"))?;
    let functions = package_info
        .functions
        .expect("Missing `functions` property in package.yml");

    let function_info = functions
        .get(&function)
        .unwrap_or_else(|| panic!("Function '{}' not found", function));

    assert_input(&function_info.parameters, &arguments)?;

    // Perform initialization.
    initialize(&arguments, &working_dir)?;
    if let Some(callback) = callback {
        callback.initialized(None).await?;
    }

    if let Some(callback) = callback {
        callback.started(None).await?;
    }

    let oas_file = working_dir.join("document.yml");
    let oas_document = brane_oas::parse_oas_file(oas_file)?;

    // Output variables are captured from the stdout
    let stdout = brane_oas::execute(&function, &arguments, &oas_document).await?;
    let output = capture_output(&stdout, &function_info.return_type, &package_info.types)?;

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
    _working_dir: &Path,
) -> Result<()> {
    // unimplemented

    Ok(())
}

///
///
///
fn capture_output(
    stdout: &str,
    return_type: &str,
    c_types: &Option<Map<Type>>,
) -> Result<Option<Value>> {
    let json = serde_json::from_str(stdout)?;
    let output = Value::from_json(&json);

    let filter = |object: &Value, c_type: &Type| {
        let mut filtered = Map::<Value>::new();

        if let Value::Struct { properties, .. } = object {
            for p in &c_type.properties {
                let property = properties
                    .get(&p.name)
                    .with_context(|| format!("Cannot find {} in output (required)", p.name))
                    .unwrap();

                filtered.insert(p.name.to_string(), property.clone());
            }

            return Value::Struct {
                data_type: c_type.name.clone(),
                properties: filtered,
            };
        }

        object.clone()
    };

    match &output {
        Value::Array { entries, .. } => {
            if let Some(c_types) = c_types {
                let c_type = return_type.strip_suffix("[]").unwrap();

                if let Some(c_type) = c_types.get(c_type) {
                    let entries = entries.iter().map(|e| filter(e, c_type)).collect();
                    return Ok(Some(Value::Array {
                        entries,
                        data_type: return_type.to_string(),
                    }));
                }
            }

            Ok(Some(output))
        }
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
                data_type: return_type.to_string(),
                properties,
            }))
        }
        Value::Unit => Ok(None),
        _ => Ok(Some(output)),
    }
}
