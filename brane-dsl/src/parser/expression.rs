use super::ast::{Expr, Operator, UnOp};
use crate::parser::{identifier, instance, literal, operator};
use crate::scanner::{Token, Tokens};
use crate::tag_token;
use nom::error::{ContextError, ParseError};
use nom::{branch, combinator as comb, multi, sequence as seq};
use nom::{IResult, Parser};
use std::num::NonZeroUsize;

///
///
///
pub fn parse<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(input: Tokens<'a>) -> IResult<Tokens, Expr, E> {
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
                    seq::pair(
                        self::parse,
                        multi::many0(seq::preceded(tag_token!(Token::Comma), self::parse)),
                    ),
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
        Ok((r, UnOp::Prio)) => seq::terminated(self::parse, tag_token!(Token::RightParen)).parse(r)?,
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
                    let (r2, rhs) = seq::terminated(self::parse, tag_token!(Token::RightBracket)).parse(r)?;
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
        instance::parse,
        call_expr,
        comb::map(literal::parse, |l| Expr::Literal(l)),
        comb::map(identifier::parse, |i| Expr::Ident(i)),
    ))
    .parse(input)
}

/// Integrate this in pratt parser? To support, e.g., f()()() ?
///
///
pub fn call_expr<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Expr, E> {
    comb::map(
        seq::pair(
            identifier::parse,
            seq::delimited(
                tag_token!(Token::LeftParen),
                comb::opt(seq::pair(
                    self::parse,
                    multi::many0(seq::preceded(tag_token!(Token::Comma), self::parse)),
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
