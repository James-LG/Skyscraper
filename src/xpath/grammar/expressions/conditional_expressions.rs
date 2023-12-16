//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-conditionals

use std::fmt::Display;

use nom::{bytes::complete::tag, character::complete::char, error::context, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::{expr, expr_single, Expr, ExprSingle};

pub fn if_expr(input: &str) -> Res<&str, IfExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-IfExpr

    context(
        "if_expr",
        tuple((
            tag("if"),
            char('('),
            expr,
            char(')'),
            tag("then"),
            expr_single,
            tag("else"),
            expr_single,
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            IfExpr {
                condition: res.2,
                then: res.5,
                else_expr: res.7,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct IfExpr {
    pub condition: Expr,
    pub then: ExprSingle,
    pub else_expr: ExprSingle,
}

impl Display for IfExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt IfExpr")
    }
}
