use crate::{executor::VmExecutor, stack::Slot, values::Value};
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
) -> Value
where
    E: VmExecutor,
{
    match builtin {
        BUILTIN_PRINT_CODE => {
            let value = arguments.first().unwrap();
            println!("{:?} (TODO: add Display to Value)", value);

            Value::Unit
        }
        _ => unreachable!(),
    }
}
