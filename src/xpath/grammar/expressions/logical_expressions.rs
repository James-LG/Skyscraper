//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-logical-expressions

use std::fmt::Display;

use nom::{bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::recipes::Res, xpath_item_set::XpathItemSet, ExpressionApplyError,
    XpathExpressionContext,
};

use super::comparison_expressions::{comparison_expr, ComparisonExpr};

pub fn or_expr(input: &str) -> Res<&str, OrExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-OrExpr

    context(
        "or_expr",
        tuple((and_expr, many0(tuple((tag("or"), and_expr))))),
    )(input)
    .map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, OrExpr { expr: res.0, items })
    })
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
        tuple((comparison_expr, many0(tuple((tag("and"), comparison_expr))))),
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
