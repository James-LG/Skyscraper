//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-paren-expressions

use std::fmt::Display;

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

#[derive(PartialEq, Debug)]
pub struct ParenthesizedExpr(pub Option<Expr>);

impl Display for ParenthesizedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        if let Some(x) = &self.0 {
            write!(f, "{}", x)?;
        }
        write!(f, ")")
    }
}
