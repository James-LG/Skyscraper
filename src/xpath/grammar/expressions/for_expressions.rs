//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-for-expressions

use std::fmt::Display;

use nom::{
    bytes::complete::tag, character::complete::char, error::context, multi::many0, sequence::tuple,
};

use crate::xpath::grammar::recipes::Res;

use super::{
    expr_single,
    primary_expressions::variable_references::{var_name, VarName},
    ExprSingle,
};

pub fn for_expr(input: &str) -> Res<&str, ForExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ForExpr

    context(
        "for_expr",
        tuple((simple_for_clause, tag("return"), expr_single)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            ForExpr {
                clause: res.0,
                expr: res.2,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct ForExpr {
    pub clause: SimpleForClause,
    pub expr: ExprSingle,
}

impl Display for ForExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt ForExpr")
    }
}

fn simple_for_clause(input: &str) -> Res<&str, SimpleForClause> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SimpleForClause

    context(
        "simple_for_clause",
        tuple((
            tag("for"),
            simple_for_binding,
            many0(tuple((char(','), simple_for_binding))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let extras = res.2.into_iter().map(|(_, binding)| binding).collect();
        (
            next_input,
            SimpleForClause {
                binding: res.1,
                extras,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleForClause {
    pub binding: SimpleForBinding,
    pub extras: Vec<SimpleForBinding>,
}

fn simple_for_binding(input: &str) -> Res<&str, SimpleForBinding> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SimpleForClause

    context(
        "simple_for_binding",
        tuple((char('$'), var_name, tag("in"), expr_single)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            SimpleForBinding {
                var: res.1,
                expr: res.3,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleForBinding {
    pub var: VarName,
    pub expr: ExprSingle,
}
