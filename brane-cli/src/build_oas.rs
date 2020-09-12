use crate::{packages, utils};
use anyhow::{Context, Result};
use console::style;
use openapiv3::OpenAPI;
use openapiv3::{Operation, ParameterSchemaOrContent, ReferenceOr, Schema, SchemaKind};
use openapiv3::{Parameter as OParameter, Type as OType};
use serde_yaml;
use specifications::common::{CallPattern, Function, Parameter, Property, Type};
use specifications::package::PackageInfo;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::Command;

type Map<T> = std::collections::HashMap<String, T>;

const INIT_URL: &str = concat!(
    "https://github.com/onnovalkering/brane/releases/download/",
    concat!("v", env!("CARGO_PKG_VERSION")),
    "/brane-init"
);

const OAS_ADD_OPERATION_ID: &str = "Please add an operation ID (operationId) to each operation.";
const OAS_CONTENT_NOT_SUPPORTED: &str = "OpenAPI parameter content mapping is not supported.";
const OAS_JSON_MEDIA_NOT_FOUND: &str = "JSON media type not found (application/json).";
const OAS_MULTIPLE_NOT_SUPPORTED: &str = "Multiple responses per operation is not supported.";
const OAS_REFS_NOT_SUPPORTED: &str = "OpenAPI references are not (yet) supported.";
const OAS_SCHEMA_NOT_SUPPORTED: &str = "Only type schemas are supported.";

///
///
///
pub fn handle(
    context: PathBuf,
    file: PathBuf,
    init_path: Option<PathBuf>,
) -> Result<()> {
    let context = fs::canonicalize(context)?;
    debug!("Using {:?} as build context", context);

    let oas_file = context.join(file);
    let oas_reader = BufReader::new(File::open(&oas_file)?);
    let oas_document: OpenAPI = serde_yaml::from_reader(oas_reader)?;

    // Prepare package directory
    let dockerfile = generate_dockerfile(&oas_document, init_path.is_some())?;
    let package_info = create_package_info(&oas_document)?;
    let package_dir = packages::get_package_dir(&package_info.name, Some(&package_info.version))?;
    prepare_directory(&oas_file, dockerfile, init_path, &package_info, &package_dir)?;

    debug!("Successfully prepared package directory.");

    // Build Docker image
    let tag = format!("{}:{}", package_info.name, package_info.version);
    build_docker_image(&package_dir, tag)?;

    println!(
        "Successfully built version {} of OAS package {}.",
        style(&package_info.version).bold().cyan(),
        style(&package_info.name).bold().cyan(),
    );

    Ok(())
}

///
///
///
fn create_package_info(oas_document: &OpenAPI) -> Result<PackageInfo> {
    let name = oas_document.info.title.to_lowercase().replace(" ", "-");
    let version = oas_document.info.version.clone();
    let description = oas_document.info.description.clone();

    let (functions, types) = build_oas_functions(&oas_document)?;

    let package_info = PackageInfo::new(
        name,
        version,
        description,
        String::from("oas"),
        Some(functions),
        Some(types),
    );

    Ok(package_info)
}

///
///
///
fn build_oas_functions(oas_document: &OpenAPI) -> Result<(Map<Function>, Map<Type>)> {
    let mut functions = Map::<Function>::new();
    let mut types = Map::<Type>::new();

    for (_, path) in oas_document.paths.iter() {
        if let ReferenceOr::Item(path) = path {
            if let Some(delete) = &path.delete {
                build_oas_function(delete, &mut functions, &mut types)?;
            }
            if let Some(get) = &path.get {
                build_oas_function(get, &mut functions, &mut types)?;
            }
            if let Some(patch) = &path.patch {
                build_oas_function(patch, &mut functions, &mut types)?;
            }
            if let Some(post) = &path.post {
                build_oas_function(post, &mut functions, &mut types)?;
            }
            if let Some(put) = &path.put {
                build_oas_function(put, &mut functions, &mut types)?;
            }
        } else {
            return Err(anyhow!(OAS_REFS_NOT_SUPPORTED));
        }
    }

    Ok((functions, types))
}

