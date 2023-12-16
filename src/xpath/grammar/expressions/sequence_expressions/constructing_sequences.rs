//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#construct_seq

use std::fmt::Display;

use nom::{bytes::complete::tag, combinator::opt, error::context, sequence::tuple};

use crate::xpath::{
    grammar::{
        expressions::arithmetic_expressions::{additive_expr, AdditiveExpr},
        recipes::Res,
    },
    Expression, ExpressionApplyError, XPathExpressionContext, XPathResult,
};

pub fn range_expr(input: &str) -> Res<&str, RangeExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-RangeExpr

    context(
        "range_expr",
        tuple((additive_expr, opt(tuple((tag("to"), additive_expr))))),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            RangeExpr {
                expr: res.0,
                to_expr: res.1.map(|res| res.1),
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct RangeExpr {
    pub expr: AdditiveExpr,
    pub to_expr: Option<AdditiveExpr>,
}

impl Display for RangeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(x) = &self.to_expr {
            write!(f, " to {}", x)?;
        }

        Ok(())
    }
}

impl Expression for RangeExpr {
    fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.to_expr.is_none() {
            return Ok(result);
        }

        // Otherwise, do the operation.
        todo!("RangeExpr::eval range operator")
    }
}
