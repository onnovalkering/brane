use std::collections::HashMap;
use crate::{builtins, bytecode::{self, opcodes::*}};
use crate::{frames::CallFrame};
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
        let mut vm = Self {
            frames,
            globals,
            heap,
            stack,
        };

        builtins::register(&mut vm.globals);

        vm
    }

    ///
    ///
    ///
    pub fn main(
        &mut self,
        function: bytecode::Function,
    ) {
        if self.frames.len() != 0 || self.stack.len() != 0 {
            panic!("VM not in a state to accept main function.");
        }

        let object = Object::Function(function.freeze(&mut self.heap));
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
        match function {
            Slot::BuiltIn(code) => builtins::call(*code, &mut self.stack),
            Slot::Object(handle) => {
                match self.heap.get(handle) {
                    Some(Object::Function(_f)) => {
                        // println!("{}", _f.chunk.disassemble().unwrap());

                        let frame = CallFrame::new(handle.clone(), frame_first);
                        self.frames.push(frame);
                    }
                    _ => panic!("Expecting a function."),
                }
            }
            _ => panic!("Not a callable object"),
        }
    }

    ///
    ///
    ///
    fn run(&mut self) -> Option<Slot> {
        while let Some(instruction) = self.next() {
            match *instruction {
                OP_ADD => self.op_add(),
                OP_CALL => self.op_call(),
                OP_CONSTANT => self.op_constant(),
                OP_DEFINE_GLOBAL => self.op_define_global(),
                OP_GET_GLOBAL => self.op_get_global(),
                OP_GET_LOCAL => self.op_get_local(),
                OP_GREATER => self.op_greater(),
                OP_JUMP => self.op_jump(),
                OP_JUMP_BACK => self.op_jump_back(),
                OP_JUMP_IF_FALSE => self.op_jump_if_false(),
                OP_NOT => self.op_not(),
                OP_POP => self.op_pop(),
                OP_RETURN => self.op_return(),
                OP_SUBSTRACT => self.op_substract(),
                x => {
                    println!("Unkown opcode: {}", x);
                    todo!();
                }
            }

            // println!("{}", self.stack);
        }

        None
    }

    ///
    ///
    ///
    #[inline]
    fn next(&mut self) -> Option<&u8> {
        self.frame().read_u8()
    }

    ///
    ///
    ///
    #[inline]
    fn frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("")
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_add(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        match (lhs, rhs) {
            (Slot::Integer(lhs), Slot::Integer(rhs)) => self.stack.push_integer(lhs + rhs),
            (Slot::Integer(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs as f64 + rhs),
            (Slot::Real(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs + rhs),
            (Slot::Real(lhs), Slot::Integer(rhs)) => self.stack.push_real(lhs + rhs as f64),
            _ => todo!(),
        };
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_call(&mut self) {
        let arity = self.frame().read_u8().expect("").clone();
        self.call(arity);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_constant(&mut self) {
        let constant = self
            .frame()
            .read_constant()
            .expect("Failed to read constant")
            .clone();

        self.stack.push(constant);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_define_global(&mut self) {
        let identifier = self.frame()
            .read_constant()
            .expect("Failed to read constant.")
            .clone();

        let value = self.stack.pop();

        if let Slot::Object(handle) = identifier {
            if let Some(Object::String(identifier)) = self.heap.get(handle) {
                self.globals.insert(identifier.clone(), value);
                return;
            }
        }

        panic!("Illegal identifier");
    }


    ///
    ///
    ///
    #[inline]
    pub fn op_get_global(&mut self) {
        let identifier = self.frame()
            .read_constant()
            .expect("Failed to read constant.")
            .clone();

        if let Slot::Object(handle) = identifier {
            if let Some(Object::String(identifier)) = self.heap.get(handle) {
                let value = self.globals.get(identifier).expect("Failed to retreive global.");
                self.stack.push(value.clone());
                return;
            }
        }

        panic!("Illegal identifier");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_get_local(&mut self) {
        let index = self.frame().read_u8().expect("Failed to read byte.").clone();
        let index = self.frame().stack_offset + index as usize;

        self.stack.copy_push(index);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_greater(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        self.stack.push_boolean(lhs > rhs);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_jump(&mut self) {
        let offset = self.frame().read_u16();
        self.frame().ip += offset as usize;
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_jump_back(&mut self) {
        let offset = self.frame().read_u16();
        self.frame().ip -= offset as usize;
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_jump_if_false(&mut self) {
        let truthy = self.stack.peek_boolean();
        if !truthy {
            self.op_jump();
        } else {
            // Skip the OP_JUMP_IF_FALSE operand.
            self.frame().ip += 2;
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_not(&mut self) {
        let value = self.stack.pop_boolean();
        self.stack.push_boolean(!value);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_pop(&mut self) {
        self.stack.pop();
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_return(&mut self) {
        if self.frames.len() == 1 {
            panic!("Cannot return outside a function.");
        }

        if let Some(frame) = self.frames.pop() {
            let return_value = self.stack.try_pop();
            self.stack.clear_from(frame.stack_offset);
            self.stack.try_push(return_value);
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_substract(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        match (lhs, rhs) {
            (Slot::Integer(lhs), Slot::Integer(rhs)) => self.stack.push_integer(lhs - rhs),
            (Slot::Integer(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs as f64 - rhs),
            (Slot::Real(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs - rhs),
            (Slot::Real(lhs), Slot::Integer(rhs)) => self.stack.push_real(lhs - rhs as f64),
            _ => todo!(),
        };
    }
}
