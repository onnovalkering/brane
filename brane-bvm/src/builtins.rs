use crate::stack::{Slot, Stack};
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
pub fn call(
    builtin: u8,
    stack: &mut Stack,
) {
    match builtin {
        BUILTIN_PRINT_CODE => {
            let value = stack.pop();
            println!("{}", value);
        },
        _ => unreachable!()
    }

    // Remove builtin from stack.
    stack.pop();
}
