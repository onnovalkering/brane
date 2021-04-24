#[macro_use]
extern crate log;

pub mod bytecode;
pub mod values;

use crate::bytecode::{Function, OpCode};
use crate::values::{Class, Value};
use std::{collections::HashMap, fmt::Write, usize};
use specifications::package::PackageIndex;

static FRAMES_MAX: usize = 64;
static STACK_MAX: usize = 256;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub slot_offset: usize,
    pub ip: usize,
    pub function: Function,
}

pub struct VM {
    call_frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    package_index: PackageIndex,
}

#[derive(Clone, Debug)]
pub struct VmCall {
    pub package: String,
    pub function: String,
    pub arguments: Vec<Value>,
}

#[repr(u8)]
pub enum VmResult {
    Ok(Option<Value>),
    Call(VmCall),
    RuntimeError,
}

impl VM {
    pub fn new(package_index: PackageIndex) -> VM {
        let mut globals = HashMap::new();
        globals.insert(
            String::from("print"),
            Value::Function(Function::Native {
                name: String::from("print"),
                arity: 1,
            }),
        );

        VM {
            call_frames: Vec::with_capacity(FRAMES_MAX),
            stack: Vec::with_capacity(STACK_MAX),
            globals,
            package_index,
        }
    }

    ///
    ///
    ///
    pub fn call(
        &mut self,
        function: Function,
        arg_count: usize,
    ) -> () {
        let new_frame = CallFrame {
            function,
            ip: 0,
            slot_offset: self.stack.len() - (arg_count - 1),
        };

        self.call_frames.push(new_frame);
    }

    ///
    ///
    ///
    pub fn result(
        &mut self,
        result: Value,
    ) -> () {
        self.stack.push(result);
    }

