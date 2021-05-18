use crate::bytecode::Function;
use crate::values::{Array, Value};
use anyhow::Result;
use std::collections::HashMap;

static BUILTIN_FN_PRINT: &str = "print";
static BUILTIN_FN_LENGTH: &str = "length";
static BUILTIN_FN_SPLIT: &str = "split";

///
///
///
pub fn register(state: &mut HashMap<String, Value>) {
    // TODO: use a macro ?

    state.insert(
        BUILTIN_FN_PRINT.to_string(),
        Value::Function(Function::Native {
            name: BUILTIN_FN_PRINT.to_string(),
            arity: 1,
        }),
    );

    state.insert(
        BUILTIN_FN_LENGTH.to_string(),
        Value::Function(Function::Native {
            name: BUILTIN_FN_LENGTH.to_string(),
            arity: 1,
        }),
    );

    state.insert(
        BUILTIN_FN_SPLIT.to_string(),
        Value::Function(Function::Native {
            name: BUILTIN_FN_SPLIT.to_string(),
            arity: 2,
        }),
    );
}

///
///
///
pub fn handle(
    name: String,
    stack: &mut Vec<Value>,
) -> Result<()> {
    match name.as_str() {
        "print" => {
            let value = stack.pop().unwrap();
            println!("{:?}", value);
            stack.pop();
        }
        "length" => {
            let value = stack.pop().unwrap();
            stack.pop(); // function

            let length = match value {
                Value::Array(Array { entries, .. }) => entries.len(),
                Value::String(string) => string.len(),
                _ => 0,
            };

            stack.push(Value::Integer(length as i64));
        }
        "split" => {
            let seperator = stack.pop().unwrap();
            let value = stack.pop().unwrap();
            stack.pop(); // function

            if let Value::String(value) = value {
                if let Value::String(seperator) = seperator {
                    let entries = value.split(&seperator).map(|e| Value::String(e.to_string())).collect();
                    stack.push(Value::Array(Array {
                        entries,
                        data_type: String::from("string[]"),
                    }));

                    return Ok(());
                }
            }

            stack.push(Value::Array(Array {
                entries: vec![],
                data_type: String::from("string[]"),
            }))
        }
        _ => unreachable!(),
    }

    Ok(())
}
