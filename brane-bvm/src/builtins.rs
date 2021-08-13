use crate::objects::{Class, Object};
use crate::{
    executor::{ServiceState, VmExecutor},
    stack::Slot,
};
use broom::Heap;
use fnv::FnvHashMap;
use specifications::common::Value;

const BUILTIN_PRINT_NAME: &str = "print";
const BUILTIN_PRINT_CODE: u8 = 0x01;

const BUILTIN_WAIT_UNTIL_STARTED_CODE: u8 = 0x02;
const BUILTIN_WAIT_UNTIL_DONE_CODE: u8 = 0x03;

const BUILTIN_SERVICE_NAME: &str = "Service";

///
///
///
pub fn register(
    globals: &mut FnvHashMap<String, Slot>,
    heap: &mut Heap<Object>,
) {
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
        name,
        methods: Default::default(),
    })
}

///
///
///
pub async fn call<E>(
    builtin: u8,
    arguments: Vec<Value>,
    executor: &E,
    _location: Option<String>,
) -> Value
where
    E: VmExecutor,
{
    match builtin {
        BUILTIN_PRINT_CODE => {
            let value = arguments.first().unwrap();
            let text = value.to_string();

            // Delegate printing to executor.
            executor.stdout(text).await.unwrap();

            Value::Unit
        }
        BUILTIN_WAIT_UNTIL_STARTED_CODE => {
            let instance = arguments.first().unwrap();
            if let Value::Struct { properties, .. } = arguments.first().unwrap() {
                let identifier = properties
                    .get("identifier")
                    .expect("Missing `identifier` property.")
                    .to_string();
                executor.wait_until(identifier, ServiceState::Started).await.unwrap();
            } else {
                dbg!(&instance);
                unreachable!();
            }

            Value::Unit
        }
        BUILTIN_WAIT_UNTIL_DONE_CODE => {
            let instance = arguments.first().unwrap();
            if let Value::Struct { properties, .. } = arguments.first().unwrap() {
                let identifier = properties
                    .get("identifier")
                    .expect("Missing `identifier` property.")
                    .to_string();
                executor.wait_until(identifier, ServiceState::Done).await.unwrap();
            } else {
                dbg!(&instance);
                unreachable!();
            }

            Value::Unit
        }
        _ => unreachable!(),
    }
}
