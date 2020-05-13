use pest::iterators::Pair;
use pest::Parser;
use semver::Version;
use specifications::common::{Literal, Value};

#[derive(Parser)]
#[grammar = "grammer/bakery.pest"]
pub struct BakeryParser;

type FResult<T> = Result<T, failure::Error>;

#[derive(Debug)]
pub enum AstNode {
    Assignment {
        name: String,
        terms: Vec<AstTerm>,
    },
    Call {
        terms: Vec<AstTerm>,
    },
    Condition {
        predicate: Box<AstNode>,
        if_exec: Box<AstNode>,
        el_exec: Option<Box<AstNode>>,
    },
    Import {
        module: String,
        version: Option<Version>,
    },
    Parameter {
        name: String,
        complex: String,
    },
    Repeat {
        predicate: Box<AstNode>,
        exec: Box<AstNode>,
    },
    Terminate {
        terms: Option<Vec<AstTerm>>,
    },
}

impl AstNode {
    ///
    ///
    ///
    pub fn is_import(&self) -> bool {
        match self {
            AstNode::Import { module: _, version: _ } => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum AstTerm {
    Name(String),
    Symbol(String),
    Value(Value),
}

impl AstTerm {
    ///
    ///
    ///
    pub fn is_value(&self) -> bool {
        match self {
            AstTerm::Value(_) => true,
            _ => false,
        }
    }
}

///
///
///
pub fn parse(input: &String) -> FResult<Vec<AstNode>> {
    let mut ast = vec![];

    let pairs = BakeryParser::parse(Rule::program, &input)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::assignment => ast.push(parse_assignment_rule(pair)?),
            Rule::call => ast.push(parse_call_rule(pair)?),
            Rule::condition => ast.push(parse_condition_rule(pair)?),
            Rule::import => ast.push(parse_import_rule(pair)?),
            Rule::parameter => ast.push(parse_parameter_rule(pair)?),
            Rule::repeat => ast.push(parse_repeat_rule(pair)?),
            Rule::terminate => ast.push(parse_terminate_rule(pair)?),
            _ => {}
        }
    }

    Ok(ast)
}

///
///
///
fn parse_array_rule(rule: Pair<Rule>) -> FResult<Vec<Value>> {
    let entries = rule.into_inner();

    let mut values = vec![];
    for entry in entries {
        values.push(parse_value_rule(entry)?);
    }

    Ok(values)
}

///
///
///
fn parse_assignment_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let mut assignment = rule.into_inner();

    let name = assignment.next().unwrap().as_str().to_string();
    let call = assignment.next().unwrap().into_inner();

    let mut terms = vec![];
    for term in call {
        terms.push(parse_term_rule(term)?);
    }

    Ok(AstNode::Assignment {
        name: name,
        terms: terms,
    })
}

///
///
///
fn parse_call_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let call = rule.into_inner();

    let mut terms = vec![];
    for term in call {
        terms.push(parse_term_rule(term)?);
    }

    Ok(AstNode::Call { terms: terms })
}

///
///
///
fn parse_condition_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let mut condition = rule.into_inner();

    let predicate = Box::new(parse_call_rule(condition.next().unwrap())?);
    let if_exec = Box::new(parse_execution_rule(condition.next().unwrap())?);
    let el_exec = if let Some(node) = condition.next() {
        Some(Box::new(parse_execution_rule(node)?))
    } else {
        None
    };

    Ok(AstNode::Condition {
        predicate: predicate,
        if_exec: if_exec,
        el_exec: el_exec,
    })
}

///
///
///
fn parse_execution_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let execution = rule;

    match execution.as_rule() {
        Rule::assignment => Ok(parse_assignment_rule(execution)?),
        Rule::call => Ok(parse_call_rule(execution)?),
        Rule::parameter => Ok(parse_parameter_rule(execution)?),
        _ => unreachable!(),
    }
}

