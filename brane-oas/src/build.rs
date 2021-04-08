use super::resolve_reference;
use anyhow::Result;
use openapiv3::OpenAPI;
use openapiv3::{Operation, ParameterSchemaOrContent, Schema, SchemaKind};
use openapiv3::{Parameter as OParameter, Type as OType};
use specifications::common::{CallPattern, Function, Parameter, Property, Type};

type Map<T> = std::collections::HashMap<String, T>;
type FunctionsAndTypes = (Map<Function>, Map<Type>);

const OAS_ADD_OPERATION_ID: &str = "Please add an operation ID (operationId) to each operation.";
const OAS_CONTENT_NOT_SUPPORTED: &str = "OpenAPI parameter content mapping is not supported.";
const OAS_JSON_MEDIA_NOT_FOUND: &str = "JSON media type not found (application/json).";
const OAS_MULTIPLE_NOT_SUPPORTED: &str = "Multiple responses per operation is not supported.";
const OAS_SCHEMA_NOT_SUPPORTED: &str = "Only type schemas are supported.";
const OAS_NESTED_OBJECTS_NOT_SUPPORTED: &str = "Nested objects are not supported.";
const OAS_ONLY_VALUE_ARRAYS_SUPPORTED: &str = "Only value arrays (string[], integer[], ..) are supported.";

/// Traverses a valid OpenAPI document and builds a function
/// for every operation it finds. Corresponding input/output
/// types, if any, are returned as well.
pub fn build_oas_functions(oas_document: &OpenAPI) -> Result<FunctionsAndTypes> {
    let mut functions = Map::<Function>::new();
    let mut types = Map::<Type>::new();

    // Wrap building into re-usable (mutable) closure.
    let mut try_build = |operation| -> Result<()> {
        if let Some(ref o) = operation {
            let (f, t) = build_oas_function(o)?;

            // Bookkeeping
            functions.extend(f);
            types.extend(t);
        }

        Ok(())
    };

    // Traverse the OpenAPI document (path items).
    for (_, path) in oas_document.paths.iter() {
        let path = resolve_reference(path)?;

        try_build(path.delete)?;
        try_build(path.get)?;
        try_build(path.head)?;
        try_build(path.options)?;
        try_build(path.patch)?;
        try_build(path.post)?;
        try_build(path.put)?;
        try_build(path.trace)?;
    }

    Ok((functions, types))
}

/// Builds a function for a OpenAPI operation. Corresponding
/// input/output types, if any, are returned as well.
pub fn build_oas_function(operation: &Operation) -> Result<FunctionsAndTypes> {
    let operation_id = get_operation_id(&operation)?;

    let (input, i_types) = build_oas_function_input(&operation_id, operation)?;
    let (output, o_types) = build_oas_function_output(&operation_id, operation)?;

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

/// Gets and validates the identifier of an OpenAPI operation.
pub fn get_operation_id(operation: &Operation) -> Result<String> {
    let operation_id = if let Some(operation_id) = &operation.operation_id {
        operation_id.clone()
    } else {
        return Err(anyhow!(OAS_ADD_OPERATION_ID));
    };

    // The identifier must be a valid Bakery / Brane DSL function name.
    ensure!(
        operation_id.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "Invalid operation ID. Must consist only of alphanumeric and/or _ characters."
    );

    Ok(operation_id)
}

///
///
///
fn build_oas_function_input(
    operation_id: &str,
    operation: &Operation,
) -> Result<(Vec<Parameter>, Map<Type>)> {
    let mut input_properties = Vec::<Property>::new();
    let mut input_types = Map::<Type>::new();

    // Construct input properties.
    for parameter in &operation.parameters {
        let parameter = resolve_reference(parameter)?;
        let mut properties = parameter_to_properties(&parameter)?;

        input_properties.append(&mut properties);
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
) -> Result<(String, Map<Type>)> {
    let mut output_properties = Vec::<Property>::new();
    let mut output_types = Map::<Type>::new();

    // Construct output properties
    let response = if let Some(default) = &operation.responses.default {
        resolve_reference(default)?
    } else {
        let responses = &operation.responses.responses;
        if responses.len() != 1 {
            return Err(anyhow!(OAS_MULTIPLE_NOT_SUPPORTED));
        }

        if let Some(response) = responses.values().next() {
            resolve_reference(response)?
        } else {
            unreachable!()
        }
    };

    // Only 'application/json' responses are supported
    if let Some(content) = response.content.get("application/json") {
        if let Some(schema) = &content.schema {
            let schema = resolve_reference(schema)?;

            let optional = false; // check if is in required list
            let properties = schema_to_properties(None, &schema, optional)?;

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
fn parameter_to_properties(parameter: &OParameter) -> Result<Vec<Property>> {
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
            let schema = resolve_reference(schema)?;
            schema_to_properties(name, &schema, required)
        }
        ParameterSchemaOrContent::Content(_) => {
            Err(anyhow!(OAS_CONTENT_NOT_SUPPORTED))
        }
    }
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
        OType::Array(array) => {
            let items_schema = *resolve_reference(&array.items)?;

            let data_type = match &items_schema.schema_kind  {
                SchemaKind::Type(OType::String(_)) => String::from("string[]"),
                SchemaKind::Type(OType::Number(_)) => String::from("real[]"),
                SchemaKind::Type(OType::Integer(_)) => String::from("integer[]"),
                SchemaKind::Type(OType::Boolean{}) => String::from("boolean[]"),
                _ => return Err(anyhow!(OAS_ONLY_VALUE_ARRAYS_SUPPORTED)),
            };

            vec![Property::new(name.unwrap_or_default(), data_type, None, None, Some(!required), None)]
        },
        OType::Object(object) => {
            ensure!(name.is_none(), OAS_NESTED_OBJECTS_NOT_SUPPORTED);

            let mut properties = Vec::<Property>::new();
            for (name, p_schema) in object.properties.iter() {
                let p_schema = *resolve_reference(p_schema)?;
                let props = schema_to_properties(Some(name.clone()), &p_schema, required)?;

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

            vec![Property::new(name.unwrap_or_default(), data_type, None, None, Some(!required), None)]
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
