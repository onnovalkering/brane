use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::cmp::{Ordering, PartialEq, PartialOrd};

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Argument {
    #[serde(rename = "type")]
    pub data_type: String,
    pub default: Option<String>,
    pub description: Option<String>,
    pub name: String,
    pub optional: Option<bool>,
    pub properties: Option<Vec<Argument>>,
    pub secret: Option<bool>,
}

impl Argument {
    pub fn new(
        name: String,
        data_type: String,
        description: Option<String>,
        optional: Option<bool>,
        default: Option<String>,
        secret: Option<bool>,
        properties: Option<Vec<Argument>>,
    ) -> Argument {
        Argument {
            data_type,
            default,
            description,
            name,
            optional,
            properties,
            secret,
        }
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunctionNotation {
    pub infix: Option<Vec<String>>,
    pub postfix: Option<String>,
    pub prefix: Option<String>,
}

#[allow(unused)]
impl FunctionNotation {
    pub fn new(
        prefix: Option<String>,
        infix: Option<Vec<String>>,
        postfix: Option<String>,
    ) -> FunctionNotation {
        FunctionNotation { infix, postfix, prefix }
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Type {
    pub description: Option<String>,
    pub name: String,
    pub properties: Option<Vec<Argument>>,
}

#[serde(untagged, rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Value {
    Array {
        complex: String,
        entries: Vec<Value>,
    },
    Literal(Literal),
    None,
    Object {
        complex: String,
        entries: Vec<(String, Value)>,
    },
    Variable(String),
}

impl Value {
    ///
    ///
    ///
    pub fn get_complex(&self) -> &str {
        match self {
            Value::Array { complex, .. } => complex.as_str(),
            Value::Literal(literal) => match literal {
                Literal::Boolean(_) => "boolean",
                Literal::Decimal(_) => "real",
                Literal::Integer(_) => "integer",
                Literal::Str(_) => "string",
            },
            Value::None => "void",
            Value::Object { complex, .. } => complex,
            Value::Variable(_) => "variable",
        }
    }
}

impl PartialEq for Value {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        use Value::*;

        match (self, other) {
            (None, None) => true,
            (Literal(lhs), Literal(rhs)) => lhs.eq(rhs),
            (Array { entries: lhs, .. }, Array { entries: rhs, .. }) => lhs.eq(rhs),
            (Object { entries: lhs, .. }, Object { entries: rhs, .. }) => lhs.eq(rhs),
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        use Value::*;

        match (self, other) {
            (Literal(lhs), Literal(rhs)) => lhs.partial_cmp(rhs),
            (Array { entries: lhs, .. }, Array { entries: rhs, .. }) => lhs.partial_cmp(rhs),
            (Object { entries: lhs, .. }, Object { entries: rhs, .. }) => lhs.partial_cmp(rhs),
            _ => Option::None,
        }
    }
}

#[serde(untagged, rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Literal {
    Boolean(bool),
    Decimal(f64),
    Integer(i64),
    Str(String),
}

impl PartialEq for Literal {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        use Literal::*;

        match (self, other) {
            (Boolean(lhs), Boolean(rhs)) => lhs.eq(rhs),
            (Decimal(lhs), Decimal(rhs)) => lhs.eq(rhs),
            (Integer(lhs), Integer(rhs)) => lhs.eq(rhs),
            (Str(lhs), Str(rhs)) => lhs.eq(rhs),
            _ => false,
        }
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        use Literal::*;

        match (self, other) {
            (Boolean(lhs), Boolean(rhs)) => lhs.partial_cmp(rhs),
            (Decimal(lhs), Decimal(rhs)) => lhs.partial_cmp(rhs),
            (Integer(lhs), Integer(rhs)) => lhs.partial_cmp(rhs),
            (Str(lhs), Str(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}