///
///
///
fn parse_import_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let mut import = rule.into_inner();

    let module = parse_string_rule(import.next().unwrap())?;

    let version = if let Some(version) = import.next() {
        Some(Version::parse(&parse_string_rule(version)?)?)
    } else {
        None
    };

    Ok(AstNode::Import {
        module: module,
        version: version,
    })
}

///
///
///
fn parse_literal_rule(rule: Pair<Rule>) -> FResult<Literal> {
    let literal = rule.into_inner().next().unwrap();

    match literal.as_rule() {
        Rule::boolean => Ok(Literal::Boolean(literal.as_str().parse()?)),
        Rule::decimal => Ok(Literal::Decimal(literal.as_str().parse()?)),
        Rule::integer => Ok(Literal::Integer(literal.as_str().parse()?)),
        Rule::string => Ok(Literal::Str(parse_string_rule(literal)?)),
        _ => unreachable!(),
    }
}

///
///
///
fn parse_name_rule(rule: Pair<Rule>) -> FResult<String> {
    Ok(rule.as_str().trim().to_string())
}

///
///
///
fn parse_object_rule(rule: Pair<Rule>) -> FResult<Vec<(String, Value)>> {
    let entries = rule.into_inner();

    let mut values = vec![];
    for entry in entries {
        let mut entry_inner = entry.into_inner();

        let name = entry_inner.next().unwrap().as_str().to_string();
        let value = entry_inner.next().unwrap();

        values.push((name, parse_value_rule(value)?));
    }

    Ok(values)
}

///
///
///
fn parse_parameter_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let mut parameter = rule.into_inner();

    let name = parameter.next().unwrap().as_str().to_string();
    let complex = parameter.next().unwrap().as_str().to_string();

    Ok(AstNode::Parameter {
        name: name,
        complex: complex,
    })
}

///
///
///
fn parse_repeat_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let mut repeat = rule.into_inner();

    let predicate = Box::new(parse_call_rule(repeat.next().unwrap())?);
    let exec = Box::new(parse_execution_rule(repeat.next().unwrap())?);

    Ok(AstNode::Repeat {
        predicate: predicate,
        exec: exec,
    })
}

///
///
///
fn parse_string_rule(rule: Pair<Rule>) -> FResult<String> {
    let string = rule.into_inner().next().unwrap();

    match string.as_rule() {
        Rule::string_sq => Ok(string.as_str().trim_matches('\'').to_string()),
        Rule::string_dq => Ok(string.as_str().trim_matches('\"').to_string()),
        _ => unreachable!(),
    }
}

///
///
///
fn parse_symbol_rule(rule: Pair<Rule>) -> FResult<String> {
    Ok(rule.as_str().trim().to_string())
}

///
///
///
fn parse_term_rule(rule: Pair<Rule>) -> FResult<AstTerm> {
    let term = rule.into_inner().next().unwrap();

    match term.as_rule() {
        Rule::value => Ok(AstTerm::Value(parse_value_rule(term)?)),
        Rule::name => Ok(AstTerm::Name(parse_name_rule(term)?)),
        Rule::symbol => Ok(AstTerm::Symbol(parse_symbol_rule(term)?)),
        _ => unreachable!(),
    }
}

///
///
///
fn parse_terminate_rule(rule: Pair<Rule>) -> FResult<AstNode> {
    let mut assignment = rule.into_inner();

    let terms = if let Some(call) = assignment.next() {
        let mut terms = vec![];
        for term in call.into_inner() {
            terms.push(parse_term_rule(term)?);
        }

        Some(terms)
    } else {
        None
    };

    Ok(AstNode::Terminate { terms: terms })
}

///
///
///
fn parse_value_rule(rule: Pair<Rule>) -> FResult<Value> {
    let value = rule.into_inner().next().unwrap();

    match value.as_rule() {
        Rule::array => Ok(Value::Array {
            entries: parse_array_rule(value)?,
            complex: "array".to_string(),
        }),
        Rule::object => Ok(Value::Object {
            entries: parse_object_rule(value)?,
            complex: "object".to_string(),
        }),
        Rule::literal => Ok(Value::Literal(parse_literal_rule(value)?)),
        _ => unreachable!(),
    }
}
