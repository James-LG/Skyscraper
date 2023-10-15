//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-paren-expressions

use nom::{character::complete::char, combinator::opt, sequence::tuple};

use crate::xpath::grammar::{
    expressions::{expr, Expr},
    recipes::Res,
};

pub fn parenthesized_expr(input: &str) -> Res<&str, ParenthesizedExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ParenthesizedExpr
    tuple((char('('), opt(expr), char(')')))(input)
        .map(|(next_input, res)| (next_input, ParenthesizedExpr(res.1)))
}

pub struct ParenthesizedExpr(pub Option<Expr>);
