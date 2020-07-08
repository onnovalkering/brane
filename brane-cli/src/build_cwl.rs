use crate::{packages, utils};
use anyhow::{Context, Result};
use console::style;
use cwl::v11_clt::{
    CommandInputParameter, CommandInputParameterType, CommandLineToolInput, CommandLineToolInputType,
    CommandLineToolOutput, CommandLineToolOutputType, CommandOutputParameter, CommandOutputParameterType,
};
use cwl::v11_wf::{
    WorkflowInputParameter, WorkflowInputParameterType, WorkflowInputType, WorkflowInputs, WorkflowOutputParameter,
    WorkflowOutputParameterType, WorkflowOutputType, WorkflowOutputs, WorkflowSteps,
};
use cwl::{v11::CwlDocument, v11_clt::CommandLineTool, v11_cm::Any, v11_cm::CwlType, v11_wf::Workflow};
use specifications::common::{CallPattern, Function, Parameter, Property, Type, Value};
use specifications::package::PackageInfo;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;

const CWL_ADD_NAME: &str = "Please add a name (s:name) to the CWL document.";
const CWL_ADD_LABEL: &str = "Please add a function name (label) to the CWL document.";
const CWL_ADD_VERSION: &str = "Please add a version (s:version) to the CWL document.";
const CWL_ONLY_PMAP_INPUT: &str = "Only ParameterMap notation is supported for inputs.";
const CWL_ONLY_PMAP_OUTPUT: &str = "Only ParameterMap notation is supported for outputs.";

///
///
///
pub fn handle(
    context: PathBuf,
    file: PathBuf,
) -> Result<()> {
    let cwl_file = context.join(file);
    let cwl_reader = BufReader::new(File::open(&cwl_file)?);
    let cwl_document = CwlDocument::from_reader(cwl_reader).unwrap();

    // Prepare package directory
    let package_info = create_package_info(&cwl_document)?;
    let package_dir = packages::get_package_dir(&package_info.name, Some(&package_info.version))?;
    prepare_directory(&cwl_document, &cwl_file, &package_info, &package_dir)?;

    println!(
        "Successfully built version {} of CWL package {}.",
        style(&package_info.version).bold().cyan(),
        style(&package_info.name).bold().cyan(),
    );

    Ok(())
}

///
///
///
fn create_package_info(cwl_document: &CwlDocument) -> Result<PackageInfo> {
    let (name, version, description) = extract_metadata(&cwl_document)?;

    let (functions, types) = build_cwl_functions(&cwl_document)?;

    let package_info = PackageInfo::new(
        name,
        version,
        description,
        String::from("cwl"),
        Some(functions),
        Some(types),
    );

    Ok(package_info)
}

///
///
///
fn extract_metadata(cwl_document: &CwlDocument) -> Result<(String, String, Option<String>)> {
    let schema = cwl_document.get_schema_props();

    let name = if let Some(name) = schema.name {
        name
    } else {
        return Err(anyhow!(CWL_ADD_NAME));
    };

    let version = if let Some(version) = schema.version {
        version
    } else {
        return Err(anyhow!(CWL_ADD_VERSION));
    };

    let description = schema.description;

    Ok((name, version, description))
}

fn build_cwl_functions(cwl_document: &CwlDocument) -> Result<(Map<Function>, Map<Type>)> {
    let mut functions = Map::<Function>::new();
    let mut types = Map::<Type>::new();

    let (name, input_properties, output_properties) = match cwl_document {
        CwlDocument::CommandLineTool(clt) => construct_clt_properties(clt)?,
        CwlDocument::Workflow(wf) => construct_wf_properties(wf)?,
    };

    let type_name = utils::uppercase_first_letter(&name);

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
            match output_property.data_type.as_str() {
                "stdout" => String::from("string"),
                _ => output_property.data_type.clone(),
            }
        } else {
            String::from("unit")
        }
    };

    // Construct function
    let call_pattern = CallPattern::new(Some(name.to_lowercase()), None, None);
    let function = Function::new(input_parameters, Some(call_pattern), return_type);
    functions.insert(name.to_lowercase(), function);

    Ok((functions, types))
}

///
///
///
fn construct_clt_properties(clt: &CommandLineTool) -> Result<(String, Vec<Property>, Vec<Property>)> {
    let name = if let Some(label) = &clt.label {
        utils::assert_valid_bakery_name(label).with_context(|| format!("Label '{}' is not valid.", label))?;

        label.clone()
    } else {
        return Err(anyhow!(CWL_ADD_LABEL));
    };

    let inputs = if let CommandLineToolInput::ParameterMap(inputs) = &clt.inputs {
        inputs
    } else {
        return Err(anyhow!(CWL_ONLY_PMAP_INPUT));
    };

    let outputs = if let CommandLineToolOutput::ParameterMap(outputs) = &clt.outputs {
        outputs
    } else {
        return Err(anyhow!(CWL_ONLY_PMAP_OUTPUT));
    };

    let mut input_properties = Vec::<Property>::new();
    let mut output_properties = Vec::<Property>::new();

    // Construct input properties
    for (p_name, p) in inputs.iter() {
        let property = construct_clt_input_prop(p_name.to_string(), p)?;
        input_properties.push(property);
    }

    // Construct output properties
    for (p_name, p) in outputs.iter() {
        let property = construct_clt_output_prop(p_name.to_string(), p)?;
        output_properties.push(property);
    }

    Ok((name, input_properties, output_properties))
}

