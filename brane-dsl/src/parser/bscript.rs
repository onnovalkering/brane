use super::ast::Stmt;
use crate::parser::{expression, identifier};
use crate::scanner::{Token, Tokens};
use crate::tag_token;
use nom::error::{ContextError, ErrorKind, ParseError, VerboseError};
use nom::{branch, combinator as comb, multi, sequence as seq};
use nom::{IResult, Parser};
use semver::Version;
use std::{collections::HashMap, num::NonZeroUsize};

///
///
///
pub fn parse_ast(input: Tokens) -> IResult<Tokens, Vec<Stmt>, VerboseError<Tokens>> {
    comb::all_consuming(multi::many0(parse_stmt))(input)
}

///
///
///
pub fn parse_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    if input.tok.is_empty() {
        return Err(nom::Err::Error(nom::error_position!(input, ErrorKind::Tag)));
    }

    branch::alt((
        for_stmt,
        assign_stmt,
        on_stmt,
        block_stmt,
        parallel_stmt,
        declare_class_stmt,
        declare_func_stmt,
        expr_stmt,
        if_stmt,
        import_stmt,
        let_assign_stmt,
        return_stmt,
        while_stmt,
    ))
    .parse(input)
}

///
///
///
pub fn let_assign_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::preceded(
            tag_token!(Token::Let),
            comb::cut(seq::terminated(
                seq::separated_pair(identifier::parse, tag_token!(Token::Assign), expression::parse),
                tag_token!(Token::Semicolon),
            )),
        ),
        |(ident, expr)| Stmt::LetAssign(ident, expr),
    )
    .parse(input)
}

///
///
///
pub fn assign_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::terminated(
            seq::separated_pair(identifier::parse, tag_token!(Token::Assign), expression::parse),
            comb::cut(tag_token!(Token::Semicolon)),
        ),
        |(ident, expr)| Stmt::Assign(ident, expr),
    )
    .parse(input)
}

///
///
///
pub fn on_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::pair(
            seq::preceded(tag_token!(Token::On), comb::cut(expression::parse)),
            comb::cut(seq::delimited(
                tag_token!(Token::LeftBrace),
                multi::many0(parse_stmt),
                tag_token!(Token::RightBrace),
            )),
        ),
        |(location, block)| Stmt::On { location, block },
    )
    .parse(input)
}

///
///
///
pub fn block_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::delimited(
            tag_token!(Token::LeftBrace),
            multi::many0(parse_stmt),
            tag_token!(Token::RightBrace),
        ),
        Stmt::Block,
    )
    .parse(input)
}

///
///
///
pub fn parallel_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    let block_or_on = |input| branch::alt((on_stmt, block_stmt)).parse(input);

    comb::map(
        seq::pair(
            comb::opt(seq::preceded(
                tag_token!(Token::Let),
                comb::cut(seq::terminated(identifier::parse, tag_token!(Token::Assign))),
            )),
            seq::preceded(
                tag_token!(Token::Parallel),
                comb::cut(seq::terminated(
                    seq::delimited(
                        tag_token!(Token::LeftBracket),
                        comb::opt(seq::pair(
                            block_or_on,
                            multi::many0(seq::preceded(tag_token!(Token::Comma), block_or_on)),
                        )),
                        tag_token!(Token::RightBracket),
                    ),
                    tag_token!(Token::Semicolon),
                )),
            ),
        ),
        |(let_assign, blocks)| {
            let blocks = blocks
                .map(|(h, e)| {
                    // Combine head and entries
                    [&[h], &e[..]].concat().to_vec()
                })
                .unwrap_or_default();

            Stmt::Parallel { let_assign, blocks }
        },
    )
    .parse(input)
}

///
///
///
pub fn declare_class_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::tuple((
            seq::preceded(tag_token!(Token::Class), identifier::parse),
            seq::delimited(
                tag_token!(Token::LeftBrace),
                multi::many0(branch::alt((declare_property_stmt, declare_func_stmt))),
                tag_token!(Token::RightBrace),
            ),
        )),
        |(ident, body)| {
            let mut properties = HashMap::new();
            let mut methods = HashMap::new();

            for stmt in body.iter() {
                match stmt {
                    Stmt::Property { ident, class } => { properties.insert(ident.clone(), class.clone()); }
                    Stmt::DeclareFunc { ident, .. } => { methods.insert(ident.clone(), stmt.clone()); }
                    _ => unreachable!()
                }
            }

            Stmt::DeclareClass { ident, properties, methods }
        },
    )
    .parse(input)
}

///
///
///
pub fn declare_property_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::terminated(
            seq::separated_pair(identifier::parse, tag_token!(Token::Colon), identifier::parse),
            comb::cut(tag_token!(Token::Semicolon)),
        ),
        |(ident, class)| Stmt::Property { ident, class },
    )
    .parse(input)
}

