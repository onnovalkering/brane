#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate firestorm;

pub mod builtins;
pub mod bytecode;
pub mod instructions;
pub mod values;

use crate::values::{Array, Value};
use crate::{
    bytecode::{opcodes::*, Function},
};
use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use bytecode::{ReadOnlyChunk};
use futures::{stream, StreamExt};
use specifications::common::Value as SpecValue;
use specifications::package::PackageIndex;
use std::cmp;
use std::{collections::HashMap, usize};
use smallvec::SmallVec;

static FRAMES_MAX: usize = 64;
static STACK_MAX: usize = 64;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub slot_offset: usize,
    pub ip: usize,
    pub chunk: ReadOnlyChunk,
}

pub type VmState = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct VM<E>
where
    E: VmExecutor + Clone + Send + Sync,
{
    call_frames: Vec<CallFrame>,
    stack: SmallVec<[Value; 64]>,
    locations: Vec<String>,
    package_index: PackageIndex,
    pub state: VmState,
    pub options: VmOptions,
    executor: E,
}

#[derive(Clone, Debug, Default)]
pub struct VmOptions {
    pub always_return: bool,
}

#[derive(Clone, Debug)]
pub struct VmCall {
    pub package: String,
    pub kind: String,
    pub version: String,
    pub function: String,
    pub location: Option<String>,
    pub arguments: HashMap<String, SpecValue>,
}

