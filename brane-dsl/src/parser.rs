use anyhow::Result;
use pest::iterators::Pair;
use pest::Parser;
use semver::Version;
use specifications::common::Value;
use std::fmt::{self, Display, Formatter};

#[derive(Parser)]
#[grammar = "grammer/bakery.pest"]
pub struct BakeryParser;

type Map<T> = std::collections::HashMap<String, T>;

#[derive(Clone, Debug)]
pub enum AstNode {
    Assignment {
        name: String,
        expr: Box<AstNode>,
    },
    Call {
        terms: Vec<AstNode>,
    },
    Condition {
        predicate: AstPredicate,
        if_exec: Box<AstNode>,
        el_exec: Option<Box<AstNode>>,
    },
    Import {
        module: String,
        version: Option<Version>,
    },
    Literal {
        value: Value,
    },
    Variable {
        name: String,
    },
    Parameter {
        name: String,
        complex: String,
    },
    Terminate {
        terms: Option<Vec<AstNode>>,
    },
    WaitUntil {
        predicate: AstPredicate,
    },
    WhileLoop {
        predicate: AstPredicate,
        exec: Box<AstNode>,
    },
    Word {
        text: String,
    }
}

impl AstNode {
    ///
    ///
    ///
    pub fn is_import(&self) -> bool {
        if let AstNode::Import { .. } = self {
            true
        } else {
            false
        }
    }

    ///
    ///
    ///
    pub fn is_literal(&self) -> bool {
        if let AstNode::Literal { .. } = self {
            true
        } else {
            false
        }
    }    

    ///
    ///
    ///
    pub fn is_term(&self) -> bool {
        use AstNode::*;
        match self {
            Literal { .. } | Variable { .. } | Word { .. } => true,
            _ => false
        }
    }        
}

impl Display for AstNode {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        match self {
            AstNode::Literal { value } => write!(f, "{}", value),
            AstNode::Variable { name } => write!(f, "{}", name),
            AstNode::Word { text } => write!(f, "{}", text),
            _ => unimplemented!()
        }
    }
}



#[derive(Clone, Debug)]
pub enum AstPredicate {
    Call {
        terms: Vec<AstNode>,
    },
    Comparison {
        lhs_terms: Vec<AstNode>,
        relation: AstRelation,
        rhs_terms: Vec<AstNode>,
    },
}

#[derive(Clone, Debug)]
pub enum AstRelation {
    Equals = 1,
    NotEquals = 2,
    Greater = 3,
    Less = 4,
    GreaterOrEqual = 5,
    LessOrEqual = 6,
}

///
///
///
pub fn parse(input: &str) -> Result<Vec<AstNode>> {
    let mut ast = vec![];

    let pairs = BakeryParser::parse(Rule::program, &input)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::assignment => ast.push(parse_assignment_rule(pair)?),
            Rule::call => ast.push(parse_call_rule(pair)?),
            Rule::condition => ast.push(parse_condition_rule(pair)?),
            Rule::import => ast.push(parse_import_rule(pair)?),
            Rule::parameter => ast.push(parse_parameter_rule(pair)?),
            Rule::terminate => ast.push(parse_terminate_rule(pair)?),
            Rule::wait_until => ast.push(parse_wait_until_rule(pair)?),
            Rule::while_loop => ast.push(parse_while_loop_rule(pair)?),
            _ => {}
        }
    }

    Ok(ast)
}

///
///
///
fn parse_array_rule(rule: Pair<Rule>) -> Result<Vec<Value>> {
    let array = rule.into_inner();

    let mut values = vec![];
    for element in array {
        let inner = element.into_inner().next().unwrap();
        values.push(parse_value_rule(inner)?);
    }

    Ok(values)
}

///
///
///
fn parse_assignment_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let mut assignment = rule.into_inner();

    let name = assignment.next().unwrap().as_str().to_string();
    let expr = assignment.next().unwrap().into_inner();
    
    let mut terms = vec![];
    for term in expr {
        terms.push(parse_term_rule(term)?);
    }

    if terms.len() == 1 {
        let term = terms.first().unwrap();
        match term {
            AstNode::Word { .. } => { }
            _ => {
                // It's not a call, but assignment from literal or variable
                let expr = Box::new(term.clone());
                return Ok(AstNode::Assignment { name, expr });
            }
        }
    }
    
    let expr = Box::new(AstNode::Call { terms });
    Ok(AstNode::Assignment { name, expr })
}

///
///
///
fn parse_call_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let call = rule.into_inner();

    let mut terms = vec![];
    for term in call {
        terms.push(parse_term_rule(term)?);
    }

    Ok(AstNode::Call { terms })
}

///
///
///
fn parse_condition_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let mut condition = rule.into_inner();

    let predicate = parse_predicate_rule(condition.next().unwrap())?;
    let if_exec = Box::new(parse_execution_rule(condition.next().unwrap())?);
    let el_exec = if let Some(node) = condition.next() {
        Some(Box::new(parse_execution_rule(node)?))
    } else {
        None
    };

    Ok(AstNode::Condition {
        predicate,
        if_exec,
        el_exec,
    })
}

///
///
///
fn parse_execution_rule(rule: Pair<Rule>) -> Result<AstNode> {
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
fn parse_import_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let mut import = rule.into_inner();

    let module = parse_string_rule(import.next().unwrap())?;
    let version = if let Some(version) = import.next() {
        Some(Version::parse(&parse_string_rule(version)?)?)
    } else {
        None
    };

    Ok(AstNode::Import { module, version })
}

