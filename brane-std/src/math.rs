use anyhow::Result;
use brane_sys::System;
use specifications::common::{CallPattern, Function, Parameter, Value};
use specifications::package::PackageInfo;

type Map<T> = std::collections::HashMap<String, T>;
type Func = fn(&Map<Value>, &Box<dyn System>) -> Result<Value>;

lazy_static! {
    pub static ref PACKAGE: PackageInfo = {
        let name = String::from("math");
        let version = env!("CARGO_PKG_VERSION").into();
        let kind = String::from("std");

        let mut functions = Map::<Function>::new();

        // Add
        functions.insert(String::from("add1"), Function::new(
            vec![
                Parameter::new(String::from("left"), String::from("integer"), None, None, None),
                Parameter::new(String::from("right"), String::from("integer"), None, None, None)
            ],
            Some(CallPattern::new(None, Some(vec![String::from("+")]), None)),
            String::from("integer")
        ));
        functions.insert(String::from("add2"), Function::new(
            vec![
                Parameter::new(String::from("left"), String::from("real"), None, None, None),
                Parameter::new(String::from("right"), String::from("real"), None, None, None)
            ],
            Some(CallPattern::new(None, Some(vec![String::from("+")]), None)),
            String::from("real")
        ));

        // Substract
        functions.insert(String::from("substract1"), Function::new(
            vec![
                Parameter::new(String::from("left"), String::from("integer"), None, None, None),
                Parameter::new(String::from("right"), String::from("integer"), None, None, None)
            ],
            Some(CallPattern::new(None, Some(vec![String::from("-")]), None)),
            String::from("integer")
        ));
        functions.insert(String::from("substract2"), Function::new(
            vec![
                Parameter::new(String::from("left"), String::from("real"), None, None, None),
                Parameter::new(String::from("right"), String::from("real"), None, None, None)
            ],
            Some(CallPattern::new(None, Some(vec![String::from("-")]), None)),
            String::from("real")
        ));

        PackageInfo::new(name, version, None, kind, Some(functions), None)
    };

    pub static ref FUNCTIONS: Map<Func> = {
        let mut functions = Map::new();
        functions.insert(String::from("add1"), add as Func);
        functions.insert(String::from("add2"), add as Func);
        functions.insert(String::from("substract1"), substract as Func);
        functions.insert(String::from("substract2"), substract as Func);

        functions
    };
}

///
///
///
pub fn add(
    arguments: &Map<Value>,
    _system: &Box<dyn System>,
) -> Result<Value> {
    let left = arguments.get("left").expect("Missing `left` argument");
    let right = arguments.get("right").expect("Missing `right` argument");

    let result = match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a + b),
        (Value::Real(a), Value::Real(b)) => Value::Real(a + b),
        _ => unreachable!()
    };

    Ok(result)
}

///
///
///
pub fn substract(
    arguments: &Map<Value>,
    _system: &Box<dyn System>,
) -> Result<Value> {
    let left = arguments.get("left").expect("Missing `left` argument");
    let right = arguments.get("right").expect("Missing `right` argument");

    let result = match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
        (Value::Real(a), Value::Real(b)) => Value::Real(a - b),
        _ => unreachable!()
    };

    Ok(result)
}
