use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use specifications::common::Parameter;
use std::fmt::{self, Write};

pub mod opcodes {
    pub const OP_ADD: u8 = 0x01;
    pub const OP_AND: u8 = 0x02;
    pub const OP_ARRAY: u8 = 0x03;
    pub const OP_CALL: u8 = 0x04;
    pub const OP_CLASS: u8 = 0x05;
    pub const OP_CONSTANT: u8 = 0x06;
    pub const OP_DEFINE_GLOBAL: u8 = 0x07;
    pub const OP_DIVIDE: u8 = 0x08;
    pub const OP_DOT: u8 = 0x09;
    pub const OP_EQUAL: u8 = 0x0A;
    pub const OP_FALSE: u8 = 0x0B;
    pub const OP_GET_GLOBAL: u8 = 0x0C;
    pub const OP_GET_LOCAL: u8 = 0x0D;
    pub const OP_GREATER: u8 = 0x0E;
    pub const OP_IMPORT: u8 = 0x0F;
    pub const OP_INDEX: u8 = 0x10;
    pub const OP_JUMP: u8 = 0x11;
    pub const OP_JUMP_BACK: u8 = 0x12;
    pub const OP_JUMP_IF_FALSE: u8 = 0x13;
    pub const OP_LESS: u8 = 0x14;
    pub const OP_LOC_POP: u8 = 0x15;
    pub const OP_LOC_PUSH: u8 = 0x16;
    pub const OP_MULTIPLY: u8 = 0x17;
    pub const OP_NEGATE: u8 = 0x18;
    pub const OP_NEW: u8 = 0x19;
    pub const OP_NOT: u8 = 0x1A;
    pub const OP_OR: u8 = 0x1B;
    pub const OP_PARALLEL: u8 = 0x1C;
    pub const OP_POP: u8 = 0x1D;
    pub const OP_RETURN: u8 = 0x1E;
    pub const OP_SET_GLOBAL: u8 = 0x1F;
    pub const OP_SET_LOCAL: u8 = 0x20;
    pub const OP_SUBSTRACT: u8 = 0x21;
    pub const OP_TRUE: u8 = 0x22;
    pub const OP_UNIT: u8 = 0x23;
}

use crate::values::Value;
use opcodes::*;

#[derive(Clone)]
pub enum Function {
    External {
        package: String,
        version: String,
        kind: String,
        name: String,
        parameters: Vec<Parameter>,
    },
    Native {
        name: String,
        arity: u8,
    },
    UserDefined {
        name: String,
        arity: u8,
        chunk: ReadOnlyChunk,
    },
}

impl Function {
    pub fn new(
        name: String,
        arity: u8,
        chunk: ReadOnlyChunk,
    ) -> Self {
        Function::UserDefined { arity, name, chunk }
    }
}

