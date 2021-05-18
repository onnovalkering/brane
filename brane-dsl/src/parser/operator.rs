use super::ast::{BinOp, Operator, UnOp};
use crate::scanner::{Token, Tokens};
use crate::tag_token;
use nom::error::{ContextError, ParseError};
use nom::{branch, combinator as comb};
use nom::{IResult, Parser};
use std::num::NonZeroUsize;

///
///
///
pub fn parse<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, Operator, E> {
    branch::alt((
        comb::map(binary_operator, |x| Operator::Binary(x)),
        comb::map(unary_operator, |x| Operator::Unary(x)),
    ))
    .parse(input)
}

///
///
///
pub fn binary_operator<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, BinOp, E> {
    branch::alt((
        comb::map(tag_token!(Token::And), |_| BinOp::And),
        comb::map(tag_token!(Token::Equal), |_| BinOp::Eq),
        comb::map(tag_token!(Token::Greater), |_| BinOp::Gt),
        comb::map(tag_token!(Token::GreaterOrEqual), |_| BinOp::Ge),
        comb::map(tag_token!(Token::Less), |_| BinOp::Lt),
        comb::map(tag_token!(Token::LessOrEqual), |_| BinOp::Le),
        comb::map(tag_token!(Token::Minus), |_| BinOp::Sub),
        comb::map(tag_token!(Token::NotEqual), |_| BinOp::Ne),
        comb::map(tag_token!(Token::Or), |_| BinOp::Or),
        comb::map(tag_token!(Token::Plus), |_| BinOp::Add),
        comb::map(tag_token!(Token::Slash), |_| BinOp::Div),
        comb::map(tag_token!(Token::Star), |_| BinOp::Mul),
        comb::map(tag_token!(Token::Dot), |_| BinOp::Dot),
    ))
    .parse(input)
}

///
///
///
pub fn unary_operator<'a, E: ParseError<Tokens<'a>> + ContextError<Tokens<'a>>>(
    input: Tokens<'a>
) -> IResult<Tokens, UnOp, E> {
    branch::alt((
        comb::map(tag_token!(Token::Not), |_| UnOp::Not),
        comb::map(tag_token!(Token::Minus), |_| UnOp::Neg),
        comb::map(tag_token!(Token::LeftBracket), |_| UnOp::Idx),
        comb::map(tag_token!(Token::LeftParen), |_| UnOp::Prio),
    ))
    .parse(input)
}
