use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
        FunctionNotation {
            infix,
            postfix,
            prefix,
        }
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
    pub fn get_complex(&self) -> String {
        use Value::*;
        match self {
            Array {
                complex,
                entries: _,
            } => complex.to_string(),
            Literal(literal) => literal.get_complex(),
            None => String::from("void"),
            Object {
                complex,
                entries: _,
            } => complex.to_string(),
            Variable(_) => String::from("variable")
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

impl Literal {
    pub fn get_complex(&self) -> String {
        use Literal::*;
        match self {
            Boolean(_) => "boolean".to_string(),
            Decimal(_) => "real".to_string(),
            Integer(_) => "integer".to_string(),
            Str(_) => "string".to_string(),
        }
    }
}