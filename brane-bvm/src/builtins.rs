use crate::{executor::VmExecutor, stack::Slot};
use specifications::common::Value;
use std::collections::HashMap;

const BUILTIN_PRINT_NAME: &str = "print";
const BUILTIN_PRINT_CODE: u8 = 0x01;

///
///
///
pub fn register(globals: &mut HashMap<String, Slot>) {
    // TODO: use a macro for this.

    globals.insert(String::from(BUILTIN_PRINT_NAME), Slot::BuiltIn(BUILTIN_PRINT_CODE));
}

///
///
///
#[inline]
pub async fn call<E>(
    builtin: u8,
    arguments: Vec<Value>,
    _executor: &E,
    _location: Option<String>,
) -> Value
where
    E: VmExecutor,
{
    match builtin {
        BUILTIN_PRINT_CODE => {
            let value = arguments.first().unwrap();
            println!("{}", value);

            Value::Unit
        }
        _ => unreachable!(),
    }
}
