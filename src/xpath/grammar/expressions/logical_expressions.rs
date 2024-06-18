//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-logical-expressions>

use std::fmt::Display;

use nom::{bytes::complete::tag, error::context};

use crate::xpath::{
    grammar::{
        recipes::Res,
        whitespace_recipes::{sep, sep_many0},
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::comparison_expressions::{comparison_expr, ComparisonExpr};

pub fn or_expr(input: &str) -> Res<&str, OrExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-OrExpr

    context("or_expr", sep_many0(and_expr, sep((tag("or"), and_expr))))(input).map(
        |(next_input, res)| {
            let items = res.1.into_iter().map(|res| res.1).collect();
            (next_input, OrExpr { expr: res.0, items })
        },
    )
}

#[derive(PartialEq, Debug, Clone)]
pub struct OrExpr {
    pub expr: AndExpr,
    pub items: Vec<AndExpr>,
}

impl Display for OrExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " or {}", x)?;
        }

        Ok(())
    }
}

impl OrExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Otherwise, do the boolean op.
        todo!("OrExpr::eval or operator")
    }
}

fn and_expr(input: &str) -> Res<&str, AndExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AndExpr

    context(
        "and_expr",
        sep_many0(comparison_expr, sep((tag("and"), comparison_expr))),
    )(input)
    .map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, AndExpr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct AndExpr {
    pub expr: ComparisonExpr,
    pub items: Vec<ComparisonExpr>,
}

impl Display for AndExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " and {}", x)?;
        }

        Ok(())
    }
}

impl AndExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Otherwise, do the boolean op.
        todo!("AndExpr::eval and operator")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn or_expr_should_parse() {
        // arrange
        let input = "a or b or c";

        // act
        let (next_input, res) = or_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "a or b or c");
    }

    #[test]
    fn or_expr_should_not_match_second_with_no_separator() {
        // arrange
        let input = "a or bor c";

        // act
        let (next_input, res) = or_expr(input).unwrap();

        // assert
        assert_eq!(next_input, " c");
        assert_eq!(res.to_string(), "a or bor");
    }

    #[test]
    fn and_expr_expr_should_parse() {
        // arrange
        let input = "a and b and c";

        // act
        let (next_input, res) = and_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "a and b and c");
    }

    #[test]
    fn and_expr_expr_should_not_match_second_with_no_separator() {
        // arrange
        let input = "a and band c";

        // act
        let (next_input, res) = and_expr(input).unwrap();

        // assert
        assert_eq!(next_input, " c");
        assert_eq!(res.to_string(), "a and band");
    }
}
