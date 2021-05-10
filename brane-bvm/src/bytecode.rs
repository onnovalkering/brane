use anyhow::Result;
use bytes::{BufMut, BytesMut};
use specifications::common::Parameter;
use std::fmt::{self, Write};

use crate::values::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpConstant = 1,
    OpAdd,
    OpSubstract,
    OpMultiply,
    OpDivide,
    OpNegate,
    OpReturn,
    OpTrue,
    OpFalse,
    OpUnit,
    OpNot,
    OpEqual,
    OpGreater,
    OpLess,
    OpPop,
    OpDefineGlobal,
    OpGetGlobal,
    OpGetLocal,
    OpJumpIfFalse,
    OpJump,
    OpAnd,
    OpOr,
    OpJumpBack,
    OpSetLocal,
    OpSetGlobal,
    OpCall,
    OpClass,
    OpImport,
    OpNew,
    OpDot,
    OpArray,
    OpIndex,
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0x01 => OpCode::OpConstant,
            0x02 => OpCode::OpAdd,
            0x03 => OpCode::OpSubstract,
            0x04 => OpCode::OpMultiply,
            0x05 => OpCode::OpDivide,
            0x06 => OpCode::OpNegate,
            0x07 => OpCode::OpReturn,
            0x08 => OpCode::OpTrue,
            0x09 => OpCode::OpFalse,
            0x0A => OpCode::OpUnit,
            0x0B => OpCode::OpNot,
            0x0C => OpCode::OpEqual,
            0x0D => OpCode::OpGreater,
            0x0E => OpCode::OpLess,
            0x0F => OpCode::OpPop,
            0x10 => OpCode::OpDefineGlobal,
            0x11 => OpCode::OpGetGlobal,
            0x12 => OpCode::OpGetLocal,
            0x13 => OpCode::OpJumpIfFalse,
            0x14 => OpCode::OpJump,
            0x15 => OpCode::OpAnd,
            0x16 => OpCode::OpOr,
            0x17 => OpCode::OpJumpBack,
            0x18 => OpCode::OpSetLocal,
            0x19 => OpCode::OpSetGlobal,
            0x1A => OpCode::OpCall,
            0x1B => OpCode::OpClass,
            0x1C => OpCode::OpImport,
            0x1D => OpCode::OpNew,
            0x1E => OpCode::OpDot,
            0x1F => OpCode::OpArray,
            0x20 => OpCode::OpIndex,
            _ => {
                panic!("ERROR: not a OpCode: {}", byte);
            }
        }
    }
}

#[derive(Clone)]
pub enum Function {
    External { package: String, version: String, kind: String, name: String, parameters: Vec<Parameter>, },
    Native { name: String, arity: u8 },
    UserDefined { name: String, arity: u8, chunk: Chunk },
}

impl Function {
    pub fn new(
        name: String,
        arity: u8,
        chunk: Chunk,
    ) -> Self {
        Function::UserDefined { arity, name, chunk }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::UserDefined { name, ..} | Function::External { name, .. } | Function::Native { name, .. } => write!(f, "{}(..)", name)
        }
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
            match OpCode::from(*instruction) {
                OpCode::OpConstant => {
                    constant_instruction("OP_CONSTANT", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpAdd => { writeln!(result, "OP_ADD")?; }
                OpCode::OpAnd => { writeln!(result, "OP_AND")?; }
                OpCode::OpDivide => { writeln!(result, "OP_DIVIDE")?; }
                OpCode::OpEqual => { writeln!(result, "OP_EQUAL")?; }
                OpCode::OpFalse => { writeln!(result, "OP_FALSE")?; }
                OpCode::OpGreater => { writeln!(result, "OP_GREATER")?; }
                OpCode::OpLess => { writeln!(result, "OP_LESS")?; }
                OpCode::OpMultiply => { writeln!(result, "OP_MULTIPLY")?; }
                OpCode::OpNegate => { writeln!(result, "OP_NEGATE")?; }
                OpCode::OpNot => { writeln!(result, "OP_NOT")?; }
                OpCode::OpOr => { writeln!(result, "OP_OR")?; }
                OpCode::OpPop => { writeln!(result, "OP_POP")?; }
                OpCode::OpReturn => { writeln!(result, "OP_RETURN")?; }
                OpCode::OpSubstract => { writeln!(result, "OP_SUBSTRACT")?; }
                OpCode::OpTrue => { writeln!(result, "OP_TRUE")?; }
                OpCode::OpUnit => { writeln!(result, "OP_UNIT")?; }
                OpCode::OpIndex => { writeln!(result, "OP_INDEX")?; }
                OpCode::OpDot => {
                    constant_instruction("OP_DOT", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpArray => {
                    byte_instruction("OP_ARRAY", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpNew => {
                    byte_instruction("OP_NEW", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpCall => {
                    byte_instruction("OP_CALL", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpJumpIfFalse => {
                    jump_instruction("OP_JUMP_IF_FALSE", 1, self, offset, &mut result);
                    skip = 2;
                }
                OpCode::OpJump => {
                    jump_instruction("OP_JUMP", 1, self, offset, &mut result);
                    skip = 2;
                }
                OpCode::OpJumpBack => {
                    jump_instruction("OP_JUMP_BACK", -1, self, offset, &mut result);
                    skip = 2;
                }
                OpCode::OpDefineGlobal => {
                    constant_instruction("OP_DEFINE_GLOBAL", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpGetGlobal => {
                    constant_instruction("OP_GET_GLOBAL", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpGetLocal => {
                    byte_instruction("OP_GET_LOCAL", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpSetGlobal => {
                    byte_instruction("OP_SET_GLOBAL", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpSetLocal => {
                    byte_instruction("OP_SET_LOCAL", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpClass => {
                    constant_instruction("OP_CLASS", &self, offset, &mut result);
                    skip = 1;
                }
                OpCode::OpImport => {
                    constant_instruction("OP_IMPORT", &self, offset, &mut result);
                    skip = 1;
                }
            }
        }

        Ok(result)
    }
}

///
///
///
fn jump_instruction(
    name: &str,
    sign: i16,
    chunk: &Chunk,
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
    ).unwrap();
}

///
///
///
fn constant_instruction(
    name: &str,
    chunk: &Chunk,
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
    chunk: &Chunk,
    offset: usize,
    result: &mut String,
) {
    let slot = chunk.code[offset + 1];
    writeln!(result, "{:<16} {:4} | ", name, slot).unwrap();
}
