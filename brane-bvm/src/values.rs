use crate::bytecode::FunctionMut;
use specifications::common::{SpecClass, Value as SpecValue};
use std::{collections::HashMap, fmt};

#[derive(Clone)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Unit,
    Function(FunctionMut),
    Class(SpecClass),
    Instance(Instance),
    Array(Array),
}

impl fmt::Debug for Value {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Value::String(string) => write!(f, "\"{}\"", string),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Integer(integer) => write!(f, "{}", integer),
            Value::Real(real) => write!(f, "{}", real),
            Value::Unit => write!(f, "unit"),
            Value::Function(_) => write!(f, "{:?}", "func(..)"),
            Value::Class(class) => write!(f, "{:?}", class),
            Value::Instance(instance) => write!(f, "{:?}", instance),
            Value::Array(array) => write!(f, "{:?}", array),
        }
    }
}

impl Value {
    pub fn as_spec_value(&self) -> SpecValue {
        match self {
            Value::String(value) => SpecValue::Unicode(value.clone()),
            Value::Boolean(value) => SpecValue::Boolean(*value),
            Value::Integer(value) => SpecValue::Integer(*value),
            Value::Real(value) => SpecValue::Real(*value),
            Value::Unit => SpecValue::Unit,
            Value::Instance(instance) => {
                let data_type = instance.class.name.clone();
                let mut properties: HashMap<String, SpecValue> = HashMap::new();
                for (key, value) in &instance.fields {
                    properties.insert(key.clone(), value.as_spec_value());
                }

                SpecValue::Struct { data_type, properties }
            }
            Value::Array(array) => {
                let entries = array.entries.iter().map(|e| e.as_spec_value()).collect();
                SpecValue::Array {
                    data_type: array.data_type.clone(),
                    entries,
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct Array {
    pub data_type: String,
    pub entries: Vec<Value>,
}

impl fmt::Debug for Array {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.data_type)
    }
}

#[derive(Clone)]
pub struct Instance {
    pub class: SpecClass,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(
        class: SpecClass,
        fields: Option<HashMap<String, Value>>,
    ) -> Self {
        let fields = fields.unwrap_or_default();
        // fields.insert(String::from("__class"), class.name.clone().into());

        Self { class, fields }
    }
}

impl fmt::Debug for Instance {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "instance<{}>", self.class.name)
    }
}

impl From<SpecValue> for Value {
    fn from(value: SpecValue) -> Self {
        match value {
            SpecValue::Unicode(value) => Value::String(value),
            SpecValue::Boolean(value) => Value::Boolean(value),
            SpecValue::Integer(value) => Value::Integer(value),
            SpecValue::Real(value) => Value::Real(value),
            SpecValue::Unit => Value::Unit,
            SpecValue::Struct { data_type, properties } => {
                let mut fields: HashMap<String, Value> = HashMap::new();
                for (key, spec_value) in &properties {
                    fields.insert(key.clone(), Value::from(spec_value.clone()));
                }

                let class = SpecClass {
                    name: data_type,
                    properties: HashMap::new(),
                    methods: HashMap::new(),
                };
                Value::Instance(Instance { class, fields })
            }
            SpecValue::Array { data_type, entries } => {
                let entries = entries.iter().map(|e| Value::from(e.clone())).collect();

                Value::Array(Array { data_type, entries })
            }
            _ => unreachable!(),
        }
    }
}
