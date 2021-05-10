use super::*;
use crate::resolver::{self, resolve_schema};
use anyhow::Result;
use openapiv3::{OpenAPI, ReferenceOr, SecurityScheme};
use openapiv3::{Components, Parameter as OParameter, Type as OType};
use openapiv3::{Operation, ParameterSchemaOrContent, Schema, SchemaKind};
use specifications::common::{CallPattern, Function, Parameter, Property, Type};

type Map<T> = std::collections::HashMap<String, T>;
type FunctionsAndTypes = (Map<Function>, Map<Type>);

const OAS_ADD_OPERATION_ID: &str = "Please add an operation ID (operationId) to each operation.";
const OAS_CONTENT_NOT_SUPPORTED: &str = "OpenAPI parameter content mapping is not supported.";
const OAS_JSON_MEDIA_NOT_FOUND: &str = "JSON media type not found (application/json).";
const OAS_NESTED_OBJECTS_NOT_SUPPORTED: &str = "Nested objects are not supported.";

/// Traverses a valid OpenAPI document and builds a function
/// for every operation it finds. Corresponding input/output
/// types, if any, are returned as well.
pub fn build_oas_functions(oas_document: &OpenAPI) -> Result<FunctionsAndTypes> {
    let mut functions = Map::<Function>::new();
    let mut types = Map::<Type>::new();

    // Wrap building into re-usable (mutable) closure.
    let mut try_build = |generated_id, operation, components| -> Result<()> {
        if let Some(ref o) = operation {
            let operation_id = get_operation_id(o, Some(generated_id))?;
            let (f, t) = build_oas_function(operation_id, o, components)?;

            // Bookkeeping
            functions.extend(f);
            types.extend(t);
        }

        Ok(())
    };

    // Traverse the OpenAPI document (path items).
    let components = oas_document.components.clone();
    for (url_path, path) in oas_document.paths.iter() {
        let path = resolver::resolve_path_item(path)?;

        try_build(generate_operation_id("delete", url_path), path.delete, &components)?;
        try_build(generate_operation_id("get", url_path), path.get, &components)?;
        try_build(generate_operation_id("head", url_path), path.head, &components)?;
        try_build(generate_operation_id("options", url_path), path.options, &components)?;
        try_build(generate_operation_id("patch", url_path), path.patch, &components)?;
        try_build(generate_operation_id("post", url_path), path.post, &components)?;
        try_build(generate_operation_id("put", url_path), path.put, &components)?;
        try_build(generate_operation_id("trace", url_path), path.trace, &components)?;
    }

    Ok((functions, types))
}

/// Generates an identifier for a OpenAPI operation.
pub fn generate_operation_id(
    method: &str,
    path: &String,
) -> String {
    let mut operation_id = method.to_string();

    let segments = path.split("/");
    for segment in segments {
        if segment.is_empty() {
            continue;
        }

        // Trim { } indicating variable placeholders.
        let trimmed = segment.trim_matches(|c| c == '{' || c == '}');
        let segment = if segment == trimmed {
            format!("_{}", segment)
        } else {
            format!("_by{}", trimmed)
        };

        operation_id.push_str(&segment);
    }

    debug!(
        "Generated ID {} based on {} ({}).",
        operation_id,
        path,
        method.to_uppercase()
    );

    operation_id
}

/// Gets and validates the identifier of an OpenAPI operation.
pub fn get_operation_id(
    operation: &Operation,
    fallback: Option<String>,
) -> Result<String> {
    let operation_id = if let Some(operation_id) = &operation.operation_id {
        operation_id.clone()
    } else {
        if let Some(fallback) = fallback {
            fallback
        } else {
            bail!(OAS_ADD_OPERATION_ID);
        }
    };

    // The identifier must be a valid Bakery / BraneScript function name.
    ensure!(
        operation_id.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "Invalid operation ID. Must consist only of alphanumeric and/or _ characters."
    );

    Ok(operation_id)
}

/// Builds a function for a OpenAPI operation. Corresponding
/// input/output types, if any, are returned as well.
pub fn build_oas_function(
    operation_id: String,
    operation: &Operation,
    components: &Option<Components>,
) -> Result<FunctionsAndTypes> {
    let (input, i_types) = build_oas_function_input(&operation_id, operation, components)?;
    let (output, o_types) = build_oas_function_output(&operation_id, operation, components)?;

    // Build function
    let name = operation_id.to_lowercase();
    let call_pattern = CallPattern::new(Some(name.clone()), None, None);
    let functions = hashmap! {
        name => Function::new(input, Some(call_pattern), output)
    };

    // Combine input and output types
    let mut types = Map::<Type>::new();
    types.extend(i_types);
    types.extend(o_types);

    Ok((functions, types))
}