///
///
///
fn construct_clt_input_prop(
    name: String,
    input_parameter: &CommandInputParameter,
) -> Result<Property> {
    if let CommandInputParameterType::Type(p_type) = &input_parameter.r#type {
        if let CommandLineToolInputType::CwlType(cwl_type) = p_type {
            if let CwlType::Str(data_type) = cwl_type {
                let default = if let Some(Any::Any(default)) = &input_parameter.default {
                    default.as_str().map(|d| Value::Unicode(d.to_string()))
                } else {
                    None
                };

                return Ok(Property::new(name, data_type.to_string(), None, default, None, None));
            }
        }
    }

    Err(anyhow!("Unsupported type (notation) for parameter: {}", name))
}

///
///
///
fn construct_clt_output_prop(
    name: String,
    output_parameter: &CommandOutputParameter,
) -> Result<Property> {
    if let CommandOutputParameterType::Type(p_type) = &output_parameter.r#type {
        if let CommandLineToolOutputType::CwlType(cwl_type) = p_type {
            if let CwlType::Str(data_type) = cwl_type {
                return Ok(Property::new(name, data_type.to_string(), None, None, None, None));
            }
        }
    }

    Err(anyhow!("Unsupported type (notation) for parameter: {}", name))
}

///
///
///
fn construct_wf_properties(wf: &Workflow) -> Result<(String, Vec<Property>, Vec<Property>)> {
    let name = if let Some(label) = &wf.label {
        utils::assert_valid_bakery_name(label).with_context(|| format!("Label '{}' is not valid.", label))?;

        label.clone()
    } else {
        return Err(anyhow!(CWL_ADD_LABEL));
    };

    let inputs = if let WorkflowInputs::ParameterMap(inputs) = &wf.inputs {
        inputs
    } else {
        return Err(anyhow!(CWL_ONLY_PMAP_INPUT));
    };

    let outputs = if let WorkflowOutputs::ParameterMap(outputs) = &wf.outputs {
        outputs
    } else {
        return Err(anyhow!(CWL_ONLY_PMAP_OUTPUT));
    };

    // Construct input properties
    let mut input_properties = vec![];
    for (p_name, p) in inputs.iter() {
        let property = construct_wf_input_prop(p_name.to_string(), p)?;
        input_properties.push(property);
    }

    // Construct output properties
    let mut output_properties = vec![];
    for (p_name, p) in outputs.iter() {
        let property = construct_wf_output_prop(p_name.to_string(), p)?;
        output_properties.push(property);
    }

    Ok((name, input_properties, output_properties))
}

///
///
///
fn construct_wf_input_prop(
    name: String,
    input_paramter: &WorkflowInputParameter,
) -> Result<Property> {
    if let WorkflowInputParameterType::Type(p_type) = &input_paramter.r#type {
        if let WorkflowInputType::CwlType(cwl_type) = p_type {
            if let CwlType::Str(data_type) = cwl_type {
                return Ok(Property::new(name, data_type.to_string(), None, None, None, None));
            }
        }
    }

    Err(anyhow!("Unsupported type (notation) for parameter: {}", name))
}

///
///
///
fn construct_wf_output_prop(
    name: String,
    output_parameter: &WorkflowOutputParameter,
) -> Result<Property> {
    if let WorkflowOutputParameterType::Type(p_type) = &output_parameter.r#type {
        if let WorkflowOutputType::CwlType(cwl_type) = p_type {
            if let CwlType::Str(data_type) = cwl_type {
                return Ok(Property::new(name, data_type.to_string(), None, None, None, None));
            }
        }
    }

    Err(anyhow!("Unsupported type (notation) for parameter: {}", name))
}

///
///
///
fn prepare_directory(
    cwl_document: &CwlDocument,
    cwl_file: &PathBuf,
    package_info: &PackageInfo,
    package_dir: &PathBuf,
) -> Result<()> {
    fs::create_dir_all(&package_dir)?;

    // Copy CWL document(s) to package directory
    fs::copy(cwl_file, package_dir.join("document.cwl"))?;
    if let CwlDocument::Workflow(wf) = cwl_document {
        let runs: Vec<String> = match &wf.steps {
            WorkflowSteps::StepArray(steps) => steps.iter().map(|s| s.run.clone()).collect(),
            WorkflowSteps::StepMap(steps) => steps.iter().map(|(_, v)| v.run.clone()).collect(),
        };

        for run in runs {
            let run_file = PathBuf::from(run);
            if run_file.exists() {
                if let Some(run_file) = run_file.file_name() {
                    fs::copy(run_file, package_dir.join(run_file))?;
                }
            } else {
                return Err(anyhow!("Can't find workfow step file: {:?}", run_file));
            }
        }
    }

    // Write package.yml to package directory
    let mut buffer = File::create(package_dir.join("package.yml"))?;
    write!(buffer, "{}", serde_yaml::to_string(&package_info)?)?;

    Ok(())
}
