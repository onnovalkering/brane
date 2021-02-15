use anyhow::Result;
use serde_json::Value as JValue;
use specifications::common::{Parameter, Type, Value};
use specifications::package::PackageInfo;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn handle(
    function: String,
    arguments: Map<Value>,
    working_dir: PathBuf,
    output_dir: Option<PathBuf>,
) -> Result<Value> {
    debug!("Executing '{}' (CWL) using arguments:\n{:#?}", function, arguments);

    let package_info = PackageInfo::from_path(working_dir.join("package.yml"))?;
    let functions = package_info
        .functions
        .expect("Missing `functions` property in package.yml");
    let function = functions
        .get(&function)
        .expect(&format!("Function '{}' not found", function));

    assert_input(&function.parameters, &arguments)?;
    initialize(&arguments, &working_dir)?;

    // Output variables are captured from the stdout
    let stdout = execute(&working_dir, output_dir)?;
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
    working_dir: &PathBuf,
) -> Result<()> {
    let mut input = Map::<JValue>::new();
    if let Some(Value::Struct { properties, .. }) = arguments.get("input") {
        for (name, value) in properties.iter() {
            input.insert(name.clone(), value.as_json());
        }
    } else {
        for (name, value) in arguments.iter() {
            input.insert(name.clone(), value.as_json());
        }
    };

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
    output_dir: Option<PathBuf>,
) -> Result<String> {
    let output_dir = output_dir.unwrap_or(working_dir.clone());
    let output_dir = output_dir.as_os_str().to_string_lossy();

    let result = Command::new("cwltool")
        .args(vec!["--quiet", "--outdir", &output_dir, "document.cwl", "input.json"])
        .current_dir(&working_dir)
        .output()
        .expect("Couldn't execute cwltool.");

    let stdout = String::from(String::from_utf8_lossy(&result.stdout));
    let stderr = String::from(String::from_utf8_lossy(&result.stderr));

    debug!("stdout:\n{}\n", &stdout);
    debug!("stderr:\n{}\n", &stderr);

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
    let outputs: Map<JValue> = serde_json::from_str(&stdout)?;
    debug!("output: {:#?}", outputs);

    if outputs.len() == 0 {
        return Ok(None);
    }

    let mut output_properties: Map<Value> = Default::default();
    for (name, output) in outputs {
        if output.is_null() {
            continue;
        }

        let data_type = output["class"]
            .as_str()
            .expect("Missing `class` property on CWL output parameter.");
        match data_type {
            "File" => {
                let location = output["location"].as_str().unwrap().to_string();

                let mut properties: Map<Value> = Default::default();
                properties.insert("url".to_string(), Value::Unicode(location));

                let value = Value::Struct {
                    data_type: String::from("File"),
                    properties,
                };

                output_properties.insert(name, value);
            }
            _ => unimplemented!(),
        }
    }

    Ok(Some(Value::Struct {
        data_type: return_type.clone(),
        properties: output_properties,
    }))
}
