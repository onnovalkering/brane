use crate::parser::ast::*;
use anyhow::Result;
use brane_bvm::{bytecode::{Chunk, Function, OpCode}, values::{Class, Value}};

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: i32,
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

    Ok(Function::new(String::from("main"), 0, chunk))
}

///
///
///
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

    let function = Function::new(name, params.len() as u8, chunk);

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
        Stmt::Import {
            package: Ident(ident), ..
        } => {
            let import = chunk.add_constant(ident.clone().into());
            chunk.write_pair(OpCode::OpImport, import);
        }
        Stmt::DeclareClass { ident: Ident(ident), properties } => {
            let class = Value::Class(Class::new(ident.clone().into(), properties));
            let class = chunk.add_constant(class);
            chunk.write_pair(OpCode::OpClass, class);

            let ident = chunk.add_constant(ident.into());
            chunk.write_pair(OpCode::OpDefineGlobal, ident);
        }
        Stmt::Assign(Ident(ident), expr) => {
            // ident must be an existing local or global.
            expr_to_opcodes(expr, chunk, locals, scope);

            if let Some(index) = locals.iter().position(|l| l.name == ident) {
                chunk.write_pair(OpCode::OpSetLocal, index as u8);
            } else {
                let ident = chunk.add_constant(ident.into());
                chunk.write_pair(OpCode::OpSetGlobal, ident);
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
            chunk.write_pair(OpCode::OpDefineGlobal, ident);
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
            chunk.write_pair(0x00, 0x00);

            chunk.write(OpCode::OpPop);
            for stmt in consequent {
                stmt_to_opcodes(stmt, chunk, locals, scope);
            }

            // Run incrementer statement
            stmt_to_opcodes(*increment, chunk, locals, scope);

            // Emit loop
            chunk.write(OpCode::OpJumpBack);
            let jump_back = (chunk.code.len() - loop_start + 2) as u16;
            chunk.write_bytes(&jump_back.to_be_bytes()[..]);

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
            chunk.write_pair(0x00, 0x00);

            chunk.write(OpCode::OpPop);
            stmt_to_opcodes(Stmt::Block(consequent), chunk, locals, scope);

            // Emit loop
            chunk.write(OpCode::OpJumpBack);
            let jump_back = (chunk.code.len() - loop_start + 2) as u16;
            chunk.write_bytes(&jump_back.to_be_bytes()[..]);

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
            chunk.write_pair(0x00, 0x00);

            chunk.write(OpCode::OpPop);
            stmt_to_opcodes(Stmt::Block(consequent), chunk, locals, scope);

            // For the else branch
            chunk.write(OpCode::OpJump);
            // Placeholders, we'll backpatch this later.
            let else_jump_pos = chunk.code.len();
            chunk.write_pair(0x00, 0x00);

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
            chunk.write_pair(OpCode::OpConstant, function);

            let ident = chunk.add_constant(ident.into());
            chunk.write_pair(OpCode::OpDefineGlobal, ident);
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
            // Always evaluate LHS
            expr_to_opcodes(*lhs_operand, chunk, locals, scope);
            let rhs_operand = *rhs_operand;

            if let BinOp::Dot = operator {
                if let Expr::Ident(Ident(ident)) = &rhs_operand   {
                    let constant = chunk.add_constant(ident.clone().into());
                    chunk.write_pair(OpCode::OpDot, constant);
                    return;
                } else {
                    unreachable!()
                }
            }

            expr_to_opcodes(rhs_operand, chunk, locals, scope);
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

                _ => unreachable!()
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
                    chunk.write_pair(OpCode::OpConstant, constant);
                }
                Lit::Real(real) => {
                    let constant = chunk.add_constant(real.into());
                    chunk.write_pair(OpCode::OpConstant, constant);
                }
                Lit::String(string) => {
                    let constant = chunk.add_constant(string.into());
                    chunk.write_pair(OpCode::OpConstant, constant);
                }
            };
        }
        Expr::Unit => chunk.write(OpCode::OpUnit),
        Expr::Ident(Ident(ident)) => {
            if let Some(index) = locals.iter().position(|l| l.name == ident) {
                chunk.write_pair(OpCode::OpGetLocal, index as u8);
            } else {
                let ident = chunk.add_constant(ident.into());
                chunk.write_pair(OpCode::OpGetGlobal, ident);
            }
        }
        Expr::Call { function, arguments } => {
            expr_to_opcodes(Expr::Ident(function), chunk, locals, scope);

            let arguments_n = arguments.len() as u8;
            for argument in arguments {
                expr_to_opcodes(argument, chunk, locals, scope);
            }

            chunk.write_pair(OpCode::OpCall, arguments_n);
        }
        Expr::Instance { class, properties } => {
            let properties_n = properties.len() as u8;
            for property in properties {
                if let Stmt::Assign(Ident(name), value) = property {
                    expr_to_opcodes(value, chunk, locals, scope);
                    expr_to_opcodes(Expr::Literal(Lit::String(name)), chunk, locals, scope);
                } else {
                    unreachable!();
                }
            }

            expr_to_opcodes(Expr::Ident(class), chunk, locals, scope);
            chunk.write_pair(OpCode::OpNew, properties_n);
        }
        _ => todo!()
    }
}
