use std::num::NonZeroUsize;

use crate::scanner::{Token, Tokens};
use ast::{Ident, UnOp};
use nom::error::{ContextError, ErrorKind, ParseError, VerboseError};
use nom::{branch, combinator as comb, multi, sequence as seq};
use nom::{IResult, Parser};
use semver::Version;

pub mod ast;
pub use ast::{Expr, Program, Stmt};
use seq::delimited;

use self::ast::{BinOp, Lit, Operator};

macro_rules! tag_token (
    (Token::$variant:ident) => (
        move |i: Tokens<'a>| {
            use nom::{Err, error_position, Needed, try_parse, take};
            use nom::error::ErrorKind;

            if i.tok.is_empty() {
                match stringify!($variant) {
                    "Dot" => Err(Err::Error(E::from_char(i, '.'))),
                    "Colon" => Err(Err::Error(E::from_char(i, ':'))),
                    "Comma" => Err(Err::Error(E::from_char(i, ','))),
                    "LeftBrace" => Err(Err::Error(E::from_char(i, '{'))),
                    "LeftBracket" => Err(Err::Error(E::from_char(i, '['))),
                    "LeftParen" => Err(Err::Error(E::from_char(i, '('))),
                    "RightBrace" => Err(Err::Error(E::from_char(i, '}'))),
                    "RightBracket" => Err(Err::Error(E::from_char(i, ']'))),
                    "RightParen" => Err(Err::Error(E::from_char(i, ')'))),
                    "Semicolon" => Err(Err::Error(E::from_char(i, ';'))),
                    _ => {
                        Err(Err::Error(error_position!(i, ErrorKind::Eof)))
                    }
                }
            } else {
                let (i1, t1) = try_parse!(i, take!(1));

                if t1.tok.is_empty() {
                    Err(Err::Incomplete(Needed::Size(NonZeroUsize::new(1).unwrap())))
                } else {
                    if let Token::$variant(_) = t1.tok[0] {
                        Ok((i1, t1))
                    } else {
                        match stringify!($variant) {
                            "Dot" => Err(Err::Error(E::from_char(i, '.'))),
                            "Colon" => Err(Err::Error(E::from_char(i, ':'))),
                            "Comma" => Err(Err::Error(E::from_char(i, ','))),
                            "LeftBrace" => Err(Err::Error(E::from_char(i, '{'))),
                            "LeftBracket" => Err(Err::Error(E::from_char(i, '['))),
                            "LeftParen" => Err(Err::Error(E::from_char(i, '('))),
                            "RightBrace" => Err(Err::Error(E::from_char(i, '}'))),
                            "RightBracket" => Err(Err::Error(E::from_char(i, ']'))),
                            "RightParen" => Err(Err::Error(E::from_char(i, ')'))),
                            "Semicolon" => Err(Err::Error(E::from_char(i, ';'))),
                            _ => {
                                Err(Err::Error(error_position!(i, ErrorKind::Tag)))
                            }
                        }
                    }
                }
            }
        }
    );
);

///
///
///
pub fn parse_ast(input: Tokens) -> IResult<Tokens, Vec<Stmt>, VerboseError<Tokens>> {
    comb::all_consuming(multi::many0(parse_stmt))(input)
}

///
///
///
pub fn parse_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    if input.tok.is_empty() {
        return Err(nom::Err::Error(nom::error_position!(input, ErrorKind::Tag)));
    }

    branch::alt((
        for_stmt,
        assign_stmt,
        block_stmt,
        declare_class_stmt,
        declare_func_stmt,
        expr_stmt,
        if_stmt,
        import_stmt,
        let_assign_stmt,
        return_stmt,
        while_stmt,
    ))
    .parse(input)
}

///
///
///
pub fn let_assign_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::delimited(
            tag_token!(Token::Let),
            seq::separated_pair(ident, tag_token!(Token::Assign), expr),
            tag_token!(Token::Semicolon),
        ),
        |(ident, expr)| Stmt::LetAssign(ident, expr),
    )
    .parse(input)
}