///
///
///
fn parse_literal_rule(rule: Pair<Rule>) -> Result<Value> {
    let literal = rule.into_inner().next().unwrap();

    match literal.as_rule() {
        Rule::boolean => Ok(Value::Boolean(literal.as_str().parse()?)),
        Rule::decimal => Ok(Value::Real(literal.as_str().parse()?)),
        Rule::integer => Ok(Value::Integer(literal.as_str().parse()?)),
        Rule::string => Ok(Value::Unicode(parse_string_rule(literal)?)),
        _ => unreachable!(),
    }
}

///
///
///
fn parse_name_rule(rule: Pair<Rule>) -> Result<String> {
    Ok(rule.as_str().trim().to_string())
}

///
///
///
fn parse_object_rule(rule: Pair<Rule>) -> Result<Value> {
    let mut object = rule.into_inner();

    let complex = object.next().unwrap();
    let data_type = complex.as_str().to_string();

    let mut properties = Map::<Value>::new();
    while let Some(prop) = object.next() {
        let mut prop_inner = prop.into_inner();

        let name = prop_inner.next().unwrap().as_str().to_string();
        let value = prop_inner.next().unwrap();

        properties.insert(name, parse_value_rule(value)?);
    }

    Ok(Value::Struct { data_type, properties })
}

///
///
///
fn parse_parameter_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let mut parameter = rule.into_inner();

    let name = parameter.next().unwrap().as_str().to_string();
    let complex = parameter.next().unwrap().as_str().to_string();

    Ok(AstNode::Parameter { name, complex })
}

///
///
///
fn parse_predicate_rule(rule: Pair<Rule>) -> Result<AstPredicate> {
    let predicate = rule;

    match predicate.as_rule() {
        Rule::call => {
            if let AstNode::Call { terms } = parse_call_rule(predicate)? {
                Ok(AstPredicate::Call { terms })
            } else {
                unreachable!()
            }
        }
        Rule::comparison => Ok(parse_comparison_rule(predicate)?),
        _ => unreachable!(),
    }
}

///
///
///
fn parse_comparison_rule(rule: Pair<Rule>) -> Result<AstPredicate> {
    let mut comparison = rule.into_inner();

    let lhs_terms = if let AstNode::Call { terms } = parse_call_rule(comparison.next().unwrap())? {
        terms
    } else {
        unreachable!();
    };

    let relation = parse_relation_rule(comparison.next().unwrap())?;

    let rhs_terms = if let AstNode::Call { terms } = parse_call_rule(comparison.next().unwrap())? {
        terms
    } else {
        unreachable!();
    };

    Ok(AstPredicate::Comparison {
        lhs_terms,
        relation,
        rhs_terms,
    })
}

///
///
///
fn parse_relation_rule(rule: Pair<Rule>) -> Result<AstRelation> {
    let relation = rule.as_str().trim();

    Ok(match relation {
        "=" => AstRelation::Equals,
        "!=" => AstRelation::NotEquals,
        ">" => AstRelation::Greater,
        "<" => AstRelation::Less,
        ">=" => AstRelation::GreaterOrEqual,
        "<=" => AstRelation::LessOrEqual,
        _ => unreachable!(),
    })
}

///
///
///
fn parse_while_loop_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let mut while_loop = rule.into_inner();

    let predicate = parse_predicate_rule(while_loop.next().unwrap())?;
    let exec = Box::new(parse_execution_rule(while_loop.next().unwrap())?);

    Ok(AstNode::WhileLoop { predicate, exec })
}

///
///
///
fn parse_string_rule(rule: Pair<Rule>) -> Result<String> {
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
fn parse_term_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let term = rule.into_inner().next().unwrap();

    match term.as_rule() {
        Rule::name => {
            // TODO: check if is in variable table

            Ok(AstNode::Word { text: parse_name_rule(term)? })
        },
        Rule::symbol => Ok(AstNode::Word { text: parse_name_rule(term)? }),
        Rule::value => Ok(AstNode::Literal { value: parse_value_rule(term)? }),
        _ => unreachable!(),
    }
}

///
///
///
fn parse_terminate_rule(rule: Pair<Rule>) -> Result<AstNode> {
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

    Ok(AstNode::Terminate { terms })
}

///
///
///
fn parse_wait_until_rule(rule: Pair<Rule>) -> Result<AstNode> {
    let mut wait_until = rule.into_inner();

    let predicate = parse_predicate_rule(wait_until.next().unwrap())?;

    Ok(AstNode::WaitUntil { predicate })
}

///
///
///
fn parse_value_rule(rule: Pair<Rule>) -> Result<Value> {
    let value = rule.into_inner().next().unwrap();

    match value.as_rule() {
        Rule::array => {
            let entries = parse_array_rule(value)?;
            let data_type = if !entries.is_empty() {
                format!("{}[]", entries.first().unwrap().data_type())
            } else {
                String::from("Array")
            };

            Ok(Value::Array { data_type, entries })
        }
        Rule::object => parse_object_rule(value),
        Rule::literal => parse_literal_rule(value),
        Rule::name => {
            let variable = parse_name_rule(value)?;
            let data_type = String::from("???");

            Ok(Value::Pointer { 
                variable,
                data_type,
                secret: false,
            })
        }
        _ => unreachable!(),
    }
}