///
///
///
pub fn declare_func_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::tuple((
            seq::preceded(
                tag_token!(Token::Function),
                comb::cut(seq::pair(
                    identifier::parse,
                    seq::delimited(
                        tag_token!(Token::LeftParen),
                        comb::opt(seq::pair(
                            identifier::parse,
                            multi::many0(seq::preceded(tag_token!(Token::Comma), identifier::parse)),
                        )),
                        tag_token!(Token::RightParen),
                    ),
                )),
            ),
            comb::cut(seq::delimited(
                tag_token!(Token::LeftBrace),
                multi::many0(parse_stmt),
                tag_token!(Token::RightBrace),
            )),
        )),
        |((ident, params), body)| {
            let params = params
                .map(|(h, e)| {
                    // Combine head and entries
                    [&[h], &e[..]].concat().to_vec()
                })
                .unwrap_or_default();

            Stmt::DeclareFunc { ident, params, body }
        },
    )
    .parse(input)
}

///
///
///
pub fn if_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::preceded(
            tag_token!(Token::If),
            comb::cut(seq::tuple((
                seq::delimited(
                    tag_token!(Token::LeftParen),
                    expression::parse,
                    tag_token!(Token::RightParen),
                ),
                seq::delimited(
                    tag_token!(Token::LeftBrace),
                    multi::many0(parse_stmt),
                    tag_token!(Token::RightBrace),
                ),
                comb::opt(seq::preceded(
                    tag_token!(Token::Else),
                    seq::delimited(
                        tag_token!(Token::LeftBrace),
                        multi::many0(parse_stmt),
                        tag_token!(Token::RightBrace),
                    ),
                )),
            ))),
        ),
        |(condition, consequent, alternative)| Stmt::If {
            condition,
            consequent,
            alternative,
        },
    )
    .parse(input)
}

///
///
///
pub fn import_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    nom::error::context(
        "'import' statement",
        comb::map(
            seq::preceded(
                tag_token!(Token::Import),
                comb::cut(seq::terminated(
                    seq::pair(
                        identifier::parse,
                        comb::opt(seq::delimited(
                            tag_token!(Token::LeftBracket),
                            comb::map(tag_token!(Token::SemVer), |x| {
                                Version::parse(&x.tok[0].as_string()).unwrap()
                            }),
                            tag_token!(Token::RightBracket),
                        )),
                    ),
                    tag_token!(Token::Semicolon),
                )),
            ),
            |(package, version)| Stmt::Import { package, version },
        ),
    )
    .parse(input)
}

///
///
///
pub fn for_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    nom::error::context(
        "'for' statement",
        comb::map(
            seq::preceded(
                tag_token!(Token::For),
                comb::cut(seq::pair(
                    seq::delimited(
                        tag_token!(Token::LeftParen),
                        seq::tuple((
                            let_assign_stmt,
                            seq::terminated(expression::parse, tag_token!(Token::Semicolon)),
                            comb::map(
                                seq::separated_pair(identifier::parse, tag_token!(Token::Assign), expression::parse),
                                |(ident, expr)| Stmt::Assign(ident, expr),
                            ),
                        )),
                        tag_token!(Token::RightParen),
                    ),
                    seq::delimited(
                        tag_token!(Token::LeftBrace),
                        multi::many0(parse_stmt),
                        tag_token!(Token::RightBrace),
                    ),
                )),
            ),
            |((initializer, condition, increment), consequent)| Stmt::For {
                initializer: Box::new(initializer),
                condition,
                increment: Box::new(increment),
                consequent,
            },
        ),
    )
    .parse(input)
}

///
///
///
pub fn while_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::pair(
            seq::preceded(
                tag_token!(Token::While),
                seq::delimited(
                    tag_token!(Token::LeftParen),
                    expression::parse,
                    tag_token!(Token::RightParen),
                ),
            ),
            seq::delimited(
                tag_token!(Token::LeftBrace),
                multi::many0(parse_stmt),
                tag_token!(Token::RightBrace),
            ),
        ),
        |(condition, consequent)| Stmt::While { condition, consequent },
    )
    .parse(input)
}

///
///
///
pub fn return_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::delimited(
            tag_token!(Token::Return),
            comb::opt(expression::parse),
            comb::cut(tag_token!(Token::Semicolon)),
        ),
        Stmt::Return,
    )
    .parse(input)
}

///
///
///
pub fn expr_stmt<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Stmt, E> {
    comb::map(
        seq::terminated(expression::parse, comb::cut(tag_token!(Token::Semicolon))),
        Stmt::Expr,
    )
    .parse(input)
}