///
///
///
fn build_oas_function_input(
    operation_id: &str,
    operation: &Operation,
    components: &Option<Components>,
) -> Result<(Vec<Parameter>, Map<Type>)> {
    let mut input_properties = Vec::<Property>::new();
    let mut input_types = Map::<Type>::new();

    // Determine input from paramaters.
    for parameter in &operation.parameters {
        let parameter = resolver::resolve_parameter(parameter, components)?;
        let mut properties = parameter_to_properties(&parameter, components, &mut input_types)?;

        input_properties.append(&mut properties);
    }

    // Determine input from security schemes.
    if let Some(security_scheme) = &operation.security.iter().next() {
        if let Some(security_scheme) = security_scheme.keys().next() {
            let item = ReferenceOr::Reference::<SecurityScheme> {
                reference: format!("#/components/schemas/{}", security_scheme)
            };

            let security_scheme = resolver::resolve_security_scheme(&item, components)?;
            let property = match security_scheme {
                SecurityScheme::APIKey { name, .. } => {
                    Property::new_quick(&name, "string")
                },
                SecurityScheme::HTTP { .. } => {
                    Property::new_quick("token", "string")
                },
                _ => todo!(),
            };

            input_properties.push(property);
        }
    }

    // Determine input from request body.
    if let Some(request_body) = &operation.request_body {
        let request_body = resolver::resolve_request_body(request_body, components)?;

        // Only 'application/json' request bodies are supported
        if let Some(content) = request_body.content.get("application/json") {
            if let Some(schema) = &content.schema {
                let (_, schema) = resolver::resolve_schema(schema, components)?;

                let optional = false; // check if is in required list
                let properties = schema_to_properties(None, &schema, optional, components, &mut input_types)?;

                input_properties.extend(properties);
            }
        } else {
            return Err(anyhow!(OAS_JSON_MEDIA_NOT_FOUND));
        }
    }

    // Convert input properties to parameters.
    let input_parameters = if input_properties.len() > 3 {
        let type_name = uppercase_first_letter(&operation_id);
        let input_data_type = format!("{}Input", type_name);

        let input_type = Type {
            name: input_data_type.clone(),
            properties: input_properties,
        };
        input_types.insert(input_data_type.clone(), input_type);

        vec![Parameter::new(String::from("input"), input_data_type, None, None, None)]
    } else {
        input_properties
            .iter()
            .map(|p| p.clone().into_parameter())
            .collect::<Vec<Parameter>>()
    };

    Ok((input_parameters, input_types))
}

///
///
///
fn build_oas_function_output(
    operation_id: &str,
    operation: &Operation,
    components: &Option<Components>,
) -> Result<(String, Map<Type>)> {
    let mut output_properties = Vec::<Property>::new();
    let mut output_types = Map::<Type>::new();

    // Construct output properties
    let response = if let Some(default) = &operation.responses.default {
        resolver::resolve_response(default, components)?
    } else {
        let responses = &operation.responses.responses;
        if let Some(response) = responses.values().next() {
            resolver::resolve_response(response, components)?
        } else {
            unreachable!()
        }
    };

    // Only 'application/json' responses are supported
    if let Some(content) = response.content.get("application/json") {
        if let Some(schema) = &content.schema {
            let (_, schema) = resolver::resolve_schema(schema, components)?;
            let required = true; // check if is in required list
            let properties = schema_to_properties(None, &schema, required, components, &mut output_types)?;

            output_properties.extend(properties);
        }
    } else {
        return Err(anyhow!(OAS_JSON_MEDIA_NOT_FOUND));
    }

    // Convert output properties to return type
    let return_type = if output_properties.len() > 1 {
        let type_name = uppercase_first_letter(&operation_id);
        let output_data_type = format!("{}Output", type_name);

        let output_type = Type {
            name: output_data_type.clone(),
            properties: output_properties,
        };

        output_types.insert(output_data_type.clone(), output_type);
        output_data_type
    } else if let Some(output_property) = output_properties.first() {
        output_property.data_type.clone()
    } else {
        String::from("unit")
    };

    Ok((return_type, output_types))
}

