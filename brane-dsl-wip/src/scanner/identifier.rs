use super::tokens::Token;
use nom;
use nom::error::{ContextError, ParseError};
use nom::{branch, character::complete as cc, combinator as comb, multi, sequence as seq};
use nom::{bytes::complete as bc, IResult, Parser};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;

///
///
///
pub fn parse<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    comb::map(ident, |s| Token::Ident(s)).parse(input)
}

///
///
///
fn ident<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    comb::recognize(seq::pair(
        branch::alt((cc::alpha1, bc::tag("_"))),
        multi::many0(branch::alt((cc::alphanumeric1, bc::tag("_")))),
    ))(input)
}
