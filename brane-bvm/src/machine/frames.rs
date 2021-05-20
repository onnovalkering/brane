use crate::{objects::Object, stack::Slot};
use broom::Handle;

///
///
///
#[derive(Debug)]
pub struct CallFrame {
    pub function: Handle<Object>,
    pub ip: usize,
    pub stack_offset: usize,
}

impl CallFrame {
    ///
    ///
    ///
    pub fn new(
        function: Handle<Object>,
        stack_offset: usize,
    ) -> Self {
        Self {
            function,
            ip: 0,
            stack_offset,
        }
    }

    ///
    ///
    ///
    pub fn read_u8(&mut self) -> Option<&u8> {
        unsafe {
            let function = self.function.get_unchecked().as_function().unwrap();
            let byte = function.chunk.code.get(self.ip);

            self.ip += 1;
            byte
        }
    }

    pub fn read_u16(&mut self) -> u16 {
        unsafe {
            let function = self.function.get_unchecked().as_function().unwrap();

            let byte1 = function.chunk.code.get(self.ip).expect("Expecting a first byte.");
            self.ip += 1;

            let byte2 = function.chunk.code.get(self.ip).expect("Expecting a second byte.");
            self.ip += 1;

            ((*byte1 as u16) << 8) | (*byte2 as u16)
        }
    }

    ///
    ///
    ///
    pub fn read_constant(&mut self) -> Option<&Slot> {
        unsafe {
            let function = self.function.get_unchecked().as_function().unwrap();
            let index = function.chunk.code.get(self.ip).expect("");
            let constant = function.chunk.constants.get(*index as usize);

            self.ip += 1;
            constant
        }
    }
}
