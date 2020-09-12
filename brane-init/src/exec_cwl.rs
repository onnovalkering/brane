use anyhow::Result;
use specifications::common::{Parameter, Value, Type};
use specifications::package::PackageInfo;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use serde_json::Value as JValue;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn handle(
    function: String,
    arguments: Map<Value>,
    working_dir: PathBuf,
) -> Result<Value> {
    debug!("Executing '{}' (CWL) using arguments:\n{:#?}", function, arguments);

    let package_info = PackageInfo::from_path(working_dir.join("package.yml"))?;
    let functions = package_info.functions.expect("Missing `functions` property in package.yml");
    let function = functions.get(&function).expect(&format!("Function '{}' not found", function));

    assert_input(&function.parameters, &arguments)?;
    initialize(&arguments, &working_dir)?;

    // Output variables are captured from the stdout
    let stdout = execute(&working_dir)?;
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
        let argument = arguments.get(&p.name);
        ensure!(argument.is_some(), "Argument not provided: {}", p.name);

        let argument = argument.unwrap();
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
    arguments: &Map<Value>,
    working_dir: &PathBuf
) -> Result<()> {
    let mut input = Map::<JValue>::new();
    for (name, value) in arguments.iter() {
        input.insert(name.clone(), value.as_json());
    }

    let input_path = working_dir.join("input.json");
    let mut input_file = File::create(input_path)?;
    writeln!(input_file, "{}", &serde_json::to_string(&input)?)?;

    Ok(())
}

///
///
///
fn execute(
    working_dir: &PathBuf,
) -> Result<String> {
    let result = Command::new("cwltool")
        .args(vec!["--quiet", "document.cwl", "input.json"])
        .current_dir(&working_dir)
        .output()
        .expect("Couldn't execute cwltool.");

    let stdout = String::from(String::from_utf8_lossy(&result.stdout));
    let stderr = String::from(String::from_utf8_lossy(&result.stderr));
    
    debug!("stdout:\n{}", &stdout);
    debug!("stderr:\n{}", &stderr);

    ensure!(result.status.success(), "Non-zero exit status for action");
    
    Ok(stdout)
}

///
///
///
fn capture_output(
    stdout: &String,
    return_type: &String,
    _c_types: &Option<Map<Type>>,
) -> Result<Option<Value>> {
    let output: Map<JValue> = serde_json::from_str(&stdout)?;
    if let Some((_, value)) = output.iter().next() {
        Ok(Some(match return_type.as_str() {
            "File" => {
                let location = value["location"].as_str().unwrap().to_string();

                let mut properties = Map::<Value>::new();
                properties.insert("url".to_string(), Value::Unicode(location));
                
                Value::Struct {
                    data_type: String::from("File"),
                    properties
                }
            },
            "string" => {
                let mut value_file = File::open(value["path"].as_str().unwrap())?;
                let mut value_content = String::new();
                value_file.read_to_string(&mut value_content)?;
                
                let value_trimmed = value_content.trim().to_string();
                Value::Unicode(value_trimmed.to_string())
            },
            _ => unimplemented!()
        }))
    } else {
        Ok(None)
    }
}