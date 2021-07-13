use crate::objects::{Class, Object};
use crate::{executor::VmExecutor, stack::Slot};
use broom::Heap;
use fnv::FnvHashMap;
use specifications::common::Value;

const BUILTIN_PRINT_NAME: &str = "print";
const BUILTIN_PRINT_CODE: u8 = 0x01;
const BUILTIN_SERVICE_NAME: &str = "Service";

///
///
///
pub fn register(globals: &mut FnvHashMap<String, Slot>, heap: &mut Heap<Object>) {
    // Classes
    let service = heap.insert(class(BUILTIN_SERVICE_NAME.to_string())).into_handle();
    globals.insert(BUILTIN_SERVICE_NAME.to_string(), Slot::Object(service));

    // Functions
    globals.insert(String::from(BUILTIN_PRINT_NAME), Slot::BuiltIn(BUILTIN_PRINT_CODE));
}

///
///
///
fn class(name: String) -> Object {
    Object::Class(Class {
        name
    })
}

///
///
///
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
