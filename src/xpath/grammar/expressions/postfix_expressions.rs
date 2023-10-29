//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-postfix-expression

use std::fmt::Display;

use nom::{branch::alt, character::complete::char, multi::many0, sequence::tuple};

use crate::xpath::grammar::{
    expressions::{
        common::argument_list, maps_and_arrays::lookup_operator::postfix_lookup::lookup,
        primary_expressions::primary_expr,
    },
    recipes::Res,
};

use super::{
    common::ArgumentList, expr, maps_and_arrays::lookup_operator::postfix_lookup::Lookup,
    primary_expressions::PrimaryExpr, Expr,
};

pub fn postfix_expr(input: &str) -> Res<&str, PostfixExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PostfixExpr

    fn predicate_map(input: &str) -> Res<&str, PostfixExprItem> {
        predicate(input).map(|(next_input, res)| (next_input, PostfixExprItem::Predicate(res)))
    }

    fn argument_list_map(input: &str) -> Res<&str, PostfixExprItem> {
        argument_list(input)
            .map(|(next_input, res)| (next_input, PostfixExprItem::ArgumentList(res)))
    }

    fn lookup_map(input: &str) -> Res<&str, PostfixExprItem> {
        lookup(input).map(|(next_input, res)| (next_input, PostfixExprItem::Lookup(res)))
    }

    tuple((
        primary_expr,
        many0(alt((predicate_map, argument_list_map, lookup_map))),
    ))(input)
    .map(|(next_input, res)| {
        (
            next_input,
            PostfixExpr {
                expr: res.0,
                items: res.1,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct PostfixExpr {
    pub expr: PrimaryExpr,
    pub items: Vec<PostfixExprItem>,
}

impl Display for PostfixExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, "{}", x)?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub enum PostfixExprItem {
    Predicate(Predicate),
    ArgumentList(ArgumentList),
    Lookup(Lookup),
}

impl Display for PostfixExprItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostfixExprItem::Predicate(x) => write!(f, "{}", x),
            PostfixExprItem::ArgumentList(x) => write!(f, "{}", x),
            PostfixExprItem::Lookup(x) => write!(f, "{}", x),
        }
    }
}

pub fn predicate(input: &str) -> Res<&str, Predicate> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Predicate
    tuple((char('['), expr, char(']')))(input)
        .map(|(next_input, res)| (next_input, Predicate(res.1)))
}

#[derive(PartialEq, Debug)]
pub struct Predicate(Expr);

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}
