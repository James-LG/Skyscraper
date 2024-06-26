//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-postfix-expression>

use std::fmt::Display;

use nom::{branch::alt, character::complete::char, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::{AnyAtomicType, XpathItem},
        expressions::{
            common::argument_list, maps_and_arrays::lookup_operator::postfix_lookup::lookup,
            primary_expressions::primary_expr,
        },
        recipes::Res,
        whitespace_recipes::ws,
    },
    ExpressionApplyError, XpathExpressionContext, XpathItemSet,
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
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let res = self.expr.eval(context)?;

        if !self.items.is_empty() {
            todo!("PostfixExpr eval items")
        }

        Ok(res)
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
    context("predicate", ws((char('['), expr, char(']'))))(input)
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
        context: &XpathExpressionContext<'tree>,
    ) -> Result<bool, ExpressionApplyError> {
        let res = self.0.eval(&context)?;

        // The predicate truth value is derived by applying the following rules, in order:
        // 1. If the value of the predicate expression is a singleton atomic value of a numeric type or derived from a numeric type,
        //    the predicate truth value is true if the value of the predicate expression is equal (by the eq operator) to the context position,
        //    and is false otherwise.
        // 2. Otherwise, the predicate truth value is the effective boolean value of the predicate expression.

        // Step 1. If the value is a number, check if it matches the context position.
        if res.len() == 1 {
            match &res[0] {
                XpathItem::AnyAtomicType(atomic_type) => match atomic_type {
                    AnyAtomicType::Integer(n) => return Ok(*n == context.position as i64),
                    AnyAtomicType::Float(n) => return Ok(*n == context.position as f32),
                    AnyAtomicType::Double(n) => return Ok(*n == context.position as f64),
                    _ => {}
                },
                _ => {}
            }
        }

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

    #[test]
    fn predicate_should_parse2() {
        // arrange
        let input = "[2]";

        // act
        let (next_input, res) = predicate(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn predicate_should_parse3() {
        // arrange
        let input = "[ 2 ]";

        // act
        let (next_input, res) = predicate(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "[2]");
    }
}
