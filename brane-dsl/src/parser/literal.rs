use super::ast::Lit;
use crate::scanner::{Token, Tokens};
use crate::tag_token;
use nom::error::{ContextError, ParseError};
use nom::{branch, combinator as comb};
use nom::{IResult, Parser};
use std::num::NonZeroUsize;

///
///
///
pub fn parse<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(input: Tokens<'a>) -> IResult<Tokens, Lit, E> {
    branch::alt((
        comb::map(tag_token!(Token::Boolean), |t| Lit::Boolean(t.tok[0].as_bool())),
        comb::map(tag_token!(Token::Integer), |t| Lit::Integer(t.tok[0].as_i64())),
        comb::map(tag_token!(Token::Real), |t| Lit::Real(t.tok[0].as_f64())),
        comb::map(tag_token!(Token::String), |t| Lit::String(t.tok[0].as_string())),
        comb::map(tag_token!(Token::Unit), |_| Lit::Unit),
    ))
    .parse(input)
}
