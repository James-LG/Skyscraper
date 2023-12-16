//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-string-concat-expr

use std::fmt::Display;

use nom::{bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::recipes::Res, Expression, ExpressionApplyError, XPathExpressionContext, XPathResult,
};

use super::sequence_expressions::constructing_sequences::{range_expr, RangeExpr};

pub fn string_concat_expr(input: &str) -> Res<&str, StringConcatExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-StringConcatExpr

    context(
        "string_concat_expr",
        tuple((range_expr, many0(tuple((tag("||"), range_expr))))),
    )(input)
    .map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, StringConcatExpr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct StringConcatExpr {
    pub expr: RangeExpr,
    pub items: Vec<RangeExpr>,
}

impl Display for StringConcatExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " || {}", x)?;
        }

        Ok(())
    }
}

impl Expression for StringConcatExpr {
    fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Otherwise, do the operation.
        todo!("StringConcatExpr::eval concat operator")
    }
}
