use crate::Payload;
use specifications::common::{Argument, Literal, Type, Value};
use specifications::container::ContainerInfo;
use std::path::PathBuf;
use std::process::Command;
use yaml_rust::{Yaml, YamlLoader};

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub async fn handle(
    container_info: ContainerInfo,
    payload: Payload,
    working_dir: PathBuf,
) -> FResult<Map<Value>> {
    let action = container_info.actions.get(&payload.action);
    ensure!(action.is_some(), "Action '{}' not found.", payload.action);

    // Validate and prepare action execution
    let action = action.unwrap();
    assert_input(&action.input, &payload.arguments)?;
    initialize(&working_dir)?;

    debug!(
        "Executing '{}' using arguments:\n{:#?}",
        payload.action, payload.arguments
    );

    let entrypoint = &container_info.entrypoint.exec;
    let command = action.command.as_ref().unwrap();

    let stdout = execute(entrypoint, &command.args, &payload.arguments, &working_dir)?;
    let output = capture_output(stdout, &action.output, &command.capture, &container_info.types)?;

    Ok(output)
}

///
///
///
fn assert_input(
    parameters: &[Argument],
    arguments: &Map<Value>,
) -> FResult<()> {
    debug!("Asserting input arguments");

    for p in parameters {
        let expected_type = p.data_type.as_str();
        if expected_type.starts_with("mount") {
            continue;
        }

        let argument = arguments.get(&p.name);
        ensure!(argument.is_some(), "Argument not provided: {}", p.name);

        let argument = argument.unwrap();
        let actual_type = argument.get_complex();

        if expected_type != actual_type {
            bail!(
                "Type check for '{}' failed: '{}' is not '{}' or subtype thereof",
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
fn initialize(working_dir: &PathBuf) -> FResult<()> {
    debug!("Initializing working directory");

    let init_sh = working_dir.join("init.sh");
    if !init_sh.exists() {
        return Ok(());
    }

    let result = Command::new(init_sh).output().expect("Couldn't execute init.sh");

    ensure!(result.status.success(), "Non-zero exit status for init.sh");

    Ok(())
}

///
///
///
fn execute(
    entrypoint: &str,
    command_args: &[String],
    arguments: &Map<Value>,
    working_dir: &PathBuf,
) -> FResult<String> {
    let entrypoint_path = working_dir.join(entrypoint).canonicalize()?;
    let mut command = if entrypoint_path.is_file() {
        Command::new(entrypoint_path)
    } else {
        let segments = entrypoint.split_whitespace().collect::<Vec<&str>>();
        let entrypoint_path = working_dir.join(&segments[0]).canonicalize()?;

        let mut command = Command::new(entrypoint_path);
        command.args(segments.iter().skip(1));

        command
    };

    let envs = construct_envs(arguments)?;
    debug!("Using environment variables:\n{:#?}", envs);

    let result = command
        .args(command_args)
        .envs(envs)
        .output()
        .expect("Couldn't execute action");

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
fn construct_envs(variables: &Map<Value>) -> FResult<Map<String>> {
    let mut envs = Map::<String>::new();

    for (name, variable) in variables.iter() {
        let name = name.to_ascii_uppercase();

        use Literal::*;
        match variable {
            Value::Array { entries, .. } => {
                envs.insert(name.clone(), entries.len().to_string());

                for (index, entry) in entries.iter().enumerate() {
                    let value = match entry {
                        Value::Literal(Boolean(value)) => value.to_string(),
                        Value::Literal(Integer(value)) => value.to_string(),
                        Value::Literal(Decimal(value)) => value.to_string(),
                        Value::Literal(Str(value)) => value.to_string(),
                        _ => unreachable!(),
                    };

                    envs.insert(format!("{}_{}", &name, index), value);
                }
            }
            Value::Literal(literal) => {
                let value = match literal {
                    Boolean(value) => value.to_string(),
                    Integer(value) => value.to_string(),
                    Decimal(value) => value.to_string(),
                    Str(value) => value.to_string(),
                };

                envs.insert(name, value.clone());
            }
            _ => unimplemented!(),
        }
    }

    Ok(envs)
}

///
///
///
fn capture_output(
    stdout: String,
    params: &[Argument],
    mode: &Option<String>,
    c_types: &Option<Map<Type>>,
) -> FResult<Map<Value>> {
    let stdout = preprocess_stdout(stdout, &mode)?;
    let docs = YamlLoader::load_from_str(&stdout)?;

    let c_types = c_types.clone().unwrap_or_default();
    let output = unwrap_yaml_hash(&docs[0], params, &c_types)?;

    Ok(output)
}

///
///
///
fn unwrap_yaml_hash(
    value: &Yaml,
    params: &[Argument],
    _types: &Map<Type>,
) -> FResult<Map<Value>> {
    let map = value.as_hash().unwrap();

    let mut output = Map::<Value>::new();
    for p in params {
        let key = Yaml::from_str(p.name.as_str());
        let value = &map[&key];

        let value = match value {
            Yaml::Array(elements) => {
                let n = p.data_type.find('[').unwrap(); // Number of array dimensions
                let value_type: String = p.data_type.chars().take(n).collect();

                let mut entries = vec![];
                for element in elements.iter() {
                    let variable = unwrap_yaml_value(&element, &value_type)?;
                    entries.push(variable);
                }

                let complex = String::from(&p.data_type);
                Value::Array { complex, entries }
            }
            Yaml::Hash(_) => unimplemented!(),
            _ => unwrap_yaml_value(&map[&key], &p.data_type)?,
        };

        output.insert(p.name.clone(), value);
    }

    Ok(output)
}

///
///
///
fn unwrap_yaml_value(
    value: &Yaml,
    data_type: &str,
) -> FResult<Value> {
    debug!("Unwrapping as {}: {:?} ", data_type, value);

    let value = match data_type {
        "boolean" => {
            let value = value.as_bool().unwrap();
            Value::Literal(Literal::Boolean(value))
        }
        "integer" => {
            let value = value.as_i64().unwrap();
            Value::Literal(Literal::Integer(value))
        }
        "real" => {
            let value = value.as_f64().unwrap();
            Value::Literal(Literal::Decimal(value))
        }
        _ => {
            let value = String::from(value.as_str().unwrap());
            Value::Literal(Literal::Str(value))
        }
    };

    Ok(value)
}

const MARK_START: &str = "--> START CAPTURE";
const MARK_END: &str = "--> END CAPTURE";
const PREFIX: &str = "~~>";

///
///
///
fn preprocess_stdout(
    stdout: String,
    mode: &Option<String>,
) -> FResult<String> {
    let mode = mode.clone().unwrap_or_else(|| String::from("complete"));

    if mode == "complete" {
        return Ok(stdout);
    }

    let mut captured = Vec::new();

    if mode == "marked" {
        let mut capture = false;

        for line in stdout.split('\n') {
            if !capture && line.trim_start().starts_with(MARK_START) {
                capture = true;
                continue;
            }

            if capture && line.trim_start().starts_with(MARK_END) {
                break;
            }

            captured.push(line);
        }
    }

    if mode == "prefixed" {
        for line in stdout.split('\n') {
            if line.starts_with(PREFIX) {
                captured.push(line.trim_start_matches(PREFIX));
            }
        }
    }

    Ok(captured.join("\n"))
}
