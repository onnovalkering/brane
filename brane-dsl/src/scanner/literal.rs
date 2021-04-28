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
    branch::alt((
        comb::map(semver, |s| Token::SemVer(s)),
        comb::map(real, |s| Token::Real(s)),
        comb::map(integer, |s| Token::Integer(s)),
        comb::map(boolean, |s| Token::Boolean(s)),
        comb::map(string, |s| Token::String(s)),
    ))
    .parse(input)
}

///
///
///
fn boolean<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    branch::alt((bc::tag("true"), bc::tag("false"))).parse(input)
}

///
///
///
fn integer<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    comb::recognize(multi::many1(seq::terminated(
        cc::one_of("0123456789"),
        multi::many0(cc::char('_')),
    )))
    .parse(input)
}

///
///
///
fn semver<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    let numbers = "0123456789";

    comb::recognize(seq::tuple((
        multi::many1(cc::one_of(numbers)),
        seq::delimited(cc::char('.'), multi::many1(cc::one_of(numbers)), cc::char('.')),
        multi::many1(cc::one_of(numbers)),
    )))
    .parse(input)
}

///
///
///
fn string<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    nom::error::context(
        "string",
        seq::preceded(
            cc::char('\"'),
            comb::cut(seq::terminated(
                bc::escaped(bc::is_not("\""), '\\', cc::one_of("\"n\\")),
                cc::char('\"'),
            )),
        ),
    )(input)
}

///
///
///
fn real<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    branch::alt((
        comb::recognize(seq::tuple((
            cc::char('.'),
            integer,
            comb::opt(seq::tuple((cc::one_of("eE"), comb::opt(cc::one_of("+-")), integer))),
        ))),
        comb::recognize(seq::tuple((
            integer,
            comb::opt(seq::preceded(cc::char('.'), integer)),
            cc::one_of("eE"),
            comb::opt(cc::one_of("+-")),
            integer,
        ))),
        comb::recognize(seq::tuple((integer, cc::char('.'), comb::opt(integer)))),
    ))(input)
}
