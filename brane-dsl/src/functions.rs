use itertools::interleave;
use specifications::common::Argument;
use specifications::package::{Function, PackageInfo};
use regex;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

#[derive(Clone, Debug)]
pub struct FunctionPattern {
    pub arguments: Vec<Argument>,
    pub name: String,
    pub meta: Map<String>,
    pub pattern: String,
    pub return_type: String,
}

///
///
///
pub fn get_module_patterns(module: &PackageInfo) -> FResult<Vec<FunctionPattern>> {
    let mut patterns = vec![];

    for (name, function) in module.functions.as_ref().unwrap().iter() {
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
        pattern.push(regex::escape(&prefix));
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
        let infix: Vec<String> = infix.iter().map(|i| regex::escape(&i)).collect();
        arguments = interleave(arguments, infix).collect();
    }

    for argument in arguments {
        pattern.push(argument);
    }

    if let Some(postfix) = notation.postfix {
        pattern.push(regex::escape(&postfix));
    }

    Ok(pattern.join(" "))
}
