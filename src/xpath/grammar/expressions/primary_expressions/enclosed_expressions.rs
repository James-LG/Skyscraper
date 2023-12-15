//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-enclosed-expr

use nom::{character::complete::char, combinator::opt, error::context, sequence::tuple};

use crate::xpath::grammar::{
    expressions::{expr, Expr},
    recipes::Res,
};

pub fn enclosed_expr(input: &str) -> Res<&str, EnclosedExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-EnclosedExpr
    context("enclosed_expr", tuple((char('{'), opt(expr), char('}'))))(input)
        .map(|(next_input, res)| (next_input, EnclosedExpr(res.1)))
}

#[derive(PartialEq, Debug)]
pub struct EnclosedExpr(Option<Expr>);