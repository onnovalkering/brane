use brane_bvm::bytecode::{Chunk, OpCode};

pub fn disassemble_chunk(
    chunk: &Chunk,
    name: &str,
) {
    println!("== {} ==", name);

    let mut skip = 0;
    for (offset, instruction) in chunk.code.iter().enumerate() {
        if skip > 0 {
            skip = skip - 1;
            continue;
        }

        print!("{:04} ", offset);

        match OpCode::from(*instruction) {
            OpCode::OpConstant => {
                constant_instruction("OP_CONSTANT", &chunk, offset);
                skip = 1;
            }
            OpCode::OpAdd => println!("OP_ADD"),
            OpCode::OpDivide => println!("OP_DIVIDE"),
            OpCode::OpMultiply => println!("OP_MULTIPLY"),
            OpCode::OpSubstract => println!("OP_SUBSTRACT"),
            OpCode::OpNegate => println!("OP_NEGATE"),
            OpCode::OpReturn => println!("OP_RETURN"),
            OpCode::OpFalse => println!("OP_FALSE"),
            OpCode::OpTrue => println!("OP_TRUE"),
            OpCode::OpUnit => println!("OP_UNIT"),
            OpCode::OpNot => println!("OP_NOT"),
            OpCode::OpEqual => println!("OP_EQUAL"),
            OpCode::OpGreater => println!("OP_GREATER"),
            OpCode::OpLess => println!("OP_LESS"),
            OpCode::OpPop => println!("OP_POP"),
            OpCode::OpOr => println!("OP_OR"),
            OpCode::OpAnd => println!("OP_AND"),
            OpCode::OpCall => {
                byte_instruction("OP_CALL", &chunk, offset);
                skip = 1;
            }
            OpCode::OpJumpIfFalse => {
                jump_instruction("OP_JUMP_IF_FALSE", 1, chunk, offset);
                skip = 2;
            }
            OpCode::OpJump => {
                jump_instruction("OP_JUMP", 1, chunk, offset);
                skip = 2;
            }
            OpCode::OpJumpBack => {
                jump_instruction("OP_JUMP_BACK", -1, chunk, offset);
                skip = 2;
            }
            OpCode::OpDefineGlobal => {
                constant_instruction("OP_DEFINE_GLOBAL", &chunk, offset);
                skip = 1;
            }
            OpCode::OpGetGlobal => {
                constant_instruction("OP_GET_GLOBAL", &chunk, offset);
                skip = 1;
            }
            OpCode::OpGetLocal => {
                byte_instruction("OP_GET_LOCAL", &chunk, offset);
                skip = 1;
            }
            OpCode::OpSetGlobal => {
                byte_instruction("OP_SET_GLOBAL", &chunk, offset);
                skip = 1;
            }
            OpCode::OpSetLocal => {
                byte_instruction("OP_SET_LOCAL", &chunk, offset);
                skip = 1;
            }
            OpCode::OpClass => {
                constant_instruction("OP_CLASS", &chunk, offset);
                skip = 1;
            }
            OpCode::OpImport => {
                constant_instruction("OP_IMPORT", &chunk, offset);
                skip = 1;
            }
        }
    }
}

pub fn jump_instruction(
    name: &str,
    sign: i16,
    chunk: &Chunk,
    offset: usize,
) {
    let jump1 = chunk.code[offset + 1] as u16;
    let jump2 = chunk.code[offset + 2] as u16;

    let jump = (jump1 << 8) | jump2;
    println!(
        "{:<16} {:4} -> {}",
        name,
        offset,
        offset as i32 + 3 + (sign * jump as i16) as i32
    );
}

pub fn constant_instruction(
    name: &str,
    chunk: &Chunk,
    offset: usize,
) {
    let constant = chunk.code[offset + 1];
    print!("{:<16} {:4} | ", name, constant);

    if let Some(value) = chunk.constants.get(constant as usize) {
        println!("{:?}", value);
    }
}

pub fn byte_instruction(
    name: &str,
    chunk: &Chunk,
    offset: usize,
) {
    let slot = chunk.code[offset + 1];
    println!("{:<16} {:4} | ", name, slot);
}
