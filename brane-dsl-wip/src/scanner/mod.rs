mod identifier;
mod literal;
mod tokens;
mod utils;

use nom::error::{ContextError, ParseError};
use nom::{branch, combinator as comb, multi};
use nom::{bytes::complete as bc, IResult, Parser};
pub use tokens::{Token, Tokens};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;

///
///
///
pub fn scan_tokens<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, Vec<Token>, E> {
    comb::all_consuming(multi::many0(scan_token)).parse(input)
}

///
///
///
fn scan_token<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, Token, E> {
    branch::alt((
        keyword,
        operator,
        punctuation,
        literal::parse,
        identifier::parse,
    ))
    .parse(input)
}

///
///
///
fn keyword<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, Token, E> {
    utils::ws0(branch::alt((
        comb::map(bc::tag("&"), |s| Token::And(s)),
        comb::map(bc::tag("break"), |s| Token::Break(s)),
        comb::map(bc::tag("catch"), |s| Token::Catch(s)),
        comb::map(bc::tag("class"), |s| Token::Class(s)),
        comb::map(bc::tag("continue"), |s| Token::Continue(s)),
        comb::map(bc::tag("else"), |s| Token::Else(s)),
        comb::map(bc::tag("extends"), |s| Token::Extends(s)),
        comb::map(bc::tag("for"), |s| Token::For(s)),
        comb::map(bc::tag("func"), |s| Token::Function(s)),
        comb::map(bc::tag("if"), |s| Token::If(s)),
        comb::map(bc::tag("import"), |s| Token::Import(s)),
        comb::map(bc::tag("let"), |s| Token::Let(s)),
        comb::map(bc::tag("return"), |s| Token::Return(s)),
        comb::map(bc::tag("super"), |s| Token::Super(s)),
        comb::map(bc::tag("this"), |s| Token::This(s)),
        comb::map(bc::tag("try"), |s| Token::Try(s)),
        comb::map(bc::tag("unit"), |s| Token::Unit(s)),
        comb::map(bc::tag("while"), |s| Token::While(s)),
        comb::map(bc::tag("|"), |s| Token::Or(s)),
    )))
    .parse(input)
}

///
///
///
fn operator<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, Token, E> {
    utils::ws0(branch::alt((
        // Two character tokens
        comb::map(bc::tag(":="), |s| Token::Assign(s)),
        comb::map(bc::tag("=="), |s| Token::Equal(s)),
        comb::map(bc::tag(">="), |s| Token::GreaterOrEqual(s)),
        comb::map(bc::tag("<="), |s| Token::LessOrEqual(s)),
        comb::map(bc::tag("!="), |s| Token::NotEqual(s)),
        // One character token
        comb::map(bc::tag(">"), |s| Token::Greater(s)),
        comb::map(bc::tag("<"), |s| Token::Less(s)),
        comb::map(bc::tag("-"), |s| Token::Minus(s)),
        comb::map(bc::tag("!"), |s| Token::Not(s)),
        comb::map(bc::tag("+"), |s| Token::Plus(s)),
        comb::map(bc::tag("/"), |s| Token::Slash(s)),
        comb::map(bc::tag("*"), |s| Token::Star(s)),
    )))
    .parse(input)
}

///
///
///
fn punctuation<'a, E: ParseError<Span<'a>> + ContextError<Span<'a>>>(
    i: Span<'a>,
) -> IResult<Span<'a>, Token, E> {
    utils::ws0(branch::alt((
        comb::map(bc::tag("."), |s| Token::Dot(s)),
        comb::map(bc::tag(":"), |s| Token::Colon(s)),
        comb::map(bc::tag(","), |s| Token::Comma(s)),
        comb::map(bc::tag("{"), |s| Token::LeftBrace(s)),
        comb::map(bc::tag("["), |s| Token::LeftBracket(s)),
        comb::map(bc::tag("("), |s| Token::LeftParen(s)),
        comb::map(bc::tag("}"), |s| Token::RightBrace(s)),
        comb::map(bc::tag("]"), |s| Token::RightBracket(s)),
        comb::map(bc::tag(")"), |s| Token::RightParen(s)),
        comb::map(bc::tag(";"), |s| Token::Semicolon(s)),
    )))
    .parse(i)
}
