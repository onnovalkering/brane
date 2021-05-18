mod comments;
mod literal;
mod tokens;

use nom::error::{ContextError, ParseError, VerboseError};
use nom::{branch, combinator as comb, multi, sequence as seq};
use nom::{bytes::complete as bc, character::complete as cc, IResult, Parser};
pub use tokens::{Token, Tokens};

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;

///
///
///
pub fn scan_tokens(input: Span) -> IResult<Span, Vec<Token>, VerboseError<Span>> {
    comb::all_consuming(multi::many0(scan_token))
        .parse(input)
        .map(|(s, t)| {
            let mut t = t;
            t.retain(|t| !t.is_none());

            (s, t)
        })
}

///
///
///
fn scan_token<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    branch::alt((
        comments::parse,
        keyword,
        operator,
        punctuation,
        literal::parse,
        identifier,
    ))
    .parse(input)
}

///
///
///
fn keyword<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    ws0(branch::alt((
        comb::map(bc::tag("break"), |s| Token::Break(s)),
        comb::map(bc::tag("class"), |s| Token::Class(s)),
        comb::map(bc::tag("continue"), |s| Token::Continue(s)),
        comb::map(bc::tag("else"), |s| Token::Else(s)),
        comb::map(bc::tag("for"), |s| Token::For(s)),
        comb::map(bc::tag("func"), |s| Token::Function(s)),
        comb::map(bc::tag("if"), |s| Token::If(s)),
        comb::map(bc::tag("import"), |s| Token::Import(s)),
        comb::map(bc::tag("let"), |s| Token::Let(s)),
        comb::map(bc::tag("new"), |s| Token::New(s)),
        comb::map(bc::tag("on"), |s| Token::On(s)),
        comb::map(bc::tag("parallel"), |s| Token::Parallel(s)),
        comb::map(bc::tag("return"), |s| Token::Return(s)),
        comb::map(bc::tag("unit"), |s| Token::Unit(s)),
        comb::map(bc::tag("while"), |s| Token::While(s)),
    )))
    .parse(input)
}

///
///
///
fn operator<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    ws0(branch::alt((
        // Two character tokens
        comb::map(bc::tag(":="), |s| Token::Assign(s)),
        comb::map(bc::tag("=="), |s| Token::Equal(s)),
        comb::map(bc::tag(">="), |s| Token::GreaterOrEqual(s)),
        comb::map(bc::tag("<="), |s| Token::LessOrEqual(s)),
        comb::map(bc::tag("!="), |s| Token::NotEqual(s)),
        // One character token
        comb::map(bc::tag("!"), |s| Token::Not(s)),
        comb::map(bc::tag("&"), |s| Token::And(s)),
        comb::map(bc::tag("*"), |s| Token::Star(s)),
        comb::map(bc::tag("+"), |s| Token::Plus(s)),
        comb::map(bc::tag("-"), |s| Token::Minus(s)),
        comb::map(bc::tag("/"), |s| Token::Slash(s)),
        comb::map(bc::tag("<"), |s| Token::Less(s)),
        comb::map(bc::tag(">"), |s| Token::Greater(s)),
        comb::map(bc::tag("|"), |s| Token::Or(s)),
    )))
    .parse(input)
}

///
///
///
fn punctuation<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    ws0(branch::alt((
        comb::map(bc::tag("("), |s| Token::LeftParen(s)),
        comb::map(bc::tag(")"), |s| Token::RightParen(s)),
        comb::map(bc::tag(","), |s| Token::Comma(s)),
        comb::map(bc::tag("."), |s| Token::Dot(s)),
        comb::map(bc::tag(":"), |s| Token::Colon(s)),
        comb::map(bc::tag(";"), |s| Token::Semicolon(s)),
        comb::map(bc::tag("["), |s| Token::LeftBracket(s)),
        comb::map(bc::tag("]"), |s| Token::RightBracket(s)),
        comb::map(bc::tag("{"), |s| Token::LeftBrace(s)),
        comb::map(bc::tag("}"), |s| Token::RightBrace(s)),
    )))
    .parse(input)
}

///
///
///
fn identifier<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    ws0(comb::map(
        comb::recognize(seq::pair(
            branch::alt((cc::alpha1, bc::tag("_"))),
            multi::many0(branch::alt((cc::alphanumeric1, bc::tag("_")))),
        )),
        |s| Token::Ident(s),
    ))
    .parse(input)
}

///
///
///
pub fn ws0<'a, O, E: ParseError<Span<'a>>, F: Parser<Span<'a>, O, E>>(f: F) -> impl Parser<Span<'a>, O, E> {
    seq::delimited(cc::multispace0, f, cc::multispace0)
}
