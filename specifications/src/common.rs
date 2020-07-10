use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JValue};
use serde_with::skip_serializing_none;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::fmt::{self, Display, Formatter};

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
    pub secret: Option<String>,
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
        secret: Option<String>,
    ) -> Self {
        Parameter {
            data_type,
            default,
            name,
            optional,
            secret,
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
    ///
    ///
    ///
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

    ///
    ///
    ///
    pub fn into_parameter(self) -> Parameter {
        Parameter::new(self.name, self.data_type, self.optional, self.default, None)
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
        #[serde(default = "Default::default")]
        secret: bool,
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
    pub fn from_json(value: &JValue) -> Self {
        match value {
            JValue::Null => Value::Unit,
            JValue::Bool(b) => Value::Boolean(*b),
            JValue::Number(n) => {
                if n.is_i64() {
                    Value::Integer(n.as_i64().unwrap())
                } else {
                    Value::Real(n.as_f64().unwrap())
                }
            }
            JValue::String(s) => Value::Unicode(s.clone()),
            JValue::Array(a) => {
                let entries: Vec<Value> = a.iter().map(|v| Value::from_json(v)).collect();
                let data_type = format!("{}[]", entries.first().unwrap().data_type());

                Value::Array { data_type, entries }
            }
            JValue::Object(o) => {
                let mut properties = Map::<Value>::new();
                let data_type = String::from("anonymous");

                for (name, jvalue) in o.iter() {
                    properties.insert(name.clone(), Value::from_json(jvalue));
                }

                Value::Struct { data_type, properties }
            }
        }
    }

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
            Unicode(_) => "string",
            Unit => "unit",
        }
    }

    ///
    ///
    ///
    pub fn as_bool(&self) -> Result<bool> {
        if let Value::Boolean(b) = self {
            Ok(*b)
        } else {
            Err(anyhow!("Value does not contain a boolean."))
        }
    }

    ///
    ///
    ///
    pub fn as_f64(&self) -> Result<f64> {
        if let Value::Real(f) = self {
            Ok(*f)
        } else {
            Err(anyhow!("Value does not contain a real (float)."))
        }
    }

    ///
    ///
    ///
    pub fn as_i64(&self) -> Result<i64> {
        if let Value::Integer(i) = self {
            Ok(*i)
        } else {
            Err(anyhow!("Value does not contain an integer."))
        }
    }

    ///
    ///
    ///
    pub fn as_string(&self) -> Result<String> {
        if let Value::Unicode(s) = self {
            Ok(s.clone())
        } else {
            Err(anyhow!("Value does not contain a string."))
        }
    }

    ///
    ///
    ///
    pub fn as_json(&self) -> JValue {
        use Value::*;
        match self {
            Array { .. } => unimplemented!(),
            Boolean(b) => json!(b),
            Integer(i) => json!(i),
            Pointer { .. } => unimplemented!(),
            Real(r) => json!(r),
            Struct { .. } => unimplemented!(),
            Unicode(s) => json!(s),
            Unit => json!(null),
        }
    }
}

impl Display for Value {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        use Value::*;
        let value = match self {
            Array { entries, .. } => {
                let entries = entries
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("[{}]", entries)
            }
            Boolean(b) => b.to_string(),
            Integer(i) => i.to_string(),
            Pointer { variable, .. } => format!("@{}", variable),
            Real(r) => r.to_string(),
            Struct { properties, .. } => {
                let properties = properties
                    .iter()
                    .map(|(n, p)| format!("{}: {}", n, p.to_string()))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{{{}}}", properties)
            }
            Unicode(s) => s.to_string(),
            Unit => String::from("unit"),
        };

        write!(f, "{}", value)
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
