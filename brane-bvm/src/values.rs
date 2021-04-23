use crate::bytecode::Function;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Unit,
    Function(Function),
    Class(Class),
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
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
