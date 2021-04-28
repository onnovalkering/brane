use super::tokens::Token;
use nom;
use nom::error::{ContextError, ParseError};
use nom::{branch, combinator as comb, sequence as seq};
use nom::{bytes::complete as bc, IResult, Parser};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;

///
///
///
pub fn parse<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    branch::alt((single_line_comment, multi_line_comment)).parse(input)
}

///
///
///
pub fn single_line_comment<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(
    input: Span<'a>
) -> IResult<Span<'a>, Token, E> {
    comb::value(
        Token::None,
        seq::pair(bc::tag("//"), bc::is_not("\n\r")),
    )
    .parse(input)
}

///
///
///
pub fn multi_line_comment<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(
    input: Span<'a>
) -> IResult<Span<'a>, Token, E> {
    comb::value(
        Token::None,
        seq::tuple((bc::tag("/*"), comb::cut(seq::pair(bc::take_until("*/"), bc::tag("*/"))))),
    )
    .parse(input)
}
