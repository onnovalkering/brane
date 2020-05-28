use crate::packages;
use cwl::v11_clt::{CommandInputParameter, CommandInputParameterType, CommandLineToolInput, CommandLineToolInputType};
use cwl::v11_clt::{
    CommandLineToolOutput, CommandLineToolOutputType, CommandOutputParameter, CommandOutputParameterType,
};
use cwl::v11_cm::CwlType;
use cwl::v11_wf::{
    WorkflowInputParameter, WorkflowInputParameterType, WorkflowInputType, WorkflowInputs, WorkflowOutputParameter,
    WorkflowOutputParameterType, WorkflowOutputType, WorkflowOutputs, WorkflowSteps,
};
use cwl::{v11::CwlDocument, v11_clt::CommandLineTool, v11_wf::Workflow};
use specifications::common::{Argument, Type};
use specifications::package::{Function, PackageInfo};
use std::io::Write;
use std::path::PathBuf;
use std::{fs, fs::File};

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn handle(
    context: PathBuf,
    file: PathBuf,
) -> FResult<()> {
    let cwl_file = context.join(file);
    let cwl_document = CwlDocument::from_path(&cwl_file)?;

    // Prepare package directory
    let package_info = create_package_info(&cwl_document)?;
    let package_dir = packages::get_package_dir(&package_info.name, Some(&package_info.version))?;
    prepare_directory(&cwl_document, &cwl_file, &package_info, &package_dir)?;

    Ok(())
}

///
///
///
fn create_package_info(cwl_document: &CwlDocument) -> FResult<PackageInfo> {
    let schema = cwl_document.get_schema_props();
    let name = schema.name.clone().expect("Please add s:name to your CWL document.");
    let version = schema
        .version
        .clone()
        .expect("Please add s:version to your CWL document.");

    let (function_name, function, types) = match cwl_document {
        CwlDocument::CommandLineTool(clt) => build_clt_function(clt)?,
        CwlDocument::Workflow(wf) => build_wf_function(wf)?,
    };

    let mut functions = Map::<Function>::new();
    functions.insert(function_name, function);

    let package_info = PackageInfo::new(
        name,
        version,
        schema.description,
        String::from("cwl"),
        Some(functions),
        Some(types),
    );

    Ok(package_info)
}

///
///
///
fn build_clt_function(clt: &CommandLineTool) -> FResult<(String, Function, Map<Type>)> {
    let name = clt.label.clone().expect("Add label").to_lowercase();

    let inputs = if let CommandLineToolInput::ParameterMap(inputs) = &clt.inputs {
        inputs
    } else {
        bail!("Only ParameterMap notation is supported for inputs.");
    };

    let outputs = if let CommandLineToolOutput::ParameterMap(outputs) = &clt.outputs {
        outputs
    } else {
        bail!("Only ParameterMap notation is supported for outputs.");
    };

    // Construct custom input type
    let mut input_properties = vec![];
    for (key, value) in inputs.iter() {
        let property = construct_clt_input_prop(key.to_string(), value)?;
        input_properties.push(property);
    }

    let input = Type {
        name: format!("{}Input", uppercase_first_letter(name.as_str())),
        description: None,
        properties: Some(input_properties),
    };

    // Construct custom output type
    let mut output_properties = vec![];
    for (key, value) in outputs.iter() {
        let property = construct_clt_output_prop(key.to_string(), value)?;
        output_properties.push(property);
    }

    let output = Type {
        name: format!("{}Output", uppercase_first_letter(name.as_str())),
        description: None,
        properties: Some(output_properties),
    };

    let argument = Argument::new(String::from("input"), input.name.clone(), None, None, None, None, None);
    let function = Function::new(vec![argument], None, output.name.clone());

    let mut types = Map::<Type>::new();
    types.insert(input.name.clone(), input);
    types.insert(output.name.clone(), output);

    Ok((name, function, types))
}

