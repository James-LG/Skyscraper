//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-let-expressions

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

pub fn let_expr(input: &str) -> Res<&str, LetExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-LetExpr

    context(
        "let_expr",
        tuple((simple_let_clause, tag("return"), expr_single)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            LetExpr {
                clause: res.0,
                expr: res.2,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct LetExpr {
    pub clause: SimpleLetClause,
    pub expr: ExprSingle,
}

impl Display for LetExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt LetExpr")
    }
}

fn simple_let_clause(input: &str) -> Res<&str, SimpleLetClause> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SimpleLetClause

    context(
        "simple_let_clause",
        tuple((
            tag("let"),
            simple_let_binding,
            many0(tuple((char(','), simple_let_binding))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let extras = res.2.into_iter().map(|(_, binding)| binding).collect();
        (
            next_input,
            SimpleLetClause {
                binding: res.1,
                extras,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct SimpleLetClause {
    pub binding: SimpleLetBinding,
    pub extras: Vec<SimpleLetBinding>,
}

fn simple_let_binding(input: &str) -> Res<&str, SimpleLetBinding> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SimpleLetBinding

    context(
        "simple_let_binding",
        tuple((char('$'), var_name, tag(":="), expr_single)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            SimpleLetBinding {
                var: res.1,
                expr: res.3,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct SimpleLetBinding {
    pub var: VarName,
    pub expr: ExprSingle,
}
