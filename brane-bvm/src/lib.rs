#[macro_use]
extern crate log;

pub mod bytecode;
pub mod values;

use crate::{bytecode::{Function, OpCode}, values::Instance};
use crate::values::Value;
use std::{collections::HashMap, fmt::Write, usize};
use specifications::package::PackageIndex;
use specifications::common::Value as SpecValue;
use std::cmp;

static FRAMES_MAX: usize = 64;
static STACK_MAX: usize = 256;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub slot_offset: usize,
    pub ip: usize,
    pub function: Function,
}

pub type VmState = HashMap<String, Value>;

pub struct VM {
    call_frames: Vec<CallFrame>,
    stack: Vec<Value>,
    package_index: PackageIndex,
    pub state: VmState,
    pub options: VmOptions,
}

#[derive(Clone, Debug, Default)]
pub struct VmOptions {
    pub always_return: bool
}

#[derive(Clone, Debug)]
pub struct VmCall {
    pub package: String,
    pub version: String,
    pub function: String,
    pub arguments: HashMap<String, SpecValue>,
}

#[repr(u8)]
pub enum VmResult {
    Ok(Option<Value>),
    Call(VmCall),
    RuntimeError,
}

impl VM {
    pub fn new<S: Into<String>>(application: S, package_index: PackageIndex, state: Option<VmState>, options: Option<VmOptions>) -> VM {
        let options = options.unwrap_or_default();
        let mut state = state.unwrap_or_default();
        state.insert(
            String::from("___application"),
            Value::String(application.into()),
        );

        state.insert(
            String::from("print"),
            Value::Function(Function::Native {
                name: String::from("print"),
                arity: 1,
            }),
        );

        VM {
            call_frames: Vec::with_capacity(FRAMES_MAX),
            stack: Vec::with_capacity(STACK_MAX),
            state,
            package_index,
            options,
        }
    }

    ///
    ///
    ///
    pub fn call(
        &mut self,
        function: Function,
        arg_count: u8,
    ) -> () {
        let new_frame = CallFrame {
            function,
            ip: 0,
            slot_offset: (cmp::max(0, (self.stack.len() as i16) - 1) - arg_count as i16) as usize,
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
        let mut counter = 0;
        loop {
            counter += 1;

            let mut debug = String::from(format!("{}         ", counter));
            self.stack.iter().for_each(|v| write!(debug, "[ {:?} ]", v).unwrap());
            debug!("{}", debug);

            if frame.ip >= chunk.code.len() {
                let result = if self.options.always_return {
                    self.stack.pop()
                } else {
                    None
                };

                self.stack.clear(); // Desired behaviour?
                return VmResult::Ok(result);
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
                            self.state.insert(ident.clone(), value);
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
                            self.state.insert(ident.clone(), value);
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
                            if let Some(value) = self.state.get(ident) {
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

                    if let Some(value) = chunk.constants.get(class as usize) {
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
                            self.call(func, arg_count);
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
                        Function::External { name, package, version, parameters } => {
                            let mut arguments: HashMap<String, SpecValue> = HashMap::new();
                            for p in parameters.iter() {
                                arguments.insert(p.name.clone(), self.stack.pop().unwrap().as_spec_value());
                            }

                            // The function itself.
                            self.stack.pop();

                            let call = VmCall {
                                package,
                                version,
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

                                    self.state.insert(
                                        name.clone(),
                                        Value::Function(Function::External {
                                            package: package_name.clone(),
                                            version: package.version.clone(),
                                            name: name.clone(),
                                            parameters: function.parameters.clone(),
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
                },
                OpNew => {
                    let properties_n = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    let class = self.stack.pop();
                    let mut properties = HashMap::new();

                    (0..properties_n).for_each(|_| {
                        let ident = self.stack.pop().unwrap();
                        let value = self.stack.pop().unwrap();

                        if let Value::String(ident) = ident {
                            properties.insert(ident, value);
                        }
                    });

                    if let Some(Value::Class(class)) = class {
                        let instance = Instance::new(class, Some(properties));
                        self.stack.push(Value::Instance(instance));
                    } else {
                        panic!("Not a class.");
                    }
                },
                OpDot => {
                    let target = self.stack.pop();

                    let property = chunk.code[frame.ip];
                    frame.ip = frame.ip + 1;

                    let property = if let Some(Value::String(property)) = chunk.constants.get(property as usize) {
                        property.clone()
                    } else {
                        warn!("constant not found!");
                        return VmResult::RuntimeError;
                    };

                    if let Some(Value::Instance(instance)) = target {
                        if let Some(property) = instance.fields.get(&property) {
                            self.stack.push(property.clone());
                        } else {
                            warn!("Property not found!");
                            return VmResult::RuntimeError;
                        }
                    } else {
                        warn!("Not an instance!");
                        return VmResult::RuntimeError;
                    }
                }
            }
        }
    }
}