///
///
///
pub fn assign_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::terminated(
            seq::separated_pair(ident, tag_token!(Token::Assign), expr),
            tag_token!(Token::Semicolon),
        ),
        |(ident, expr)| Stmt::Assign(ident, expr),
    )
    .parse(input)
}

///
///
///
pub fn block_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::delimited(
            tag_token!(Token::LeftBrace),
            multi::many0(parse_stmt),
            tag_token!(Token::RightBrace),
        ),
        |stmts| Stmt::Block(stmts),
    )
    .parse(input)
}

///
///
///
pub fn declare_class_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::tuple((
            seq::preceded(tag_token!(Token::Class), ident),
            seq::delimited(
                tag_token!(Token::LeftBrace),
                multi::many0(parse_stmt),
                tag_token!(Token::RightBrace),
            ),
        )),
        |(ident, _)| Stmt::DeclareClass { ident },
    )
    .parse(input)
}

///
///
///
pub fn declare_func_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::tuple((
            seq::preceded(
                tag_token!(Token::Function),
                comb::cut(seq::pair(
                    ident,
                    delimited(
                        tag_token!(Token::LeftParen),
                        comb::opt(seq::pair(
                            ident,
                            multi::many0(seq::preceded(tag_token!(Token::Comma), ident)),
                        )),
                        tag_token!(Token::RightParen),
                    ),
                )),
            ),
            comb::cut(seq::delimited(
                tag_token!(Token::LeftBrace),
                multi::many0(parse_stmt),
                tag_token!(Token::RightBrace),
            )),
        )),
        |((ident, params), body)| {
            let params = params
                .map(|(h, e)| {
                    // Combine head and entries
                    [&[h], &e[..]].concat().to_vec()
                })
                .unwrap_or_default();

            Stmt::DeclareFunc { ident, params, body }
        },
    )
    .parse(input)
}

///
///
///
pub fn if_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::preceded(
            tag_token!(Token::If),
            comb::cut(seq::tuple((
                seq::delimited(tag_token!(Token::LeftParen), expr, tag_token!(Token::RightParen)),
                seq::delimited(
                    tag_token!(Token::LeftBrace),
                    multi::many0(parse_stmt),
                    tag_token!(Token::RightBrace),
                ),
                comb::opt(seq::preceded(
                    tag_token!(Token::Else),
                    seq::delimited(
                        tag_token!(Token::LeftBrace),
                        multi::many0(parse_stmt),
                        tag_token!(Token::RightBrace),
                    ),
                )),
            ))),
        ),
        |(condition, consequent, alternative)| Stmt::If {
            condition,
            consequent,
            alternative,
        },
    )
    .parse(input)
}

///
///
///
pub fn import_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    nom::error::context(
        "'import' statement",
        comb::map(
            seq::preceded(
                tag_token!(Token::Import),
                comb::cut(seq::terminated(
                    seq::pair(
                        ident,
                        comb::opt(seq::delimited(
                            tag_token!(Token::LeftBracket),
                            comb::map(tag_token!(Token::SemVer), |x| {
                                Version::parse(&x.tok[0].as_string()).unwrap()
                            }),
                            tag_token!(Token::RightBracket),
                        )),
                    ),
                    tag_token!(Token::Semicolon),
                )),
            ),
            |(package, version)| Stmt::Import { package, version },
        ),
    )
    .parse(input)
}

///
///
///
pub fn for_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    nom::error::context(
        "'for' statement",
        comb::map(
            seq::preceded(
                tag_token!(Token::For),
                comb::cut(seq::pair(
                    seq::delimited(
                        tag_token!(Token::LeftParen),
                        seq::tuple((
                            let_assign_stmt,
                            seq::terminated(expr, tag_token!(Token::Semicolon)),
                            comb::map(
                                seq::separated_pair(ident, tag_token!(Token::Assign), expr),
                                |(ident, expr)| Stmt::Assign(ident, expr),
                            ),
                        )),
                        tag_token!(Token::RightParen),
                    ),
                    seq::delimited(
                        tag_token!(Token::LeftBrace),
                        multi::many0(parse_stmt),
                        tag_token!(Token::RightBrace),
                    ),
                )),
            ),
            |((initializer, condition, increment), consequent)| Stmt::For {
                initializer: Box::new(initializer),
                condition,
                increment: Box::new(increment),
                consequent,
            },
        ),
    )
    .parse(input)
}

