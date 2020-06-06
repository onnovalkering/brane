use crate::packages;
use openapiv3::OpenAPI;
use openapiv3::{Operation, Parameter as OParameter, SchemaKind, ParameterSchemaOrContent, ReferenceOr, Schema, Type as OType};
use serde_yaml;
use specifications::common::{Function, Parameter, Property, Type};
use specifications::package::PackageInfo;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn handle(
    context: PathBuf,
    file: PathBuf,
) -> FResult<()> {
    let oas_file = context.join(file);
    let oas_reader = BufReader::new(File::open(&oas_file)?);
    let oas_document: OpenAPI = serde_yaml::from_reader(oas_reader)?;

    // Prepare package directory
    let package_info = create_package_info(&oas_document)?;
    let package_dir = packages::get_package_dir(&package_info.name, Some(&package_info.version))?;
    prepare_directory(&oas_document, &oas_file, &package_info, &package_dir)?;

    Ok(())
}

///
///
///
fn create_package_info(oas_document: &OpenAPI) -> FResult<PackageInfo> {
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
fn build_oas_functions(oas_document: &OpenAPI) -> FResult<(Map<Function>, Map<Type>)> {
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
            bail!("References to paths are not supported.");
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
) -> FResult<()> {
    let name = if let Some(operation_id) = &operation.operation_id {
        operation_id.replace(" ", "-")
    } else {
        bail!("Please add an operationId to each operation.")
    };

    let mut input_properties = Vec::<Property>::new();
    for parameter in &operation.parameters {
        let parameter = if let ReferenceOr::Item(parameter) = parameter {
            parameter
        } else {
            bail!("Unsupported?");
        };

        let parameter_data = match parameter {
            OParameter::Query { parameter_data, .. } => parameter_data,
            OParameter::Header { parameter_data, .. } => parameter_data,
            OParameter::Path { parameter_data, .. } => parameter_data,
            OParameter::Cookie { parameter_data, .. } => parameter_data,
        };

        let name = parameter_data.name.clone();
        let optional = !parameter_data.required;
        let data_type = if let ParameterSchemaOrContent::Schema(ReferenceOr::Item(schema)) = &parameter_data.format {
            match &schema.schema_kind {
                SchemaKind::Type(data_type) => {
                    match data_type {
                        OType::String(_) => String::from("string"),
                        OType::Number(_) => String::from("real"),
                        OType::Integer(_) => String::from("integer"),
                        OType::Boolean { } => String::from("boolean"),
                        _ => unimplemented!(),
                    }
                }
                _ => bail!("Unsupported schema kind.")
            }
        } else {
            bail!("Unsupported paramter format.");
        };

        let property = Property::new(name, data_type, None, None, Some(optional), None);
        input_properties.push(property);
    }

    let mut output_properties = Vec::<Property>::new();
    for (_, response) in operation.responses.responses.iter() {
        let response = if let ReferenceOr::Item(response) = response {
            response
        } else {
            unimplemented!();
        };

        for (_, content) in response.content.iter() {
            if let Some(ReferenceOr::Item(schema)) = &content.schema {
                let properties = schema_to_properties(None, schema)?;
                output_properties.extend(properties);
            } else {
                unimplemented!();
            };
        }
    }

    let common_type_name = uppercase_first_letter(&name.replace("-", ""));

    let input_data_type = format!("{}Input", common_type_name);
    let input_type = Type { name: input_data_type.clone(), properties: input_properties };
    types.insert(input_data_type.clone(), input_type);

    let output_data_type = format!("{}Output", common_type_name);
    let output_type = Type { name: output_data_type.clone(), properties: output_properties };

    types.insert(output_data_type.clone(), output_type);

    let input_parameter = Parameter::new(String::from("input"), input_data_type, None, None);
    let function = Function::new(vec![input_parameter], None, output_data_type);
    functions.insert(name.to_lowercase(), function);

    Ok(())
}

///
///
///
fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

///
///
///
fn schema_to_properties(name: Option<String>, schema: &Schema) -> FResult<Vec<Property>> {
    let properties = match &schema.schema_kind {
        SchemaKind::Type(data_type) => {
            match data_type {
                OType::Array(_) => unimplemented!(),
                OType::Object(object) => {
                    let mut properties = Vec::<Property>::new();
                    for (name, p_schema) in object.properties.iter() {
                        if let ReferenceOr::Item(p_schema) = p_schema {
                            let props = schema_to_properties(Some(name.clone()), p_schema)?;
                            properties.extend(props);
                        }
                    }

                    properties
                },
                _ => {
                    let name = name.expect("Invalid");
                    let data_type = match data_type {
                        OType::String(_) => String::from("string"),
                        OType::Number(_) => String::from("real"),
                        OType::Integer(_) => String::from("integer"),
                        OType::Boolean { } => String::from("boolean"),
                        _ => unimplemented!(),
                    };

                    vec![Property::new(name, data_type, None, None, None, None)]
                }
            }
        }
        _ => bail!("Unsupported schema kind.")
    };

    Ok(properties)
}

///
///
///
fn prepare_directory(
    _oas_document: &OpenAPI,
    oas_file: &PathBuf,
    package_info: &PackageInfo,
    package_dir: &PathBuf,
) -> FResult<()> {
    fs::create_dir_all(&package_dir)?;

    // Copy OAS document to package directory
    fs::copy(oas_file, package_dir.join("document.yml"))?;

    // Write package.yml to package directory
    let mut buffer = File::create(package_dir.join("package.yml"))?;
    write!(buffer, "{}", serde_yaml::to_string(&package_info)?)?;

    Ok(())
}