///
///
///
fn build_wf_function(wf: &Workflow) -> FResult<(String, Function, Map<Type>)> {
    let name = wf.label.clone().expect("Add label").to_lowercase();

    let inputs = if let WorkflowInputs::ParameterMap(inputs) = &wf.inputs {
        inputs
    } else {
        bail!("Only ParameterMap notation is supported for workflow inputs.");
    };

    let outputs = if let WorkflowOutputs::ParameterMap(outputs) = &wf.outputs {
        outputs
    } else {
        bail!("Only ParameterMap notation is supported for outputs.");
    };

    // Construct custom input type
    let mut input_properties = vec![];
    for (key, value) in inputs.iter() {
        let property = construct_wf_input_prop(key.to_string(), value)?;
        input_properties.push(property);
    }

    let input = Type {
        name: format!("{}Input", uppercase_first_letter(name.as_str())),
        description: None,
        properties: Some(input_properties),
    };

    // Construct custom output type
    let mut output_properties = vec![];
    for (key, value) in outputs.iter() {
        let property = construct_wf_output_prop(key.to_string(), value)?;
        output_properties.push(property);
    }

    let output = Type {
        name: format!("{}Output", uppercase_first_letter(name.as_str())),
        description: None,
        properties: Some(output_properties),
    };

    let argument = Argument::new(String::from("input"), input.name.clone(), None, None, None, None, None);
    let function = Function::new(vec![argument], None, output.name.clone());

    let mut types = Map::<Type>::new();
    types.insert(input.name.clone(), input);
    types.insert(output.name.clone(), output);

    Ok((name, function, types))
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
fn construct_clt_input_prop(
    name: String,
    input_paramter: &CommandInputParameter,
) -> FResult<Argument> {
    let data_type = if let CommandInputParameterType::Type(r#type) = &input_paramter.r#type {
        if let CommandLineToolInputType::CwlType(cwl_type) = r#type {
            if let CwlType::Str(data_type) = cwl_type {
                data_type.to_owned()
            } else {
                bail!("Unsupported type for paramter: {}", name);
            }
        } else {
            bail!("Unsupported type notation for paramter: {}", name);
        }
    } else {
        bail!("Unsupported type notation for paramter: {}", name);
    };

    Ok(Argument::new(name, data_type, None, None, None, None, None))
}

///
///
///
fn construct_clt_output_prop(
    name: String,
    output_parameter: &CommandOutputParameter,
) -> FResult<Argument> {
    let data_type = if let CommandOutputParameterType::Type(r#type) = &output_parameter.r#type {
        if let CommandLineToolOutputType::CwlType(cwl_type) = r#type {
            if let CwlType::Str(data_type) = cwl_type {
                data_type.to_owned()
            } else {
                bail!("Unsupported type for paramter: {}", name);
            }
        } else {
            bail!("Unsupported type notation for paramter: {}", name);
        }
    } else {
        bail!("Unsupported type notation for paramter: {}", name);
    };

    Ok(Argument::new(name, data_type, None, None, None, None, None))
}

///
///
///
fn construct_wf_input_prop(
    name: String,
    input_paramter: &WorkflowInputParameter,
) -> FResult<Argument> {
    let data_type = if let WorkflowInputParameterType::Type(r#type) = &input_paramter.r#type {
        if let WorkflowInputType::CwlType(cwl_type) = r#type {
            if let CwlType::Str(data_type) = cwl_type {
                data_type.to_owned()
            } else {
                bail!("Unsupported type for paramter: {}", name);
            }
        } else {
            bail!("Unsupported type notation for paramter: {}", name);
        }
    } else {
        bail!("Unsupported type notation for paramter: {}", name);
    };

    Ok(Argument::new(name, data_type, None, None, None, None, None))
}

///
///
///
fn construct_wf_output_prop(
    name: String,
    output_parameter: &WorkflowOutputParameter,
) -> FResult<Argument> {
    let data_type = if let WorkflowOutputParameterType::Type(r#type) = &output_parameter.r#type {
        if let WorkflowOutputType::CwlType(cwl_type) = r#type {
            if let CwlType::Str(data_type) = cwl_type {
                data_type.to_owned()
            } else {
                bail!("Unsupported type for paramter: {}", name);
            }
        } else {
            bail!("Unsupported type notation for paramter: {}", name);
        }
    } else {
        bail!("Unsupported type notation for paramter: {}", name);
    };

    Ok(Argument::new(name, data_type, None, None, None, None, None))
}

///
///
///
fn prepare_directory(
    cwl_document: &CwlDocument,
    cwl_file: &PathBuf,
    package_info: &PackageInfo,
    package_dir: &PathBuf,
) -> FResult<()> {
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
                bail!("Can't find workfow step file: {:?}", run_file);
            }
        }
    }

    // Write package.yml to package directory
    let mut buffer = File::create(package_dir.join("package.yml"))?;
    write!(buffer, "{}", serde_yaml::to_string(&package_info)?)?;

    Ok(())
}
