//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-expressions

use nom::{branch::alt, character::complete::char, multi::many0, sequence::tuple};

use crate::xpath::grammar::expressions::{
    conditional_expressions::if_expr, for_expressions::for_expr, let_expressions::let_expr,
    logical_expressions::or_expr, quantified_expressions::quantified_expr,
};

use self::{
    conditional_expressions::IfExpr, for_expressions::ForExpr, let_expressions::LetExpr,
    logical_expressions::OrExpr, quantified_expressions::QuantifiedExpr,
};

use super::recipes::Res;

mod arithmetic_expressions;
mod arrow_operator;
mod common;
mod comparison_expressions;
mod conditional_expressions;
mod expressions_on_sequence_types;
mod for_expressions;
mod let_expressions;
mod logical_expressions;
mod maps_and_arrays;
mod path_expressions;
mod postfix_expressions;
mod primary_expressions;
mod quantified_expressions;
mod sequence_expressions;
mod simple_map_operator;
mod string_concat_expressions;

pub fn xpath(input: &str) -> Res<&str, XPath> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-XPath

    expr(input).map(|(next_input, res)| (next_input, XPath(res)))
}

pub struct XPath(Expr);

pub fn expr(input: &str) -> Res<&str, Expr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Expr

    tuple((expr_single, many0(tuple((char(','), expr_single)))))(input).map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, Expr { expr: res.0, items })
    })
}

pub struct Expr {
    pub expr: ExprSingle,
    pub items: Vec<ExprSingle>,
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

    alt((
        for_expr_map,
        let_expr_map,
        quantified_expr_map,
        if_expr_map,
        or_expr_map,
    ))(input)
}

pub enum ExprSingle {
    ForExpr(Box<ForExpr>),
    LetExpr(Box<LetExpr>),
    QuantifiedExpr(Box<QuantifiedExpr>),
    IfExpr(Box<IfExpr>),
    OrExpr(Box<OrExpr>),
}