///
///
///
fn build_oas_function(
    operation: &Operation,
    functions: &mut Map<Function>,
    types: &mut Map<Type>,
) -> Result<()> {
    let name = if let Some(operation_id) = &operation.operation_id {
        utils::assert_valid_bakery_name(operation_id)
            .with_context(|| format!("Operation ID '{}' is not valid.", operation_id))?;

        operation_id
    } else {
        return Err(anyhow!(OAS_ADD_OPERATION_ID));
    };

    let type_name = utils::uppercase_first_letter(&name);
    let mut input_properties = Vec::<Property>::new();
    let mut output_properties = Vec::<Property>::new();

    // Construct input properties
    for parameter in &operation.parameters {
        let parameter = if let ReferenceOr::Item(parameter) = parameter {
            parameter
        } else {
            return Err(anyhow!(OAS_REFS_NOT_SUPPORTED));
        };

        // Get inner parameter object
        let parameter_data = match parameter {
            OParameter::Query { parameter_data, .. } => parameter_data,
            OParameter::Header { parameter_data, .. } => parameter_data,
            OParameter::Path { parameter_data, .. } => parameter_data,
            OParameter::Cookie { parameter_data, .. } => parameter_data,
        };

        let name = Some(parameter_data.name.clone());
        let required = parameter_data.required;
        let mut properties = match &parameter_data.format {
            ParameterSchemaOrContent::Schema(schema) => {
                if let ReferenceOr::Item(schema) = schema {
                    schema_to_properties(name, schema, required)?
                } else {
                    return Err(anyhow!(OAS_REFS_NOT_SUPPORTED));
                }
            }
            ParameterSchemaOrContent::Content(_) => {
                return Err(anyhow!(OAS_CONTENT_NOT_SUPPORTED));
            }
        };

        input_properties.append(&mut properties);
    }

    // Construct output properties
    let response = if let Some(default) = &operation.responses.default {
        if let ReferenceOr::Item(default) = default {
            default
        } else {
            return Err(anyhow!(OAS_REFS_NOT_SUPPORTED));
        }
    } else {
        let responses = &operation.responses.responses;
        if responses.len() != 1 {
            return Err(anyhow!(OAS_MULTIPLE_NOT_SUPPORTED));
        }

        if let Some(response) = responses.values().next() {
            if let ReferenceOr::Item(response) = response {
                response
            } else {
                return Err(anyhow!(OAS_REFS_NOT_SUPPORTED));
            }
        } else {
            unreachable!()
        }
    };

    // Only 'application/json' responses are supported
    if let Some(content) = response.content.get("application/json") {
        if let Some(ReferenceOr::Item(schema)) = &content.schema {
            let optional = false; // check if is in required list
            let properties = schema_to_properties(None, schema, optional)?;
            output_properties.extend(properties);
        } else {
            return Err(anyhow!(OAS_REFS_NOT_SUPPORTED));
        };
    } else {
        return Err(anyhow!(OAS_JSON_MEDIA_NOT_FOUND));
    }

    // Convert input properties to parameters
    let input_parameters = if input_properties.len() > 3 {
        let input_data_type = format!("{}Input", type_name);
        let input_type = Type {
            name: input_data_type.clone(),
            properties: input_properties,
        };
        types.insert(input_data_type.clone(), input_type);

        let input_parameter = Parameter::new(String::from("input"), input_data_type, None, None, None);
        vec![input_parameter]
    } else {
        input_properties
            .iter()
            .map(|p| p.clone().into_parameter())
            .collect::<Vec<Parameter>>()
    };

    // Convert output properties to return type
    let return_type = if output_properties.len() > 1 {
        let output_data_type = format!("{}Output", type_name);
        let output_type = Type {
            name: output_data_type.clone(),
            properties: output_properties,
        };

        types.insert(output_data_type.clone(), output_type);
        output_data_type
    } else {
        if let Some(output_property) = output_properties.first() {
            output_property.data_type.clone()
        } else {
            String::from("unit")
        }
    };

    // Construct function
    let call_pattern = CallPattern::new(Some(name.to_lowercase()), None, None);
    let function = Function::new(input_parameters, Some(call_pattern), return_type);
    functions.insert(name.to_lowercase(), function);

    Ok(())
}

