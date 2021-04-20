use std::fmt::{self, Formatter};

use crate::parser::ast::*;
use anyhow::Result;
use bytes::{BufMut, BytesMut};

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

#[derive(Clone)]
pub enum Function {
    External { name: String },
    Native { name: String, arity: i32 },
    UserDefined { name: String, arity: i32, chunk: Chunk },
}

impl fmt::Debug for Function {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Function::External { name, .. } | Function::Native { name, .. } | Function::UserDefined { name, .. } => {
                write!(f, "{}", name)
            }
        }
    }
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

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Unit,
    Function(Function),
    Class(Class),
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

impl From<Function> for Value {
    fn from(function: Function) -> Self {
        Value::Function(function)
    }
}

impl From<Ident> for Value {
    fn from(ident: Ident) -> Self {
        Value::String(ident.0)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::String(string)
    }
}

impl From<bool> for Value {
    fn from(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }
}

impl From<i64> for Value {
    fn from(integer: i64) -> Self {
        Value::Integer(integer)
    }
}

impl From<f64> for Value {
    fn from(real: f64) -> Self {
        Value::Real(real)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Unit
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

///
///
///
pub fn compile(program: Program) -> Result<Function> {
    let mut chunk = Chunk::new();
    let mut locals = Vec::new();

    for stmt in program {
        stmt_to_opcodes(stmt, &mut chunk, &mut locals, 0);
    }

    disassemble_chunk(&chunk, "main");
    Ok(Function::new(String::from("main"), 0, chunk))
}

pub fn compile_function(
    block: Block,
    scope: i32,
    params: &Vec<Ident>,
    name: String,
) -> Result<Function> {
    let mut locals = Vec::new();
    let mut chunk = Chunk::new();

    let local = Local {
        name: String::from("func"),
        depth: scope,
    };
    locals.push(local);

    for Ident(param) in params {
        let local = Local {
            name: param.clone(),
            depth: scope,
        };
        locals.push(local);
    }

    for stmt in block {
        stmt_to_opcodes(stmt, &mut chunk, &mut locals, scope);
    }

    disassemble_chunk(&chunk, &name);
    let function = Function::new(name, params.len() as i32, chunk);

    Ok(function)
}

///
///
///
pub fn stmt_to_opcodes(
    stmt: Stmt,
    chunk: &mut Chunk,
    locals: &mut Vec<Local>,
    scope: i32,
) {
    match stmt {
        Stmt::Import(Ident(ident)) => {
            let import = chunk.add_constant(ident.clone().into());
            chunk.write(OpCode::OpImport);
            chunk.write(import);
        }
        Stmt::DeclareClass { ident: Ident(ident) } => {
            let class = chunk.add_constant(ident.clone().into());
            chunk.write(OpCode::OpClass);
            chunk.write(class);

            let ident = chunk.add_constant(ident.into());
            chunk.write(OpCode::OpDefineGlobal);
            chunk.write(ident);
        }
        Stmt::Assign(Ident(ident), expr) => {
            // ident must be an existing local or global.
            expr_to_opcodes(expr, chunk, locals, scope);

            if let Some(index) = locals.iter().position(|l| l.name == ident) {
                chunk.write(OpCode::OpSetLocal);
                chunk.write(index as u8);
            } else {
                let ident = chunk.add_constant(ident.into());
                chunk.write(OpCode::OpSetGlobal);
                chunk.write(ident);
            }
        }
        Stmt::LetAssign(Ident(ident), expr) => {
            expr_to_opcodes(expr, chunk, locals, scope);

            // Don't put a local's name in the globals table.
            // Instead, just note that there's a local on the stack.
            if scope > 0 {
                let local = Local {
                    name: ident,
                    depth: scope,
                };
                locals.push(local);
                return;
            }

            let ident = chunk.add_constant(ident.into());
            chunk.write(OpCode::OpDefineGlobal);
            chunk.write(ident);
        }
        Stmt::Block(block) => {
            // Create a new scope (shadow).
            let scope = scope + 1;

            for stmt in block {
                stmt_to_opcodes(stmt, chunk, locals, scope);
            }

            // Remove any locals created in this scope.
            while let Some(local) = locals.pop() {
                if local.depth >= scope {
                    chunk.write(OpCode::OpPop);
                } else {
                    // Oops, one to many, place it back.
                    locals.push(local);
                    break;
                }
            }
        }
        Stmt::For {
            initializer,
            condition,
            increment,
            consequent,
        } => {
            let scope = scope + 1;

            stmt_to_opcodes(*initializer, chunk, locals, scope);

            let loop_start = chunk.code.len();

            expr_to_opcodes(condition, chunk, locals, scope);
            // Now the result of the condition is on the stack.

            chunk.write(OpCode::OpJumpIfFalse);
            // Placeholders, we'll backpatch this later.
            let plh_pos = chunk.code.len();
            chunk.write(0x00);
            chunk.write(0x00);

            chunk.write(OpCode::OpPop);
            for stmt in consequent {
                stmt_to_opcodes(stmt, chunk, locals, scope);
            }

            // Run incrementer statement
            stmt_to_opcodes(*increment, chunk, locals, scope);

            // Emit loop
            chunk.write(OpCode::OpJumpBack);
            let jump_back = (chunk.code.len() - loop_start + 2) as u16;
            let [first, second, ..] = jump_back.to_be_bytes();
            chunk.write(first);
            chunk.write(second);

            // How much to jump if condition is false (exit).
            let jump = (chunk.code.len() - plh_pos - 2) as u16;
            let [first, second, ..] = jump.to_be_bytes();
            chunk.code[plh_pos] = first;
            chunk.code[plh_pos + 1] = second;

            chunk.write(OpCode::OpPop);
        }
        Stmt::While { condition, consequent } => {
            let loop_start = chunk.code.len();

            expr_to_opcodes(condition, chunk, locals, scope);
            // Now the result of the condition is on the stack.

            chunk.write(OpCode::OpJumpIfFalse);
            // Placeholders, we'll backpatch this later.
            let plh_pos = chunk.code.len();
            chunk.write(0x00);
            chunk.write(0x00);

            chunk.write(OpCode::OpPop);
            stmt_to_opcodes(Stmt::Block(consequent), chunk, locals, scope);

            // Emit loop
            chunk.write(OpCode::OpJumpBack);
            let jump_back = (chunk.code.len() - loop_start + 2) as u16;
            let [first, second, ..] = jump_back.to_be_bytes();
            chunk.write(first);
            chunk.write(second);

            // How much to jump?
            let jump = (chunk.code.len() - plh_pos - 2) as u16;
            let [first, second, ..] = jump.to_be_bytes();
            chunk.code[plh_pos] = first;
            chunk.code[plh_pos + 1] = second;

            chunk.write(OpCode::OpPop);
        }
        Stmt::If {
            condition,
            consequent,
            alternative,
        } => {
            expr_to_opcodes(condition, chunk, locals, scope);
            // Now the result of the condition is on the stack.

            chunk.write(OpCode::OpJumpIfFalse);
            // Placeholders, we'll backpatch this later.
            let plh_pos = chunk.code.len();
            chunk.write(0x00);
            chunk.write(0x00);

            chunk.write(OpCode::OpPop);
            stmt_to_opcodes(Stmt::Block(consequent), chunk, locals, scope);

            // For the else branch
            chunk.write(OpCode::OpJump);
            // Placeholders, we'll backpatch this later.
            let else_jump_pos = chunk.code.len();
            chunk.write(0x00);
            chunk.write(0x00);

            // How much to jump?
            let jump = (chunk.code.len() - plh_pos - 2) as u16;
            let [first, second, ..] = jump.to_be_bytes();
            chunk.code[plh_pos] = first;
            chunk.code[plh_pos + 1] = second;

            chunk.write(OpCode::OpPop);

            if let Some(alternative) = alternative {
                stmt_to_opcodes(Stmt::Block(alternative), chunk, locals, scope);
            }

            let jump = (chunk.code.len() - else_jump_pos - 2) as u16;
            let [first, second, ..] = jump.to_be_bytes();
            chunk.code[else_jump_pos] = first;
            chunk.code[else_jump_pos + 1] = second;
        }
        Stmt::Expr(expr) => {
            expr_to_opcodes(expr, chunk, locals, scope);
            // chunk.write(OpCode::OpPop);
        }
        Stmt::Return(Some(expr)) => {
            expr_to_opcodes(expr, chunk, locals, scope);
            chunk.write(OpCode::OpReturn);
        }
        Stmt::Return(None) => {
            chunk.write(OpCode::OpReturn);
        }
        Stmt::DeclareFunc {
            ident: Ident(ident),
            params,
            body,
        } => {
            let function = compile_function(body, scope + 1, &params, ident.clone()).unwrap();

            let function = chunk.add_constant(function.into());
            chunk.write(OpCode::OpConstant);
            chunk.write(function);

            let ident = chunk.add_constant(ident.into());
            chunk.write(OpCode::OpDefineGlobal);
            chunk.write(ident);
        }
    }
}

///
///
///
pub fn expr_to_opcodes(
    expr: Expr,
    chunk: &mut Chunk,
    locals: &mut Vec<Local>,
    scope: i32,
) {
    match expr {
        Expr::Binary {
            operator,
            lhs_operand,
            rhs_operand,
        } => {
            expr_to_opcodes(*lhs_operand, chunk, locals, scope);
            expr_to_opcodes(*rhs_operand, chunk, locals, scope);
            match operator {
                // Arithmetic
                BinOp::Add => chunk.write(OpCode::OpAdd),
                BinOp::Sub => chunk.write(OpCode::OpSubstract),
                BinOp::Mul => chunk.write(OpCode::OpMultiply),
                BinOp::Div => chunk.write(OpCode::OpDivide),

                // Equality / Comparison
                BinOp::Eq => chunk.write(OpCode::OpEqual),
                BinOp::Lt => chunk.write(OpCode::OpLess),
                BinOp::Gt => chunk.write(OpCode::OpGreater),
                BinOp::Le => {
                    // !(lhs > rhs)
                    chunk.write(OpCode::OpGreater);
                    chunk.write(OpCode::OpNot);
                }
                BinOp::Ge => {
                    // !(lhs < rhs)
                    chunk.write(OpCode::OpLess);
                    chunk.write(OpCode::OpNot);
                }
                BinOp::Ne => {
                    // !(lhs == rhs)
                    chunk.write(OpCode::OpEqual);
                    chunk.write(OpCode::OpNot);
                }

                // Logical
                BinOp::And => chunk.write(OpCode::OpAnd),
                BinOp::Or => chunk.write(OpCode::OpOr),
            }
        }
        Expr::Unary { operator, operand } => {
            expr_to_opcodes(*operand, chunk, locals, scope);
            match operator {
                UnOp::Neg => chunk.write(OpCode::OpNegate),
                UnOp::Not => chunk.write(OpCode::OpNot),
                _ => unreachable!(),
            }
        }
        Expr::Literal(literal) => {
            match literal {
                Lit::Boolean(boolean) => match boolean {
                    true => chunk.write(OpCode::OpTrue),
                    false => chunk.write(OpCode::OpFalse),
                },
                Lit::Integer(integer) => {
                    let constant = chunk.add_constant(integer.into());
                    chunk.write(OpCode::OpConstant);
                    chunk.write(constant);
                }
                Lit::Real(real) => {
                    let constant = chunk.add_constant(real.into());
                    chunk.write(OpCode::OpConstant);
                    chunk.write(constant);
                }
                Lit::String(string) => {
                    let constant = chunk.add_constant(string.into());
                    chunk.write(OpCode::OpConstant);
                    chunk.write(constant);
                }
            };
        }
        Expr::Unit => chunk.write(OpCode::OpUnit),
        Expr::Ident(Ident(ident)) => {
            if let Some(index) = locals.iter().position(|l| l.name == ident) {
                chunk.write(OpCode::OpGetLocal);
                chunk.write(index as u8);
            } else {
                let ident = chunk.add_constant(ident.into());
                chunk.write(OpCode::OpGetGlobal);
                chunk.write(ident);
            }
        }
        Expr::Call { function, arguments } => {
            expr_to_opcodes(Expr::Ident(function), chunk, locals, scope);

            let arguments_n = arguments.len() as u8;
            for argument in arguments {
                expr_to_opcodes(argument, chunk, locals, scope);
            }

            chunk.write(OpCode::OpCall);
            chunk.write(arguments_n);
        }
        _ => todo!(),
    }
}

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
    print!("{:<16} {:4} '", name, constant);

    if let Some(value) = chunk.constants.get(constant as usize) {
        println!("{:?}'", value);
    }
}

pub fn byte_instruction(
    name: &str,
    chunk: &Chunk,
    offset: usize,
) {
    let slot = chunk.code[offset + 1];
    println!("{:<16} {:4} '", name, slot);
}
