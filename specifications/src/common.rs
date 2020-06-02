use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::cmp::{Ordering, PartialEq, PartialOrd};

type Map<T> = std::collections::HashMap<String, T>;

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub data_type: String,
    pub default: Option<Value>,
    pub name: String,
    pub optional: Option<bool>,
}

impl Parameter {
    ///
    ///
    ///
    pub fn new(
        name: String,
        data_type: String,
        optional: Option<bool>,
        default: Option<Value>,
    ) -> Self {
        Parameter {
            data_type,
            default,
            name,
            optional,
        }
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    pub parameters: Vec<Parameter>,
    pub pattern: Option<CallPattern>,
    pub return_type: String,
}

impl Function {
    ///
    ///
    ///
    pub fn new(
        parameters: Vec<Parameter>,
        pattern: Option<CallPattern>,
        return_type: String,
    ) -> Self {
        Function {
            parameters,
            pattern,
            return_type,
        }
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CallPattern {
    pub infix: Option<Vec<String>>,
    pub postfix: Option<String>,
    pub prefix: Option<String>,
}

impl CallPattern {
    ///
    ///
    ///
    pub fn new(
        prefix: Option<String>,
        infix: Option<Vec<String>>,
        postfix: Option<String>,
    ) -> Self {
        CallPattern { infix, postfix, prefix }
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Type {
    pub name: String,
    pub properties: Vec<Property>,
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub data_type: String,
    pub default: Option<Value>,
    pub name: String,
    pub optional: Option<bool>,
    pub properties: Option<Vec<Property>>,
    pub secret: Option<bool>,
}

impl Property {
    pub fn new(
        name: String,
        data_type: String,
        properties: Option<Vec<Property>>,
        default: Option<Value>,
        optional: Option<bool>,
        secret: Option<bool>,
    ) -> Self {
        Property {
            data_type,
            default,
            name,
            optional,
            properties,
            secret,
        }
    }
}

#[serde(tag = "v", content = "c", rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Value {
    Array {
        #[serde(rename = "type")]
        data_type: String,
        entries: Vec<Value>,
    },
    Boolean(bool),
    Integer(i64),
    Pointer {
        #[serde(rename = "type")]
        data_type: String,
        variable: String,
    },
    Real(f64),
    Struct {
        #[serde(rename = "type")]
        data_type: String,
        properties: Map<Value>,
    },
    Unicode(String),
    Unit,
}

impl Value {
    ///
    ///
    ///
    pub fn data_type(&self) -> &str {
        use Value::*;
        match self {
            Array { data_type, .. } => data_type.as_str(),
            Boolean(_) => "boolean",
            Integer(_) => "integer",
            Pointer { data_type, .. } => data_type.as_str(),
            Real(_) => "real",
            Struct { data_type, .. } => data_type.as_str(),
            Unicode(_) => "unicode",
            Unit => "unit",
        }
    }
}

impl PartialEq for Value {
    ///
    ///
    ///
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        use Value::*;

        match (self, other) {
            (Array { .. }, Array { .. }) => unimplemented!(),
            (Boolean(lhs), Boolean(rhs)) => lhs.eq(rhs),
            (Integer(lhs), Integer(rhs)) => lhs.eq(rhs),
            (Pointer { .. }, Pointer { .. }) => unimplemented!(),
            (Real(lhs), Real(rhs)) => lhs.eq(rhs),
            (Struct { .. }, Struct { .. }) => unimplemented!(),
            (Unicode(lhs), Unicode(rhs)) => lhs.eq(rhs),
            (Unit, Unit) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    ///
    ///
    ///
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        use Value::*;

        match (self, other) {
            (Boolean(lhs), Boolean(rhs)) => lhs.partial_cmp(rhs),
            (Integer(lhs), Integer(rhs)) => lhs.partial_cmp(rhs),
            (Real(lhs), Real(rhs)) => lhs.partial_cmp(rhs),
            (Unicode(lhs), Unicode(rhs)) => lhs.partial_cmp(rhs),
            _ => Option::None,
        }
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    #[serde(rename = "type")]
    pub data_type: String,
    pub name: String,
    pub scope: Option<String>,
    pub value: Option<Value>,
}

impl Variable {
    ///
    ///
    ///
    pub fn new(
        name: String,
        data_type: String,
        scope: Option<String>,
        value: Option<Value>,
    ) -> Self {
        Variable {
            data_type,
            name,
            scope,
            value,
        }
    }
}