#[async_trait]
pub trait VmExecutor {
    async fn execute(
        &self,
        call: VmCall,
    ) -> Result<Value>;
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum VmResult {
    Ok(Option<Value>),
    RuntimeError,
}

impl<E> VM<E>
where
    E: 'static + VmExecutor + Clone + Send + Sync,
{
    pub fn new<S>(
        application: S,
        package_index: PackageIndex,
        state: Option<VmState>,
        options: Option<VmOptions>,
        executor: E,
    ) -> VM<E>
    where
        S: Into<String>,
        E: VmExecutor,
    {
        let options = options.unwrap_or_default();
        let mut state = state.unwrap_or_default();
        state.insert(String::from("___application"), Value::String(application.into()));

        builtins::register(&mut state);

        VM {
            call_frames: Vec::with_capacity(FRAMES_MAX),
            stack: SmallVec::new(),
            state,
            locations: Vec::with_capacity(STACK_MAX),
            package_index,
            options,
            executor,
        }
    }

    ///
    ///
    ///
    pub fn call(
        &mut self,
        chunk: ReadOnlyChunk,
        arg_count: u8,
    ) -> () {
        let new_frame = CallFrame {
            chunk,
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
    #[async_recursion]
    pub async fn run(
        &mut self,
        function: Option<Function>,
    ) -> Result<VmResult> {
        if let Some(Function::UserDefined { chunk, ..}) = function {
            self.call(chunk, 0);
        }

        let frame = self.call_frames.last().unwrap().clone();
        let mut ip = 0;

        // Decodes and dispatches the instruction
        // let mut counter: i32 = 0;
        loop {
            // counter += 1;

            // let mut debug = String::from(format!("{}         ", counter));
            // self.stack.iter().for_each(|v| write!(debug, "[ {:?} ]", v).unwrap());
            // debug!("{}", debug);

            if ip >= frame.chunk.code.len() {
                let result = if self.options.always_return {
                    self.stack.pop()
                } else {
                    None
                };

                self.stack.clear(); // Desired behaviour?
                return Ok(VmResult::Ok(result));
            }

            let instruction: u8 = frame.chunk.code[ip];
            ip += 1;

            match instruction {
                OP_CONSTANT => ip = instructions::op_constant(ip, &frame, &mut self.stack)?,
                OP_GET_LOCAL => ip = instructions::op_get_local(ip, &frame, &mut self.stack)?,
                OP_SET_LOCAL => ip = instructions::op_set_local(ip, &frame, &mut self.stack)?,
                OP_DEFINE_GLOBAL => ip = instructions::op_define_global(ip, &frame, &mut self.stack, &mut self.state)?,
                OP_GET_GLOBAL => ip = instructions::op_get_global(ip, &frame, &mut self.stack, &mut self.state)?,
                OP_SET_GLOBAL => ip = instructions::op_set_global(ip, &frame, &mut self.stack, &mut self.state)?,
                OP_CLASS => ip = instructions::op_class(ip, &frame, &mut self.stack)?,
                OP_ADD => instructions::op_add(&mut self.stack)?,
                OP_SUBSTRACT => instructions::op_substract(&mut self.stack)?,
                OP_MULTIPLY => instructions::op_multiply(&mut self.stack)?,
                OP_DIVIDE => instructions::op_divide(&mut self.stack)?,
                OP_NEGATE => instructions::op_negate(&mut self.stack)?,
                OP_TRUE => instructions::op_true(&mut self.stack)?,
                OP_FALSE => instructions::op_false(&mut self.stack)?,
                OP_UNIT => instructions::op_unit(&mut self.stack)?,
                OP_NOT => instructions::op_not(&mut self.stack)?,
                OP_AND => instructions::op_and(&mut self.stack)?,
                OP_OR => instructions::op_or(&mut self.stack)?,
                OP_EQUAL => instructions::op_equal(&mut self.stack)?,
                OP_GREATER => instructions::op_greater(&mut self.stack)?,
                OP_LESS => instructions::op_less(&mut self.stack)?,
                OP_POP => instructions::op_pop(&mut self.stack)?,
                OP_RETURN => {
                    let value = instructions::op_return(&mut self.stack, &mut self.call_frames)?;
                    return Ok(VmResult::Ok(Some(value)));
                }
                OP_LOC_PUSH => instructions::op_loc_push(&mut self.stack, &mut self.locations)?,
                OP_LOC_POP => instructions::op_loc_pop(&mut self.locations)?,
                OP_JUMP => ip = instructions::op_jump(ip, &frame)?,
                OP_JUMP_BACK => ip = instructions::op_jump_back(ip, &frame)?,
                OP_JUMP_IF_FALSE => ip = instructions::op_jump_if_false(ip, &frame, &mut self.stack)?,
                OP_IMPORT => ip = instructions::op_import(ip, &frame, &mut self.state, &self.package_index)?,
                OP_NEW => ip = instructions::op_new(ip, &frame, &mut self.stack)?,
                OP_ARRAY => ip = instructions::op_array(ip, &frame, &mut self.stack)?,
                OP_DOT => ip = instructions::op_dot(ip, &frame, &mut self.stack)?,
                OP_INDEX => instructions::op_index(&mut self.stack)?,
                //
                //
                //
                OP_CALL => {
                    profile_section!(op_call);

                    if let Some(arg_count) = frame.chunk.code.get(ip) {
                        ip += 1;

                        let offset = *arg_count as usize + 1;
                        let function = self.stack.get(self.stack.len() - offset);

                        match function {
                            Some(Value::Function(Function::UserDefined { ref chunk, .. })) => {
                                let chunk = chunk.clone();
                                self.call(chunk, *arg_count);

                                if let Ok(VmResult::Ok(Some(result))) = self.run(None).await {
                                    for _i in vec![0; offset as usize] {
                                        self.stack.pop();
                                    }

                                    self.stack.push(result);
                                }
                            }
                            Some(Value::Function(Function::Native { ref name, .. })) => {
                                let name = name.clone();
                                builtins::handle(name, &mut self.stack).unwrap();
                            }
                            Some(Value::Function(Function::External {
                                name,
                                package,
                                kind,
                                version,
                                parameters,
                            })) => {
                                let name = name.clone();
                                let package = package.clone();
                                let kind = kind.clone();
                                let version = version.clone();
                                let parameters = parameters.clone();

                                let mut arguments: HashMap<String, SpecValue> = HashMap::new();
                                // Reverse order because of stack
                                for p in parameters.into_iter().rev() {
                                    arguments.insert(p.name.clone(), self.stack.pop().unwrap().as_spec_value());
                                }

                                // The function itself.
                                self.stack.pop();
                                let location = self.locations.last().cloned();

                                let call = VmCall {
                                    package: package.clone(),
                                    version: version.clone(),
                                    kind: kind.clone(),
                                    location,
                                    function: name.clone(),
                                    arguments,
                                };

                                self.call_frames.pop();
                                self.call_frames.push(frame.clone());

                                let result = self.executor.execute(call).await.unwrap();
                                self.stack.push(result);
                            },
                            _ => bail!("Trying to call something that's not a function."),
                        }
                    }
                }
                //
                //
                //
                OP_PARALLEL => {
                    let blocks_n = frame.chunk.code[ip];
                    ip += 1;

                    let blocks: Vec<Value> = (0..blocks_n).map(|_| self.stack.pop().unwrap()).rev().collect();

                    if blocks.is_empty() {
                        self.stack.push(Value::Array(Array {
                            data_type: String::from("unit[]"),
                            entries: blocks,
                        }));
                    } else {
                        let results = stream::iter(blocks)
                            .map(|block| {
                                let mut fork = self.clone();
                                if let Value::Function(block) = block {
                                    tokio::spawn(async move { fork.run(Some(block)).await })
                                } else {
                                    unreachable!()
                                }
                            })
                            .buffer_unordered(128)
                            .collect::<Vec<_>>()
                            .await;

                        if results.iter().any(|r| match r {
                            Ok(Ok(VmResult::RuntimeError)) => true,
                            Err(_) => true,
                            _ => false,
                        }) {
                            return Ok(VmResult::RuntimeError);
                        }

                        let entries: Vec<Value> = results
                            .into_iter()
                            .map(|r| match r {
                                Ok(Ok(VmResult::Ok(value))) => value.unwrap_or(Value::Unit),
                                _ => unreachable!(),
                            })
                            .collect();

                        let data_type = match entries.get(0).unwrap() {
                            Value::String(_) => String::from("string"),
                            Value::Real(_) => String::from("real"),
                            Value::Integer(_) => String::from("integer"),
                            Value::Boolean(_) => String::from("boolean"),
                            Value::Array(array) => array.data_type.clone(),
                            Value::Instance(instance) => instance.class.name.clone(),
                            Value::Class(_) | Value::Function(_) => todo!(),
                            Value::Unit => String::from("unit"),
                        };

                        let data_type = format!("{}[]", data_type);
                        self.stack.push(Value::Array(Array { data_type, entries }));
                    }
                }
                0x00 | 0x24..=u8::MAX => {
                    unreachable!()
                }
            }
        }
    }
}