    ///
    ///
    ///
    pub fn run(
        &mut self,
        function: Option<Function>,
    ) -> VmResult {
        if let Some(function) = function {
            self.call_frames.push(CallFrame {
                slot_offset: 0,
                ip: 0,
                function,
            })
        }

        let mut frame = self.call_frames.last().unwrap().clone();
        let chunk = if let Function::UserDefined { chunk, .. } = frame.function.clone() {
            chunk
        } else {
            return VmResult::RuntimeError;
        };

        // Decodes and dispatches the instruction
        loop {
            let mut debug = String::from("          ");
            self.stack.iter().for_each(|v| write!(debug, "[ {:?} ]", v).unwrap());

            debug!("{}", debug);

            if frame.ip >= chunk.code.len() {
                self.call_frames.pop();
                return VmResult::Ok(None);
            }

            let instruction: OpCode = chunk.code[frame.ip].into();
            frame.ip = frame.ip + 1;

            use OpCode::*;
            match instruction {
                OpSetLocal => {
                    let index = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    let value = self.stack.pop().unwrap();
                    self.stack[frame.slot_offset + index as usize] = value;
                }
                OpSetGlobal => {
                    let ident = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    if let Some(ident) = chunk.constants.get(ident as usize) {
                        let value = self.stack.pop().unwrap();

                        if let Value::String(ident) = ident {
                            self.globals.insert(ident.clone(), value);
                        }
                    } else {
                        panic!("Tried to assign to undefined variable: {:?}", ident);
                    }
                }
                OpDefineGlobal => {
                    let ident = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    if let Some(ident) = chunk.constants.get(ident as usize) {
                        let value = self.stack.pop().unwrap();

                        if let Value::String(ident) = ident {
                            self.globals.insert(ident.clone(), value);
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpGetGlobal => {
                    let ident = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    if let Some(ident) = chunk.constants.get(ident as usize) {
                        if let Value::String(ident) = ident {
                            if let Some(value) = self.globals.get(ident) {
                                self.stack.push(value.clone());
                            } else {
                                panic!("Tried to access undefined variable: {:?}", ident);
                            }
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpGetLocal => {
                    let index = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    let local = self.stack.get_mut(frame.slot_offset + index as usize).unwrap().clone();
                    self.stack.push(local)
                }
                OpConstant => {
                    let constant = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    if let Some(value) = chunk.constants.get(constant as usize) {
                        self.stack.push(value.clone());
                    } else {
                        unreachable!()
                    }
                }
                OpClass => {
                    let class = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    if let Some(Value::String(name)) = chunk.constants.get(class as usize) {
                        let value = Value::Class(Class { name: name.clone() });

                        self.stack.push(value.clone());
                    } else {
                        unreachable!()
                    }
                }
                OpAdd => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs + rhs).into(),
                            (Value::Real(lhs), Value::Real(rhs)) => (lhs + rhs).into(),
                            (Value::Real(lhs), Value::Integer(rhs)) => (lhs + rhs as f64).into(),
                            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 + rhs).into(),
                            (Value::String(lhs), Value::String(rhs)) => (format!("{}{}", lhs, rhs)).into(),
                            (lhs, rhs) => {
                                println!("{:?} + {:?}", lhs, rhs);
                                unreachable!()
                            }
                        };

                        self.stack.push(value);
                    }
                }
                OpSubstract => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs - rhs).into(),
                            (Value::Real(lhs), Value::Real(rhs)) => (lhs - rhs).into(),
                            (Value::Real(lhs), Value::Integer(rhs)) => (lhs - rhs as f64).into(),
                            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 - rhs).into(),
                            _ => unreachable!(),
                        };

                        self.stack.push(value);
                    }
                }
                OpMultiply => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs * rhs).into(),
                            (Value::Real(lhs), Value::Real(rhs)) => (lhs * rhs).into(),
                            (Value::Real(lhs), Value::Integer(rhs)) => (lhs * rhs as f64).into(),
                            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 * rhs).into(),
                            _ => unreachable!(),
                        };

                        self.stack.push(value);
                    }
                }
                OpDivide => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs / rhs).into(),
                            (Value::Real(lhs), Value::Real(rhs)) => (lhs / rhs).into(),
                            (Value::Real(lhs), Value::Integer(rhs)) => (lhs / rhs as f64).into(),
                            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 / rhs).into(),
                            _ => unreachable!(),
                        };

                        self.stack.push(value);
                    }
                }
                OpNegate => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            Value::Integer(i) => self.stack.push((-i).into()),
                            Value::Real(r) => self.stack.push((-r).into()),
                            _ => unreachable!(),
                        }
                    }
                }
                OpReturn => {
                    let result = self.stack.pop();
                    dbg!(&result);

                    self.call_frames.pop();
                    // if self.call_frames.is_empty() {
                    //     return VmResult::Ok(None);
                    // }

                    return VmResult::Ok(result);
                }
                OpTrue => self.stack.push(true.into()),
                OpFalse => self.stack.push(false.into()),
                OpUnit => self.stack.push(().into()),
                OpNot => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            Value::Boolean(i) => self.stack.push((!i).into()),
                            Value::Unit => self.stack.push(true.into()),
                            _ => unreachable!(),
                        }
                    }
                }
                OpAnd => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Boolean(lhs), Value::Boolean(rhs)) => (lhs & rhs).into(),
                            _ => false.into(),
                        };

                        self.stack.push(value);
                    }
                }
                OpOr => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Boolean(lhs), Value::Boolean(rhs)) => (lhs | rhs).into(),
                            _ => false.into(),
                        };

                        self.stack.push(value);
                    }
                }
                OpEqual => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs == rhs).into(),
                            (Value::Real(lhs), Value::Real(rhs)) => (lhs == rhs).into(),
                            (Value::Boolean(lhs), Value::Boolean(rhs)) => (lhs == rhs).into(),
                            (Value::Unit, Value::Unit) => true.into(),
                            (Value::String(lhs), Value::String(rhs)) => (lhs == rhs).into(),
                            _ => false.into(),
                        };

                        self.stack.push(value);
                    }
                }
                OpGreater => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs > rhs).into(),
                            (Value::Real(lhs), Value::Real(rhs)) => (lhs > rhs).into(),
                            (Value::Real(lhs), Value::Integer(rhs)) => (lhs > rhs as f64).into(),
                            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 > rhs).into(),
                            _ => unreachable!(),
                        };

                        self.stack.push(value);
                    }
                }
                OpLess => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let value = match (lhs, rhs) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs < rhs).into(),
                            (Value::Real(lhs), Value::Real(rhs)) => (lhs < rhs).into(),
                            (Value::Real(lhs), Value::Integer(rhs)) => (lhs < rhs as f64).into(),
                            (Value::Integer(lhs), Value::Real(rhs)) => ((lhs as f64) < rhs).into(),
                            _ => unreachable!(),
                        };

                        self.stack.push(value);
                    }
                }
                OpPop => {
                    self.stack.pop();
                }
                OpJumpIfFalse => {
                    let offset1 = chunk.code[frame.ip] as u16;
                    frame.ip = frame.ip + 1;

                    let offset2 = chunk.code[frame.ip] as u16;
                    frame.ip = frame.ip + 1;

                    if let Some(Value::Boolean(false)) = self.stack.last() {
                        let offset = (offset1 << 8) | offset2;
                        frame.ip = frame.ip + offset as usize;
                    }
                }
                OpJump => {
                    let offset1 = chunk.code[frame.ip] as u16;
                    frame.ip = frame.ip + 1;

                    let offset2 = chunk.code[frame.ip] as u16;
                    frame.ip = frame.ip + 1;

                    let offset = (offset1 << 8) | offset2;
                    frame.ip = frame.ip + offset as usize;
                }
                OpJumpBack => {
                    let offset1 = chunk.code[frame.ip] as u16;
                    frame.ip = frame.ip + 1;

                    let offset2 = chunk.code[frame.ip] as u16;
                    frame.ip = frame.ip + 1;

                    let offset = (offset1 << 8) | offset2;
                    frame.ip = frame.ip - offset as usize;
                }
                OpCall => {
                    let arg_count = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    let offset = arg_count + 1;

                    let func = self.stack[self.stack.len() - offset as usize].clone();
                    let func = if let Value::Function(func) = func {
                        func
                    } else {
                        return VmResult::RuntimeError;
                    };

                    match func {
                        Function::UserDefined { .. } => {
                            self.call(func, arg_count as usize);
                            if let VmResult::Ok(Some(result)) = self.run(None) {
                                for _i in vec![0; offset as usize] {
                                    self.stack.pop();
                                }
                                self.stack.push(result);
                            }
                        }
                        Function::Native { name, .. } => match name.as_str() {
                            "print" => {
                                let value = self.stack.pop().unwrap();
                                println!("{:?}", value);
                                self.stack.pop();
                            }
                            _ => unreachable!(),
                        },
                        Function::External { name, package, arity } => {
                            let arguments = (0..arity)
                                .map(|_| self.stack.pop().unwrap())
                                .collect();

                            let call = VmCall {
                                package,
                                function: name,
                                arguments,
                            };

                            self.call_frames.pop();
                            self.call_frames.push(frame.clone());

                            return VmResult::Call(call);
                        }
                    }
                }
                OpImport => {
                    let constant = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    if let Some(Value::String(package_name)) = chunk.constants.get(constant as usize) {
                        if let Some(package) = self.package_index.get(package_name, None) {
                            if let Some(functions) = &package.functions {
                                for (name, function) in functions {
                                    self.globals.insert(
                                        name.clone(),
                                        Value::Function(Function::External {
                                            package: package_name.clone(),
                                            name: name.clone(),
                                            arity: function.parameters.len() as i32,
                                        }),
                                    );
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    }
}
