use anyhow::Result;
use specifications::common::{CallPattern, Function, Parameter, Value};
use specifications::package::PackageInfo;

type Map<T> = std::collections::HashMap<String, T>;
type Func = fn(&Map<Value>) -> Result<Value>;

lazy_static! {
    pub static ref PACKAGE: PackageInfo = {
        let name = String::from("text");
        let version = String::from("1.0.0");
        let kind = String::from("std");

        let mut functions = Map::<Function>::new();

        // Concat
        functions.insert(String::from("concat1"), Function::new(
            vec![
                Parameter::new(String::from("left"), String::from("integer"), None, None, None),
                Parameter::new(String::from("right"), String::from("string"), None, None, None)
            ],
            Some(CallPattern::new(None, Some(vec![String::from("+")]), None)),
            String::from("string")
        ));

        functions.insert(String::from("concat2"), Function::new(
            vec![
                Parameter::new(String::from("left"), String::from("string"), None, None, None),
                Parameter::new(String::from("right"), String::from("integer"), None, None, None)
            ],
            Some(CallPattern::new(None, Some(vec![String::from("+")]), None)),
            String::from("string")
        ));

        functions.insert(String::from("concat3"), Function::new(
            vec![
                Parameter::new(String::from("left"), String::from("string"), None, None, None),
                Parameter::new(String::from("right"), String::from("string"), None, None, None)
            ],
            Some(CallPattern::new(None, Some(vec![String::from("+")]), None)),
            String::from("string")
        ));

        // Split
        functions.insert(String::from("split"), Function::new(
            vec![Parameter::new(String::from("input"), String::from("string"), None, None, None)],
            Some(CallPattern::new(Some(String::from("split")), None, None)),
            String::from("string[]")
        ));

        PackageInfo::new(name, version, None, kind, Some(functions), None)
    };

    pub static ref FUNCTIONS: Map<Func> = {
        let mut functions = Map::new();
        functions.insert(String::from("concat1"), concat as Func);
        functions.insert(String::from("concat2"), concat as Func);
        functions.insert(String::from("concat3"), concat as Func);
        functions.insert(String::from("split"), split as Func);

        functions
    };
}

///
///
///
pub fn concat(arguments: &Map<Value>) -> Result<Value> {
    let left = arguments.get("left").expect("Missing `left` argument");
    let right = arguments.get("right").expect("Missing `right` argument");

    Ok(Value::Unicode(format!("{}{}", left.to_string(), right.to_string())))
}

///
///
///
pub fn split(arguments: &Map<Value>) -> Result<Value> {
    let input = arguments.get("input").expect("Missing `input` argument.");
    if let Value::Unicode(text) = input {
        let data_type = String::from("string[]");
        let entries: Vec<Value> = text.split(' ').map(|e| Value::Unicode(e.to_string())).collect();

        Ok(Value::Array { data_type, entries })
    } else {
        unreachable!();
    }
}
