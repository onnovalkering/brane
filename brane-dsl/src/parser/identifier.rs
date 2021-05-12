use std::num::NonZeroUsize;
use crate::tag_token;
use crate::scanner::{Token, Tokens};
use super::ast::Ident;
use nom::error::{ContextError, ParseError};
use nom::{combinator as comb};
use nom::{IResult, Parser};

///
///
///
pub fn parse<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(input: Tokens<'a>) -> IResult<Tokens, Ident, E> {
    comb::map(tag_token!(Token::Ident), |x| Ident(x.tok[0].as_string())).parse(input)
}
