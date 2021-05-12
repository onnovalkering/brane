use std::num::NonZeroUsize;
use crate::tag_token;
use crate::parser::{expression, identifier};
use crate::scanner::{Token, Tokens};
use super::ast::{Stmt, Expr};
use nom::error::{ContextError, ParseError};
use nom::{combinator as comb, multi, sequence as seq};
use nom::{IResult, Parser};

///
///
///
pub fn parse<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Expr, E> {
    comb::map(
        seq::preceded(
            tag_token!(Token::New),
            comb::cut(
                seq::pair(
                    identifier::parse,
                    seq::delimited(
                        tag_token!(Token::LeftBrace),
                        comb::opt(seq::pair(
                            instance_property_stmt,
                            multi::many0(seq::preceded(tag_token!(Token::Comma), instance_property_stmt)),
                        )),
                        tag_token!(Token::RightBrace),
                    )
                )
            )
        ),
        |(class, properties)| {
            let properties: Vec<Stmt> = properties
                .map(|(h, e)| [&[h], &e[..]].concat().to_vec())
                .unwrap_or_default();

            Expr::Instance { class, properties }
        },
    )
    .parse(input)
}

///
///
///
pub fn instance_property_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::separated_pair(identifier::parse, tag_token!(Token::Assign), expression::parse),
        |(ident, expr)| Stmt::Assign(ident, expr),
    )
    .parse(input)
}