///
///
///
fn schema_to_properties(
    name: Option<String>,
    schema: &Schema,
    required: bool,
) -> Result<Vec<Property>> {
    let data_type = if let SchemaKind::Type(data_type) = &schema.schema_kind {
        data_type
    } else {
        return Err(anyhow!(OAS_SCHEMA_NOT_SUPPORTED));
    };

    let properties = match data_type {
        OType::Array(_) => unimplemented!(),
        OType::Object(object) => {
            let mut properties = Vec::<Property>::new();
            for (name, p_schema) in object.properties.iter() {
                if let ReferenceOr::Item(p_schema) = p_schema {
                    let props = schema_to_properties(Some(name.clone()), p_schema, required)?;
                    properties.extend(props);
                }
            }

            properties
        }
        _ => {
            let name = name.expect("Invalid");
            let data_type = match data_type {
                OType::String(_) => String::from("string"),
                OType::Number(_) => String::from("real"),
                OType::Integer(_) => String::from("integer"),
                OType::Boolean {} => String::from("boolean"),
                _ => unimplemented!(),
            };

            vec![Property::new(name, data_type, None, None, Some(!required), None)]
        }
    };

    Ok(properties)
}

///
///
///
fn generate_dockerfile(
    _oas_document: &OpenAPI,
    override_init: bool,
) -> Result<String> {
    let mut contents = String::new();

    // Add default heading
    writeln!(contents, "# Generated by Brane")?;
    writeln!(contents, "FROM ubuntu:20.04")?;
    
    // Add dependencies
    writeln!(contents, "RUN apt-get update && apt-get install -y curl")?;

    // Add Brane's init executable
    if override_init {
        writeln!(contents, "ADD init init")?;
    } else {
        writeln!(contents, "ADD {} init", INIT_URL)?;
        writeln!(contents, "RUN chmod +x init")?;
    }

    // Copy files
    writeln!(contents, "ADD wd.tar.gz /opt")?;
    writeln!(contents, "WORKDIR /opt/wd")?;

    writeln!(contents, "WORKDIR /")?;
    writeln!(contents, "ENTRYPOINT [\"./init\"]")?;

    Ok(contents)
}

///
///
///
fn prepare_directory(
    oas_file: &PathBuf,
    dockerfile: String,
    init_path: Option<PathBuf>,
    package_info: &PackageInfo,
    package_dir: &PathBuf,
) -> Result<()> {
    fs::create_dir_all(&package_dir)?;
    debug!("Created {:?} as package directory", package_dir);

    // Write Dockerfile to package directory
    let mut buffer = File::create(package_dir.join("Dockerfile"))?;
    write!(buffer, "{}", dockerfile)?;

    // Write package.yml to package directory
    let mut buffer = File::create(package_dir.join("package.yml"))?;
    write!(buffer, "{}", serde_yaml::to_string(&package_info)?)?;

    // Copy custom init binary to package directory
    if let Some(init_path) = init_path {
        fs::copy(fs::canonicalize(init_path)?, package_dir.join("init"))?;
    }

    // Create the working directory and copy required files.
    let wd = package_dir.join("wd");
    if !wd.exists() {
        fs::create_dir(&wd)?;
    }

    // Always copy these two files, required by convention
    fs::copy(oas_file, wd.join("document.yml"))?;
    fs::copy(package_dir.join("package.yml"), wd.join("package.yml"))?;

    // Archive the working directory and remove the original.
    let output = Command::new("tar")
        .arg("-zcf")
        .arg("wd.tar.gz")
        .arg("wd")
        .current_dir(&package_dir)
        .output()
        .expect("Couldn't run 'tar' command.");

    if !output.status.success() {
        return Err(anyhow!("Failed to prepare working directory archive."));
    }

    let output = Command::new("rm")
        .arg("-rf")
        .arg("wd")
        .current_dir(&package_dir)
        .output()
        .expect("Couldn't run 'rm' command.");

    if !output.status.success() {
        warn!("Failed to cleanup working directory.");
    }
    
    Ok(())
}

///
///
///
fn build_docker_image(
    package_dir: &PathBuf,
    tag: String,
) -> Result<()> {
    let buildx = Command::new("docker")
        .arg("buildx")
        .output()
        .expect("Couldn't run 'docker' command.");

    if !buildx.status.success() {
        return Err(anyhow!("Failed to build Docker image. Is BuildKit enabled (see documentation)?"));
    }

    let output = Command::new("docker")
        .arg("buildx")
        .arg("build")
        .arg("--output")
        .arg("type=docker,dest=image.tar")
        .arg("--tag")
        .arg(tag)
        .arg(".")
        .current_dir(&package_dir)
        .status()
        .expect("Couldn't run 'docker' command.");

    if !output.success() {
        return Err(anyhow!("Failed to build Docker image. See Docker output above for more information."));
    }

    Ok(())
}
