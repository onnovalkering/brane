use crate::ast::AstNode;
use crate::configuration::Configuration;
use curl::easy::Easy;
use itertools::interleave;
use specifications::common::Argument;
use specifications::package::{Function, PackageInfo};
use std::cmp::Ordering;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

#[derive(Debug)]
pub struct FunctionPattern {
    pub arguments: Vec<Argument>,
    pub name: String,
    pub meta: Map<String>,
    pub pattern: String,
    pub return_type: String,
}

impl Ord for FunctionPattern {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for FunctionPattern {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for FunctionPattern {}

impl PartialEq for FunctionPattern {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.name == other.name
    }
}

const LATEST: &str = "latest";

///
///
///
pub fn build_function_patterns(
    imports: Vec<AstNode>,
    config: &Configuration,
) -> FResult<Vec<FunctionPattern>> {
    let mut patterns = vec![];

    for import in imports {
        if let AstNode::Import { module, version } = import {
            let module_meta = get_module_meta(&module, &version, &config)?;
            let module_patterns = get_module_patterns(module_meta)?;

            patterns.extend(module_patterns);
        } else {
            unreachable!();
        }
    }

    Ok(patterns)
}

///
///
///
fn get_module_meta(
    module: &String,
    version: &Option<String>,
    config: &Configuration,
) -> FResult<PackageInfo> {
    let mut handle = Easy::new();

    // No provided version implies the use of the latest version
    let version = if let Some(version) = version {
        version.to_owned()
    } else {
        LATEST.to_string()
    };

    let meta_url = format!("{}/v1/{}/{}/meta", &config.registry_api_url, module, version);

    handle.url(meta_url.as_str())?;
    handle.perform()?;

    let mut content = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            content.extend_from_slice(data);
            Ok(data.len())
        })?;

        transfer.perform()?;
    }

    ensure!(
        handle.response_code()? == 200,
        "Failed to get module metadata from the registry."
    );

    let response = String::from_utf8_lossy(&content[..]).to_owned().to_string();
    let group_meta: PackageInfo = serde_json::from_str(&response).expect("Failed to deserialize provided group meta.");

    Ok(group_meta)
}

///
///
///
fn get_module_patterns(module: PackageInfo) -> FResult<Vec<FunctionPattern>> {
    let mut patterns = vec![];

    for (name, function) in module.functions.unwrap().iter() {
        let pattern = build_pattern(function)?;
        let mut meta = Map::<String>::new();

        meta.insert("kind".to_string(), module.kind.clone());
        meta.insert("name".to_string(), module.name.clone());
        meta.insert("version".to_string(), module.version.clone());

        let function_pattern = FunctionPattern {
            arguments: function.arguments.clone(),
            meta: meta,
            name: name.clone(),
            pattern: pattern,
            return_type: function.return_type.clone(),
        };

        patterns.push(function_pattern);
    }

    Ok(patterns)
}

///
///
///
fn build_pattern(function: &Function) -> FResult<String> {
    let mut pattern = vec![];

    if let None = &function.notation {
        bail!("No function notation...");
    }

    let notation = function.notation.clone().unwrap();

    if let Some(prefix) = notation.prefix {
        pattern.push(prefix.clone());
    }

    let mut arguments: Vec<String> = function
        .arguments
        .iter()
        .map(|arg| {
            let data_type = regex::escape(&arg.data_type);
            let data_type = if data_type.ends_with("]") {
                format!("{}|array", data_type)
            } else if data_type.chars().next().unwrap().is_uppercase() {
                format!("{}|object", data_type)
            } else {
                data_type
            };

            format!("<\\w+:({})>", data_type).to_owned()
        })
        .collect();

    if let Some(infix) = notation.infix {
        arguments = interleave(arguments, infix).collect();
    }

    for argument in arguments {
        pattern.push(argument);
    }

    if let Some(postfix) = notation.postfix {
        pattern.push(postfix.clone());
    }

    Ok(pattern.join(" "))
}
