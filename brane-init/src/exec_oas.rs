use anyhow::Result;
use specifications::common::Value;
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn handle(
    function: String,
    arguments: Map<Value>,
    _working_dir: PathBuf,
) -> Result<Value> {
    debug!("Executing '{}' (OAS) using arguments:\n{:#?}", function, arguments);

    Ok(Value::Unit)
}