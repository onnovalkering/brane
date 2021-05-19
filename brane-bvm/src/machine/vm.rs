use std::collections::HashMap;

use crate::{frames::CallFrame, objects::Function};
use crate::objects::Object;
use crate::stack::{Slot, Stack};
use broom::Heap;

///
///
///
pub struct Vm {
    frames: Vec<CallFrame>,
    globals: HashMap<String, Slot>,
    heap: Heap<Object>,
    stack: Stack,
}

impl Default for Vm {
    fn default() -> Self {
        let frames = Vec::default();
        let globals = HashMap::default();
        let heap = Heap::default();
        let stack = Stack::default();

        Self::new(frames, globals, heap, stack)
    }
}

impl Vm {
    ///
    ///
    ///
    pub fn new(
        frames: Vec<CallFrame>,
        globals: HashMap<String, Slot>,
        heap: Heap<Object>,
        stack: Stack,
    ) -> Self {
        let vm = Self {
            frames,
            globals,
            heap,
            stack,
        };
        // TODO: load built-ins

        vm
    }

    ///
    ///
    ///
    pub fn main(
        &mut self,
        function: Function,
    ) {
        if self.frames.len() != 0 || self.stack.len() != 0 {
            panic!("VM not in a state to accept main function.");
        }

        let object = Object::Function(function);
        let handle = self.heap.insert(object).into_handle();

        self.stack.push_object(handle);
        self.call(0);
        self.run();
    }

    ///
    ///
    ///
    fn call(
        &mut self,
        arity: u8,
    ) {
        let frame_last = self.stack.len();
        let frame_first = frame_last - (arity + 1) as usize;
        let function = self.stack.get(frame_first);

        if let Slot::Object(handle) = function {
            match self.heap.get(handle) {
                Some(Object::Function(_)) => {
                    let frame = CallFrame::new(handle.clone(), frame_first);
                    self.frames.push(frame);
                }
                _ => panic!("Expecting a function."),
            }
        }
    }

    fn run(&mut self) {
        while let Some(instruction) = self.next() {
            dbg!(&instruction);
        }
    }

    #[inline]
    fn next(&mut self) -> Option<&u8> {
        self.current_frame().read_u8()
    }

    #[inline]
    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("")
    }
}
