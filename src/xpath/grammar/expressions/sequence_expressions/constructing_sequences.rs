//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#construct_seq

use nom::{bytes::complete::tag, combinator::opt, sequence::tuple};

use crate::xpath::grammar::{
    expressions::arithmetic_expressions::{additive_expr, AdditiveExpr},
    recipes::Res,
};

pub fn range_expr(input: &str) -> Res<&str, RangeExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-RangeExpr

    tuple((additive_expr, opt(tuple((tag("to"), additive_expr)))))(input).map(
        |(next_input, res)| {
            (
                next_input,
                RangeExpr {
                    expr: res.0,
                    to_expr: res.1.map(|res| res.1),
                },
            )
        },
    )
}

pub struct RangeExpr {
    pub expr: AdditiveExpr,
    pub to_expr: Option<AdditiveExpr>,
}
