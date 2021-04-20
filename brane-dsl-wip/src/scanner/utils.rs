use nom;
use nom::error::ParseError;
use nom::Parser;
use nom::{character::complete as cc, sequence as seq};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;

///
///
///
pub fn ws0<'a, O, E: ParseError<Span<'a>>, F: Parser<Span<'a>, O, E>>(f: F) -> impl Parser<Span<'a>, O, E> {
    seq::delimited(cc::multispace0, f, cc::multispace0)
}
