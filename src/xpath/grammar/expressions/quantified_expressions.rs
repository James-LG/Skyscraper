//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-quantified-expressions

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, multi::many0, sequence::tuple,
};

use crate::xpath::grammar::{
    expressions::{expr_single, primary_expressions::variable_references::var_name},
    recipes::Res,
};

use super::{primary_expressions::variable_references::VarName, ExprSingle};

pub fn quantified_expr(input: &str) -> Res<&str, QuantifiedExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-QuantifiedExpr

    fn some_quantifier(input: &str) -> Res<&str, Quantifier> {
        tag("some")(input).map(|(next_input, _res)| (next_input, Quantifier::Some))
    }

    fn every_quantifier(input: &str) -> Res<&str, Quantifier> {
        tag("every")(input).map(|(next_input, _res)| (next_input, Quantifier::Every))
    }

    tuple((
        alt((some_quantifier, every_quantifier)),
        char('$'),
        var_name,
        tag("in"),
        expr_single,
        many0(tuple((
            char(','),
            char('$'),
            var_name,
            tag("in"),
            expr_single,
        ))),
        tag("satisfies"),
        expr_single,
    ))(input)
    .map(|(next_input, res)| {
        let extras = res
            .5
            .into_iter()
            .map(|r| QuantifiedExprItem {
                var: r.2,
                expr: r.4,
            })
            .collect();
        (
            next_input,
            QuantifiedExpr {
                quantifier: res.0,
                item: QuantifiedExprItem {
                    var: res.2,
                    expr: res.4,
                },
                extras,
                satisfies: res.7,
            },
        )
    })
}

pub struct QuantifiedExpr {
    pub quantifier: Quantifier,
    pub item: QuantifiedExprItem,
    pub extras: Vec<QuantifiedExprItem>,
    pub satisfies: ExprSingle,
}

pub enum Quantifier {
    Some,
    Every,
}

pub struct QuantifiedExprItem {
    pub var: VarName,
    pub expr: ExprSingle,
}
