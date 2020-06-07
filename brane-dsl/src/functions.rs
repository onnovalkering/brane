use anyhow::Result;
use itertools::interleave;
use specifications::common::{Function, Parameter};
use specifications::package::PackageInfo;

type Map<T> = std::collections::HashMap<String, T>;

#[derive(Clone, Debug)]
pub struct FunctionPattern {
    pub parameters: Vec<Parameter>,
    pub name: String,
    pub meta: Map<String>,
    pub pattern: String,
    pub return_type: String,
}

///
///
///
pub fn get_module_patterns(module: &PackageInfo) -> Result<Vec<FunctionPattern>> {
    let mut patterns = vec![];

    for (name, function) in module.functions.as_ref().unwrap().iter() {
        let pattern = build_pattern(function)?;
        let mut meta = Map::<String>::new();

        meta.insert("kind".to_string(), module.kind.clone());
        meta.insert("name".to_string(), module.name.clone());
        meta.insert("version".to_string(), module.version.clone());
        meta.insert(String::from("image"), format!("{}:{}", module.name, module.version));

        let function_pattern = FunctionPattern {
            parameters: function.parameters.clone(),
            meta,
            name: name.clone(),
            pattern,
            return_type: function.return_type.clone(),
        };

        patterns.push(function_pattern);
    }

    Ok(patterns)
}

///
///
///
fn build_pattern(function: &Function) -> Result<String> {
    let mut pattern = vec![];

    if function.pattern.is_none() {
        return Err(anyhow!("Function doesn't have a call pattern."));
    }

    let notation = function.pattern.clone().unwrap();
    if let Some(prefix) = notation.prefix {
        pattern.push(regex::escape(&prefix));
    }

    let mut arguments: Vec<String> = function
        .parameters
        .iter()
        .map(|arg| {
            let data_type = regex::escape(&arg.data_type);
            let data_type = if data_type.ends_with(']') {
                format!("{}|array", data_type)
            } else if data_type.chars().next().unwrap().is_uppercase() {
                format!("{}|object", data_type)
            } else {
                data_type
            };

            format!("<\\w+:({})>", data_type)
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
