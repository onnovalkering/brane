use super::ast::{Expr, Ident, Operator, Stmt, UnOp};
use crate::parser::{identifier, literal, operator, pattern};
use crate::scanner::{Token, Tokens};
use crate::tag_token;
use nom::{branch, combinator as comb, multi, sequence as seq};
use nom::{
    error::{ContextError, ErrorKind, ParseError, VerboseError},
    Needed,
};
use nom::{IResult, Parser};
use specifications::package::PackageIndex;
use std::num::NonZeroUsize;

///
///
///
pub fn parse_ast(
    input: Tokens,
    package_index: PackageIndex,
) -> IResult<Tokens, Vec<Stmt>, VerboseError<Tokens>> {
    comb::all_consuming(multi::many0(parse_stmt))
        .parse(input)
        .map(|(tokens, program)| {
            let program = pattern::resolve_patterns(program, &package_index)
                .map_err(|_| nom::Err::Incomplete(Needed::Size(NonZeroUsize::new(1).unwrap())))?;

            Ok((tokens, program))
        })?
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

    branch::alt((import_stmt, assign_stmt, return_stmt, expr_stmt)).parse(input)
}

///
///
///
pub fn assign_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::terminated(
            seq::separated_pair(identifier::parse, tag_token!(Token::Assign), expr),
            comb::cut(tag_token!(Token::Semicolon)),
        ),
        |(ident, expr)| Stmt::Assign(ident, expr),
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
                        identifier::parse,
                        comb::opt(multi::many0(seq::preceded(tag_token!(Token::Comma), identifier::parse))),
                    ),
                    tag_token!(Token::Semicolon),
                )),
            ),
            |(package, packages)| {
                let mut packages: Vec<Ident> = packages.unwrap_or_default();
                packages.insert(0, package);
                packages.dedup_by(|Ident(a), Ident(b)| a.eq_ignore_ascii_case(b));

                let imports = packages
                    .into_iter()
                    .map(|package| Stmt::Import { package, version: None })
                    .collect();

                Stmt::Block(imports)
            },
        ),
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
        Stmt::Return,
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
    let (mut remainder, mut lhs) = match operator::unary_operator::<E>(input) {
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
        // Append any subsequent atoms to LHS.
        // The LHS will be turned into a pattern expression.
        if let Ok((r, ident)) = expr_atom::<E>(remainder) {
            let terms = match lhs {
                Expr::Pattern(terms) => {
                    let mut terms = terms;
                    terms.push(ident);

                    terms
                }
                current => {
                    vec![current, ident]
                }
            };

            lhs = Expr::Pattern(terms);

            remainder = r;
            continue;
        }

        //
        //
        //
        match operator::parse::<E>(remainder) {
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
    branch::alt((
        comb::map(literal::parse, Expr::Literal),
        comb::map(identifier::parse, Expr::Ident),
    ))
    .parse(input)
}
