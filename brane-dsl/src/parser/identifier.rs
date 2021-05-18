use super::ast::Ident;
use crate::scanner::{Token, Tokens};
use crate::tag_token;
use nom::combinator as comb;
use nom::error::{ContextError, ParseError};
use nom::{IResult, Parser};
use std::num::NonZeroUsize;

///
///
///
pub fn parse<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(input: Tokens<'a>) -> IResult<Tokens, Ident, E> {
    comb::map(tag_token!(Token::Ident), |x| Ident(x.tok[0].as_string())).parse(input)
}
