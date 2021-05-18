use crate::parser::ast::{Expr, Ident, Stmt};
use anyhow::Result;
use itertools::interleave;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use specifications::{
    common::{CallPattern, Function, Parameter},
    package::{PackageIndex, PackageInfo},
};

type Map<T> = std::collections::HashMap<String, T>;

/// !! Rudimentary support for patterns.
///
///
pub fn resolve_patterns(
    program: Vec<Stmt>,
    package_index: &PackageIndex,
) -> Result<Vec<Stmt>> {
    let mut function_patterns = vec![];
    for (_, package) in &package_index.packages {
        let package_patterns = get_module_patterns(package)?;
        function_patterns.extend(package_patterns);
    }

    let mut statements = vec![];
    for stmt in program {
        let stmt = match stmt {
            Stmt::Expr(Expr::Pattern(pattern)) => {
                let call = pattern_to_call(pattern, &function_patterns)?;
                Stmt::Expr(call)
            }
            stmt => stmt,
        };

        statements.push(stmt);
    }

    Ok(statements)
}

///
///
///
fn pattern_to_call(
    pattern: Vec<Expr>,
    patterns: &Vec<FunctionPattern>,
) -> Result<Expr> {
    dbg!(&patterns);
    let terms_pattern = build_terms_pattern(&pattern)?;
    debug!("Attempting to rewrite to call: {:?}", terms_pattern);

    let (function, indexes) = match_pattern_to_function(terms_pattern, patterns)?;
    let arguments = indexes.into_iter().map(|i| pattern.get(i).unwrap()).cloned().collect();

    Ok(Expr::Call {
        function: Ident(function.name.clone()),
        arguments,
    })
}

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
    if module.functions.is_none() {
        return Ok(patterns);
    }

    for (name, function) in module.functions.as_ref().unwrap().iter() {
        let pattern = build_pattern(name, function)?;
        let mut meta = Map::<String>::new();

        meta.insert(String::from("kind"), module.kind.clone());
        meta.insert(String::from("name"), module.name.clone());
        meta.insert(String::from("version"), module.version.clone());
        if module.kind != "dsl" {
            meta.insert(String::from("image"), format!("{}:{}", module.name, module.version));
        }

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
fn build_pattern(
    name: &String,
    function: &Function,
) -> Result<String> {
    let mut pattern = vec![];

    if function.pattern.is_none() {
        pattern.push(regex::escape(name));
    }

    let notation = function
        .pattern
        .clone()
        .unwrap_or_else(|| CallPattern::new(None, None, None));
    if let Some(prefix) = notation.prefix {
        pattern.push(regex::escape(&prefix));
    }

    let mut arguments: Vec<String> = function
        .parameters
        .iter()
        .filter(|p| p.secret.is_none()) // Ignore implicit arguments
        .map(|arg| {
            let data_type = regex::escape(&arg.data_type);
            let data_type = if data_type.ends_with(']') {
                format!("{}|array", data_type)
            } else if data_type.chars().next().unwrap().is_uppercase() {
                format!("{}|object", data_type)
            } else {
                data_type
            };

            format!("<[\\.\\w]+:({})>", data_type)
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

///
///
///
fn build_terms_pattern(terms: &Vec<Expr>) -> Result<String> {
    let mut term_pattern_segments = vec![];
    for term in terms {
        match term {
            Expr::Ident(Ident(name)) => {
                term_pattern_segments.push(name.to_string());
            }
            Expr::Literal(literal) => {
                let temp_var = create_temp_var(true);
                let segment = format!("<{}:{}>", temp_var, literal.data_type());

                term_pattern_segments.push(segment);
            }
            _ => unreachable!(),
        }
    }

    let term_pattern = term_pattern_segments.join(" ");
    Ok(term_pattern)
}

///
///
///
fn create_temp_var(literal: bool) -> String {
    let random_name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .collect::<String>()
        .to_lowercase();

    if literal {
        random_name
    } else {
        format!("_{}", random_name)
    }
}

///
///
///
fn match_pattern_to_function(
    pattern: String,
    functions: &Vec<FunctionPattern>,
) -> Result<(FunctionPattern, Vec<usize>)> {
    for function in functions {
        debug!("Check: {:?}", &function.pattern);
        let needle = Regex::new(&function.pattern).unwrap();

        if let Some(coverage) = needle.find(&pattern) {
            if coverage.start() == 0 && coverage.end() == pattern.len() {
                debug!("match: {:?}", &function.pattern);

                let arg_indexes: Vec<usize> = pattern
                    .split(' ')
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, t)| if t.starts_with('<') { Some(i) } else { None })
                    .collect();

                return Ok((function.clone(), arg_indexes));
            }
        }
    }

    Err(anyhow!("Failed to match pattern: {}", pattern))
}
