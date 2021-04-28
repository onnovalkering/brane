use crate::bytecode::Function;
use specifications::common::Value as SpecValue;
use std::{collections::HashMap, fmt};

#[derive(Clone)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Unit,
    Function(Function),
    Class(Class),
    Instance(Instance),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(string) => write!(f, "\"{}\"", string),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Integer(integer) => write!(f, "{}", integer),
            Value::Real(real) => write!(f, "{}", real),
            Value::Unit => write!(f, "unit"),
            Value::Function(function) => write!(f, "{:?}", function),
            Value::Class(class) => write!(f, "{:?}", class),
            Value::Instance(instance) => write!(f, "{:?}", instance),
        }
    }
}


impl Value {
    pub fn as_spec_value(&self) -> SpecValue {
        match self {
            Value::String(value) => SpecValue::Unicode(value.clone()),
            Value::Boolean(value) => SpecValue::Boolean(value.clone()),
            Value::Integer(value) => SpecValue::Integer(value.clone()),
            Value::Real(value) => SpecValue::Real(value.clone()),
            Value::Unit => SpecValue::Unit,
            Value::Instance(instance) => {
                let data_type = instance.class.name.clone();
                let mut properties: HashMap<String, SpecValue> = HashMap::new();
                for (key, value) in &instance.fields {
                    properties.insert(key.clone(), value.as_spec_value());
                }

                SpecValue::Struct {
                    data_type,
                    properties
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct Class {
    pub name: String,
    pub properties: HashMap<String, String>,
}

impl Class {
    pub fn new(name: String, properties: HashMap<String, String>) -> Self {
        Self {
            name,
            properties,
        }
    }
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class<{}>", self.name)
    }
}

#[derive(Clone)]
pub struct Instance {
    pub class: Class,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Class, fields: Option<HashMap<String, Value>>) -> Self {
        let mut fields = fields.unwrap_or_default();
        fields.insert(String::from("__class"), class.name.clone().into());

        Self {
            class,
            fields,
        }
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "instance<{}>", self.class.name)
    }
}

impl From<SpecValue> for Value {
    fn from(value: SpecValue) -> Self {
        match value {
            SpecValue::Unicode(value) => Value::String(value.clone()),
            SpecValue::Boolean(value) => Value::Boolean(value.clone()),
            SpecValue::Integer(value) => Value::Integer(value.clone()),
            SpecValue::Real(value) => Value::Real(value.clone()),
            SpecValue::Unit => Value::Unit,
            SpecValue::Struct { data_type, properties } => {
                let mut fields: HashMap<String, Value> = HashMap::new();
                for (key, spec_value) in &properties {
                    fields.insert(key.clone(), Value::from(spec_value.clone()));
                }

                let class  = Class { name: data_type.clone(), properties: HashMap::new() };
                Value::Instance(Instance { class, fields })
            }
            _ => unreachable!(),
        }
    }
}

impl From<Function> for Value {
    fn from(function: Function) -> Self {
        Value::Function(function)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::String(string)
    }
}

impl From<bool> for Value {
    fn from(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }
}

impl From<i64> for Value {
    fn from(integer: i64) -> Self {
        Value::Integer(integer)
    }
}

impl From<f64> for Value {
    fn from(real: f64) -> Self {
        Value::Real(real)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Unit
    }
}
