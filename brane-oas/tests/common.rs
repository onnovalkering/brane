use anyhow::Result;
use brane_oas::{build, parse_oas_file, resolve_reference};
use specifications::common::{Function, Type};

type Map<T> = std::collections::HashMap<String, T>;
type FunctionAndTypes = (Function, Map<Type>);

#[allow(dead_code)]
pub fn build_oas_function_body(path: &str) -> Result<FunctionAndTypes> {
    build_oas_function(path, "body.yml")
}

#[allow(dead_code)]
pub fn build_oas_function_param(path: &str) -> Result<FunctionAndTypes> {
    build_oas_function(path, "param.yml")
}

#[allow(dead_code)]
pub fn build_oas_function_resp(path: &str) -> Result<FunctionAndTypes> {
    build_oas_function(path, "resp.yml")
}

///
///
///
fn build_oas_function(path: &str, file: &str) -> Result<FunctionAndTypes> {
    let oas = parse_oas_file(format!("tests/resources/{}", file))?;
    let path_item = resolve_reference(oas.paths.get(path).unwrap())?;
    let (functions, types) = build::build_oas_function(&path_item.get.unwrap())?;

    // Unwrap first (and only) function.
    let function = functions.iter().next().unwrap().1.clone();

    Ok((function, types))
}
