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
        comb::map(bc::tag("break"), Token::Break),
        comb::map(bc::tag("class"), Token::Class),
        comb::map(bc::tag("continue"), Token::Continue),
        comb::map(bc::tag("else"), Token::Else),
        comb::map(bc::tag("for"), Token::For),
        comb::map(bc::tag("func"), Token::Function),
        comb::map(bc::tag("if"), Token::If),
        comb::map(bc::tag("import"), Token::Import),
        comb::map(bc::tag("let"), Token::Let),
        comb::map(bc::tag("new"), Token::New),
        comb::map(bc::tag("on"), Token::On),
        comb::map(bc::tag("parallel"), Token::Parallel),
        comb::map(bc::tag("return"), Token::Return),
        comb::map(bc::tag("unit"), Token::Unit),
        comb::map(bc::tag("while"), Token::While),
    )))
    .parse(input)
}

///
///
///
fn operator<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    ws0(branch::alt((
        // Two character tokens
        comb::map(bc::tag(":="), Token::Assign),
        comb::map(bc::tag("=="), Token::Equal),
        comb::map(bc::tag(">="), Token::GreaterOrEqual),
        comb::map(bc::tag("<="), Token::LessOrEqual),
        comb::map(bc::tag("!="), Token::NotEqual),
        // One character token
        comb::map(bc::tag("!"), Token::Not),
        comb::map(bc::tag("&"), Token::And),
        comb::map(bc::tag("*"), Token::Star),
        comb::map(bc::tag("+"), Token::Plus),
        comb::map(bc::tag("-"), Token::Minus),
        comb::map(bc::tag("/"), Token::Slash),
        comb::map(bc::tag("<"), Token::Less),
        comb::map(bc::tag(">"), Token::Greater),
        comb::map(bc::tag("|"), Token::Or),
    )))
    .parse(input)
}

///
///
///
fn punctuation<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Token, E> {
    ws0(branch::alt((
        comb::map(bc::tag("("), Token::LeftParen),
        comb::map(bc::tag(")"), Token::RightParen),
        comb::map(bc::tag(","), Token::Comma),
        comb::map(bc::tag("."), Token::Dot),
        comb::map(bc::tag(":"), Token::Colon),
        comb::map(bc::tag(";"), Token::Semicolon),
        comb::map(bc::tag("["), Token::LeftBracket),
        comb::map(bc::tag("]"), Token::RightBracket),
        comb::map(bc::tag("{"), Token::LeftBrace),
        comb::map(bc::tag("}"), Token::RightBrace),
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
        Token::Ident,
    ))
    .parse(input)
}

///
///
///
pub fn ws0<'a, O, E: ParseError<Span<'a>>, F: Parser<Span<'a>, O, E>>(f: F) -> impl Parser<Span<'a>, O, E> {
    seq::delimited(cc::multispace0, f, cc::multispace0)
}
