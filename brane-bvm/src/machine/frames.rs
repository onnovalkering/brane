use crate::objects::Object;
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
}