///
///
///
pub fn while_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::pair(
            seq::preceded(
                tag_token!(Token::While),
                seq::delimited(tag_token!(Token::LeftParen), expr, tag_token!(Token::RightParen)),
            ),
            seq::delimited(
                tag_token!(Token::LeftBrace),
                multi::many0(parse_stmt),
                tag_token!(Token::RightBrace),
            ),
        ),
        |(condition, consequent)| Stmt::While { condition, consequent },
    )
    .parse(input)
}

///
///
///
pub fn return_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::delimited(
            tag_token!(Token::Return),
            comb::opt(expr),
            comb::cut(tag_token!(Token::Semicolon)),
        ),
        |expr| Stmt::Return(expr),
    )
    .parse(input)
}

///
///
///
pub fn expr_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(seq::terminated(expr, comb::cut(tag_token!(Token::Semicolon))), |e| {
        Stmt::Expr(e)
    })
    .parse(input)
}

///
///
///
pub fn expr<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(input: Tokens<'a>) -> IResult<Tokens, Expr, E> {
    expr_pratt(input, 0)
}

///
///
///
fn expr_pratt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>,
    min_bp: u8,
) -> IResult<Tokens, Expr, E> {
    let (mut remainder, mut lhs) = match unary_operator::<E>(input) {
        Ok((r, UnOp::Idx)) => {
            let (r2, entries) = seq::terminated(
                comb::opt(seq::terminated(
                    seq::pair(expr, multi::many0(seq::preceded(tag_token!(Token::Comma), expr))),
                    comb::opt(tag_token!(Token::Comma)),
                )),
                tag_token!(Token::RightBracket),
            )
            .parse(r)?;

            let expr = if let Some((head, entries)) = entries {
                let e = [&[head], &entries[..]].concat().to_vec();

                Expr::Array(e)
            } else {
                Expr::Array(vec![])
            };

            (r2, expr)
        }
        Ok((r, UnOp::Prio)) => seq::terminated(expr, tag_token!(Token::RightParen)).parse(r)?,
        Ok((r, operator)) => {
            let (_, r_bp) = operator.binding_power();
            let (r, rhs) = expr_pratt(r, r_bp)?;

            (
                r,
                Expr::Unary {
                    operator,
                    operand: Box::new(rhs),
                },
            )
        }
        _ => expr_atom(input)?,
    };

    loop {
        match operator::<E>(remainder) {
            Ok((r, Operator::Binary(operator))) => {
                let (left_bp, right_bp) = operator.binding_power();
                if left_bp < min_bp {
                    break;
                }

                // Recursive until lower binding power is encountered.
                let (remainder_3, rhs) = expr_pratt(r, right_bp)?;

                remainder = remainder_3;
                lhs = Expr::Binary {
                    operator,
                    lhs_operand: Box::new(lhs),
                    rhs_operand: Box::new(rhs),
                };
            }
            Ok((r, Operator::Unary(operator))) => {
                let (left_bp, _) = operator.binding_power();
                if left_bp < min_bp {
                    break;
                }

                lhs = if let UnOp::Idx = operator {
                    let (r2, rhs) = seq::terminated(expr, tag_token!(Token::RightBracket)).parse(r)?;
                    remainder = r2;

                    Expr::Index {
                        array: Box::new(lhs),
                        index: Box::new(rhs),
                    }
                } else {
                    Expr::Unary {
                        operator,
                        operand: Box::new(lhs),
                    }
                };
            }
            _ => break,
        }
    }

    Ok((remainder, lhs))
}