impl fmt::Debug for Function {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Function::UserDefined { name, .. } | Function::External { name, .. } | Function::Native { name, .. } => {
                write!(f, "{}(..)", name)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadOnlyChunk {
    pub code: Bytes,
    pub constants: Vec<Value>,
}

impl ReadOnlyChunk {
    ///
    ///
    ///
    pub fn disassemble(&self) -> Result<String> {
        let mut result = String::new();
        let mut skip = 0;

        for (offset, instruction) in self.code.iter().enumerate() {
            if skip > 0 {
                skip = skip - 1;
                continue;
            }

            write!(result, "{:04} ", offset)?;
            match *instruction {
                OP_CONSTANT => {
                    constant_instruction("OP_CONSTANT", &self, offset, &mut result);
                    skip = 1;
                }
                OP_ADD => {
                    writeln!(result, "OP_ADD")?;
                }
                OP_AND => {
                    writeln!(result, "OP_AND")?;
                }
                OP_DIVIDE => {
                    writeln!(result, "OP_DIVIDE")?;
                }
                OP_EQUAL => {
                    writeln!(result, "OP_EQUAL")?;
                }
                OP_FALSE => {
                    writeln!(result, "OP_FALSE")?;
                }
                OP_GREATER => {
                    writeln!(result, "OP_GREATER")?;
                }
                OP_LESS => {
                    writeln!(result, "OP_LESS")?;
                }
                OP_MULTIPLY => {
                    writeln!(result, "OP_MULTIPLY")?;
                }
                OP_NEGATE => {
                    writeln!(result, "OP_NEGATE")?;
                }
                OP_NOT => {
                    writeln!(result, "OP_NOT")?;
                }
                OP_OR => {
                    writeln!(result, "OP_OR")?;
                }
                OP_POP => {
                    writeln!(result, "OP_POP")?;
                }
                OP_RETURN => {
                    writeln!(result, "OP_RETURN")?;
                }
                OP_SUBSTRACT => {
                    writeln!(result, "OP_SUBSTRACT")?;
                }
                OP_TRUE => {
                    writeln!(result, "OP_TRUE")?;
                }
                OP_UNIT => {
                    writeln!(result, "OP_UNIT")?;
                }
                OP_INDEX => {
                    writeln!(result, "OP_INDEX")?;
                }
                OP_LOC_PUSH => {
                    writeln!(result, "OP_LOC_PUSH")?;
                }
                OP_LOC_POP => {
                    writeln!(result, "OP_LOC_POP")?;
                }
                OP_DOT => {
                    constant_instruction("OP_DOT", &self, offset, &mut result);
                    skip = 1;
                }
                OP_ARRAY => {
                    byte_instruction("OP_ARRAY", &self, offset, &mut result);
                    skip = 1;
                }
                OP_PARALLEL => {
                    byte_instruction("OP_PARALLEL", &self, offset, &mut result);
                    skip = 1;
                }
                OP_NEW => {
                    byte_instruction("OP_NEW", &self, offset, &mut result);
                    skip = 1;
                }
                OP_CALL => {
                    byte_instruction("OP_CALL", &self, offset, &mut result);
                    skip = 1;
                }
                OP_JUMP_IF_FALSE => {
                    jump_instruction("OP_JUMP_IF_FALSE", 1, self, offset, &mut result);
                    skip = 2;
                }
                OP_JUMP => {
                    jump_instruction("OP_JUMP", 1, self, offset, &mut result);
                    skip = 2;
                }
                OP_JUMP_BACK => {
                    jump_instruction("OP_JUMP_BACK", -1, self, offset, &mut result);
                    skip = 2;
                }
                OP_DEFINE_GLOBAL => {
                    constant_instruction("OP_DEFINE_GLOBAL", &self, offset, &mut result);
                    skip = 1;
                }
                OP_GET_GLOBAL => {
                    constant_instruction("OP_GET_GLOBAL", &self, offset, &mut result);
                    skip = 1;
                }
                OP_GET_LOCAL => {
                    byte_instruction("OP_GET_LOCAL", &self, offset, &mut result);
                    skip = 1;
                }
                OP_SET_GLOBAL => {
                    byte_instruction("OP_SET_GLOBAL", &self, offset, &mut result);
                    skip = 1;
                }
                OP_SET_LOCAL => {
                    byte_instruction("OP_SET_LOCAL", &self, offset, &mut result);
                    skip = 1;
                }
                OP_CLASS => {
                    constant_instruction("OP_CLASS", &self, offset, &mut result);
                    skip = 1;
                }
                OP_IMPORT => {
                    constant_instruction("OP_IMPORT", &self, offset, &mut result);
                    skip = 1;
                }
                0x00 | 0x24..=u8::MAX => {
                    unreachable!()
                }
            }
        }

        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub code: BytesMut,
    pub constants: Vec<Value>,
}

impl Chunk {
    ///
    ///
    ///
    pub fn new() -> Self {
        Chunk {
            code: BytesMut::new(),
            constants: Vec::new(),
        }
    }

    ///
    ///
    ///
    pub fn freeze(self) -> ReadOnlyChunk {
        ReadOnlyChunk {
            code: self.code.freeze(),
            constants: self.constants,
        }
    }

    ///
    ///
    ///
    pub fn write<B: Into<u8>>(
        &mut self,
        byte: B,
    ) {
        self.code.put_u8(byte.into());
    }

    ///
    ///
    ///
    pub fn write_pair<B1: Into<u8>, B2: Into<u8>>(
        &mut self,
        byte1: B1,
        byte2: B2,
    ) {
        self.code.put_u8(byte1.into());
        self.code.put_u8(byte2.into());
    }

    ///
    ///
    ///
    pub fn write_bytes(
        &mut self,
        bytes: &[u8],
    ) {
        self.code.extend(bytes);
    }

    ///
    ///
    ///
    pub fn add_constant(
        &mut self,
        value: Value,
    ) -> u8 {
        self.constants.push(value);

        (self.constants.len() as u8) - 1
    }
}

///
///
///
fn jump_instruction(
    name: &str,
    sign: i16,
    chunk: &ReadOnlyChunk,
    offset: usize,
    result: &mut String,
) {
    let jump1 = chunk.code[offset + 1] as u16;
    let jump2 = chunk.code[offset + 2] as u16;

    let jump = (jump1 << 8) | jump2;
    writeln!(
        result,
        "{:<16} {:4} -> {}",
        name,
        offset,
        offset as i32 + 3 + (sign * jump as i16) as i32
    )
    .unwrap();
}

///
///
///
fn constant_instruction(
    name: &str,
    chunk: &ReadOnlyChunk,
    offset: usize,
    result: &mut String,
) {
    let constant = chunk.code[offset + 1];
    write!(result, "{:<16} {:4} | ", name, constant).unwrap();

    if let Some(value) = chunk.constants.get(constant as usize) {
        writeln!(result, "{:?}", value).unwrap();
    }
}

///
///
///
fn byte_instruction(
    name: &str,
    chunk: &ReadOnlyChunk,
    offset: usize,
    result: &mut String,
) {
    let slot = chunk.code[offset + 1];
    writeln!(result, "{:<16} {:4} | ", name, slot).unwrap();
}