///
///
///
fn parameter_to_properties(
    parameter: &OParameter,
    components: &Option<Components>,
    types: &mut Map::<Type>,
) -> Result<Vec<Property>> {
    // Get inner parameter object.
    let parameter_data = match parameter {
        OParameter::Query { parameter_data, .. } => parameter_data,
        OParameter::Header { parameter_data, .. } => parameter_data,
        OParameter::Path { parameter_data, .. } => parameter_data,
        OParameter::Cookie { parameter_data, .. } => parameter_data,
    };

    let name = Some(parameter_data.name.clone());
    let required = parameter_data.required;
    match &parameter_data.format {
        ParameterSchemaOrContent::Schema(schema) => {
            let (_, schema) = resolver::resolve_schema(schema, components)?;
            schema_to_properties(name, &schema, required, components, types)
        }
        ParameterSchemaOrContent::Content(_) => Err(anyhow!(OAS_CONTENT_NOT_SUPPORTED)),
    }
}

///
///
///
pub fn schema_to_properties(
    name: Option<String>,
    schema: &Schema,
    required: bool,
    components: &Option<Components>,
    types: &mut Map::<Type>,
) -> Result<Vec<Property>> {
    match schema.schema_kind {
        SchemaKind::Any(_) => any_schema_to_properties(name, schema, required, components, types),
        SchemaKind::Type(_) => type_schema_to_properties(name, schema, required, components, types),
        _ => todo!(),
    }
}

///
///
///
fn any_schema_to_properties(
    _name: Option<String>,
    schema: &Schema,
    required: bool,
    components: &Option<Components>,
    types: &mut Map::<Type>,
) -> Result<Vec<Property>> {
    let any_schema = if let SchemaKind::Any(any_schema) = &schema.schema_kind {
        any_schema
    } else {
        unreachable!()
    };

    let mut properties = vec![];
    for (name, property) in any_schema.properties.iter() {
        let property = property.clone().unbox();
        let (_, schema) = resolve_schema(&property, components)?;

        let props = schema_to_properties(Some(name.clone()), &schema, required, components, types)?;
        properties.extend(props);
    }

    Ok(properties)
}

///
///
///
fn type_schema_to_properties(
    name: Option<String>,
    schema: &Schema,
    required: bool,
    components: &Option<Components>,
    types: &mut Map::<Type>,
) -> Result<Vec<Property>> {
    let data_type = if let SchemaKind::Type(data_type) = &schema.schema_kind {
        data_type
    } else {
        unreachable!()
    };

    let properties = match data_type {
        OType::Array(array) => {
            let items = array.items.clone().unbox();
            let (ref_name, items_schema) = resolver::resolve_schema(&items, components)?;

            let data_type = match &items_schema.schema_kind {
                SchemaKind::Type(OType::String(_)) => String::from("string[]"),
                SchemaKind::Type(OType::Number(_)) => String::from("real[]"),
                SchemaKind::Type(OType::Integer(_)) => String::from("integer[]"),
                SchemaKind::Type(OType::Boolean {}) => String::from("boolean[]"),
                SchemaKind::Any(_) => {
                    let item_type_properties = any_schema_to_properties(None, &items_schema, required, components, types)?;
                    let item_type_name = if let Some(ref_name) = ref_name {
                        ref_name.clone()
                    } else {
                        String::from("todogenerate")
                    };

                    let item_type = Type {
                        name: item_type_name.clone(),
                        properties: item_type_properties,
                    };

                    types.insert(item_type_name.clone(), item_type);

                    format!("{}[]", item_type_name)
                },
                _ => todo!()
            };

            vec![Property::new(
                name.unwrap_or_default(),
                data_type,
                None,
                None,
                Some(!required),
                None,
            )]
        }
        OType::Object(object) => {
            ensure!(name.is_none(), OAS_NESTED_OBJECTS_NOT_SUPPORTED);

            let mut properties = Vec::<Property>::new();
            for (name, p_schema) in object.properties.iter() {
                let p_schema = p_schema.clone().unbox();
                let (_, p_schema) = resolver::resolve_schema(&p_schema, components)?;
                let props = schema_to_properties(Some(name.clone()), &p_schema, required, components, types)?;

                properties.extend(props);
            }

            properties
        }
        _ => {
            let data_type = match data_type {
                OType::String(_) => String::from("string"),
                OType::Number(_) => String::from("real"),
                OType::Integer(_) => String::from("integer"),
                OType::Boolean {} => String::from("boolean"),
                _ => unreachable!(),
            };

            vec![Property::new(
                name.unwrap_or_default(),
                data_type,
                None,
                None,
                Some(!required),
                None,
            )]
        }
    };

    Ok(properties)
}

/// Utility to capitalize the first letter.
fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
