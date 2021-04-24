use bytes::{BufMut, BytesMut};

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
    OpJumpBack, // Allow negative operand for OpJump instead?
    OpSetLocal,
    OpSetGlobal,
    OpCall,
    OpClass,
    OpImport,
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
            _ => {
                panic!("ERROR: not a OpCode: {}", byte);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Function {
    External { package: String, name: String, arity: i32 },
    Native { name: String, arity: i32 },
    UserDefined { name: String, arity: i32, chunk: Chunk },
}

impl Function {
    pub fn new(
        name: String,
        arity: i32,
        chunk: Chunk,
    ) -> Self {
        Function::UserDefined { arity, name, chunk }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub code: BytesMut,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: BytesMut::new(),
            constants: Vec::new(),
        }
    }

    pub fn write<B: Into<u8>>(
        &mut self,
        byte: B,
    ) {
        self.code.put_u8(byte.into());
    }

    pub fn add_constant(
        &mut self,
        value: Value,
    ) -> u8 {
        self.constants.push(value);

        (self.constants.len() as u8) - 1
    }
}