///
///
///
pub fn expr_atom<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Expr, E> {
    branch::alt((call_expr, literal_expr, unit_expr, ident_expr)).parse(input)
}

/// Integrate this in pratt parser? To support, e.g., f()()() ?
///
///
pub fn call_expr<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Expr, E> {
    comb::map(
        seq::pair(
            ident,
            seq::delimited(
                tag_token!(Token::LeftParen),
                comb::opt(seq::pair(
                    expr,
                    multi::many0(seq::preceded(tag_token!(Token::Comma), expr)),
                )),
                tag_token!(Token::RightParen),
            ),
        ),
        |(function, arguments)| {
            let arguments = arguments
                .map(|(h, e)| [&[h], &e[..]].concat().to_vec())
                .unwrap_or_default();

            Expr::Call { function, arguments }
        },
    )
    .parse(input)
}

///
///
///
pub fn literal_expr<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Expr, E> {
    comb::map(literal, |l| Expr::Literal(l)).parse(input)
}

///
///
///
pub fn literal<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(input: Tokens<'a>) -> IResult<Tokens, Lit, E> {
    branch::alt((
        comb::map(tag_token!(Token::Boolean), |t| Lit::Boolean(t.tok[0].as_bool())),
        comb::map(tag_token!(Token::Integer), |t| Lit::Integer(t.tok[0].as_i64())),
        comb::map(tag_token!(Token::Real), |t| Lit::Real(t.tok[0].as_f64())),
        comb::map(tag_token!(Token::String), |t| Lit::String(t.tok[0].as_string())),
    ))
    .parse(input)
}

///
///
///
pub fn unit_expr<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Expr, E> {
    comb::map(tag_token!(Token::Unit), |_| Expr::Unit).parse(input)
}

///
///
///
pub fn ident_expr<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Expr, E> {
    comb::map(ident, |x| Expr::Ident(x)).parse(input)
}

///
///
///
pub fn ident<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(input: Tokens<'a>) -> IResult<Tokens, Ident, E> {
    comb::map(tag_token!(Token::Ident), |x| Ident(x.tok[0].as_string())).parse(input)
}

///
///
///
pub fn operator<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Operator, E> {
    branch::alt((
        comb::map(binary_operator, |x| Operator::Binary(x)),
        comb::map(unary_operator, |x| Operator::Unary(x)),
    ))
    .parse(input)
}

///
///
///
fn binary_operator<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, BinOp, E> {
    branch::alt((
        comb::map(tag_token!(Token::And), |_| BinOp::And),
        comb::map(tag_token!(Token::Equal), |_| BinOp::Eq),
        comb::map(tag_token!(Token::Greater), |_| BinOp::Gt),
        comb::map(tag_token!(Token::GreaterOrEqual), |_| BinOp::Ge),
        comb::map(tag_token!(Token::Less), |_| BinOp::Lt),
        comb::map(tag_token!(Token::LessOrEqual), |_| BinOp::Le),
        comb::map(tag_token!(Token::Minus), |_| BinOp::Sub),
        comb::map(tag_token!(Token::NotEqual), |_| BinOp::Ne),
        comb::map(tag_token!(Token::Or), |_| BinOp::Or),
        comb::map(tag_token!(Token::Plus), |_| BinOp::Add),
        comb::map(tag_token!(Token::Slash), |_| BinOp::Div),
        comb::map(tag_token!(Token::Star), |_| BinOp::Mul),
    ))
    .parse(input)
}

///
///
///
fn unary_operator<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, UnOp, E> {
    branch::alt((
        comb::map(tag_token!(Token::Not), |_| UnOp::Not),
        comb::map(tag_token!(Token::Minus), |_| UnOp::Neg),
        comb::map(tag_token!(Token::LeftBracket), |_| UnOp::Idx),
        comb::map(tag_token!(Token::LeftParen), |_| UnOp::Prio),
    ))
    .parse(input)
}
