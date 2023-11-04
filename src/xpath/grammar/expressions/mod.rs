//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-expressions

use std::fmt::Display;

use nom::{branch::alt, character::complete::char, error::context, multi::many0, sequence::tuple};

use crate::xpath::grammar::{
    expressions::{
        conditional_expressions::if_expr, for_expressions::for_expr, let_expressions::let_expr,
        logical_expressions::or_expr, quantified_expressions::quantified_expr,
    },
    recipes::max,
};

use self::{
    conditional_expressions::IfExpr, for_expressions::ForExpr, let_expressions::LetExpr,
    logical_expressions::OrExpr, quantified_expressions::QuantifiedExpr,
};

use super::recipes::Res;

pub mod arithmetic_expressions;
pub mod arrow_operator;
pub mod common;
pub mod comparison_expressions;
pub mod conditional_expressions;
pub mod expressions_on_sequence_types;
pub mod for_expressions;
pub mod let_expressions;
pub mod logical_expressions;
pub mod maps_and_arrays;
pub mod path_expressions;
pub mod postfix_expressions;
pub mod primary_expressions;
pub mod quantified_expressions;
pub mod sequence_expressions;
pub mod simple_map_operator;
pub mod string_concat_expressions;

pub fn xpath(input: &str) -> Res<&str, XPath> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-XPath

    context("xpath", expr)(input).map(|(next_input, res)| (next_input, XPath(res)))
}

#[derive(PartialEq, Debug)]
pub struct XPath(pub Expr);

impl Display for XPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn expr(input: &str) -> Res<&str, Expr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Expr

    context(
        "expr",
        tuple((expr_single, many0(tuple((char(','), expr_single))))),
    )(input)
    .map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, Expr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug)]
pub struct Expr {
    pub expr: ExprSingle,
    pub items: Vec<ExprSingle>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " {}", x)?;
        }

        Ok(())
    }
}

pub fn expr_single(input: &str) -> Res<&str, ExprSingle> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ExprSingle

    fn for_expr_map(input: &str) -> Res<&str, ExprSingle> {
        for_expr(input).map(|(next_input, res)| (next_input, ExprSingle::ForExpr(Box::new(res))))
    }

    fn let_expr_map(input: &str) -> Res<&str, ExprSingle> {
        let_expr(input).map(|(next_input, res)| (next_input, ExprSingle::LetExpr(Box::new(res))))
    }

    fn quantified_expr_map(input: &str) -> Res<&str, ExprSingle> {
        quantified_expr(input)
            .map(|(next_input, res)| (next_input, ExprSingle::QuantifiedExpr(Box::new(res))))
    }

    fn if_expr_map(input: &str) -> Res<&str, ExprSingle> {
        if_expr(input).map(|(next_input, res)| (next_input, ExprSingle::IfExpr(Box::new(res))))
    }

    fn or_expr_map(input: &str) -> Res<&str, ExprSingle> {
        or_expr(input).map(|(next_input, res)| (next_input, ExprSingle::OrExpr(Box::new(res))))
    }

    context(
        "expr_single",
        max((
            for_expr_map,
            let_expr_map,
            quantified_expr_map,
            if_expr_map,
            or_expr_map,
        )),
    )(input)
}

#[derive(PartialEq, Debug)]
pub enum ExprSingle {
    ForExpr(Box<ForExpr>),
    LetExpr(Box<LetExpr>),
    QuantifiedExpr(Box<QuantifiedExpr>),
    IfExpr(Box<IfExpr>),
    OrExpr(Box<OrExpr>),
}

impl Display for ExprSingle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprSingle::ForExpr(x) => write!(f, "{}", x),
            ExprSingle::LetExpr(x) => write!(f, "{}", x),
            ExprSingle::QuantifiedExpr(x) => write!(f, "{}", x),
            ExprSingle::IfExpr(x) => write!(f, "{}", x),
            ExprSingle::OrExpr(x) => write!(f, "{}", x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expr_should_parse1() {
        // arrange
        let input = "/(chapter|appendix)";

        // act
        let (next_input, res) = expr(input).unwrap();

        // assert
        assert_eq!(res.to_string(), input);
        assert_eq!(next_input, "");
    }
}
