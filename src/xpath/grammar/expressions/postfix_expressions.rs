//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-postfix-expression

use std::fmt::Display;

use indexmap::{indexset, IndexSet};
use nom::{branch::alt, character::complete::char, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::{Node, XpathItem},
        expressions::{
            common::argument_list, maps_and_arrays::lookup_operator::postfix_lookup::lookup,
            primary_expressions::primary_expr,
        },
        recipes::{ws, Res},
    },
    xpath_item_set, ExpressionApplyError, XPathExpressionContext, XPathResult, XpathItemSet,
};

use crate::xpath_item_set;

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

    context(
        "postfix_expr",
        tuple((
            primary_expr,
            many0(alt((predicate_map, argument_list_map, lookup_map))),
        )),
    )(input)
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

#[derive(PartialEq, Debug, Clone)]
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

impl PostfixExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let res = self.expr.eval(context)?;

        let items = match res {
            XPathResult::ItemSet(x) => x,
            XPathResult::Item(x) => xpath_item_set![x],
        };

        if !self.items.is_empty() {
            todo!("PostfixExpr eval items")
        }

        Ok(items)
    }
}

#[derive(PartialEq, Debug, Clone)]
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
    context("predicate", tuple((ws(char('[')), expr, ws(char(']')))))(input)
        .map(|(next_input, res)| (next_input, Predicate(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct Predicate(Expr);

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

impl Predicate {
    pub(crate) fn is_match<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<bool, ExpressionApplyError> {
        let res = self.0.eval(&context)?;

        Ok(res.boolean())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn postfix_expr_should_parse1() {
        // arrange
        let input = "$products[price gt 100]";

        // act
        let (next_input, res) = postfix_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn predicate_should_parse1() {
        // arrange
        let input = "[price gt 100]";

        // act
        let (next_input, res) = predicate(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
