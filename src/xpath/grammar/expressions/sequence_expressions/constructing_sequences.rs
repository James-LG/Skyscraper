//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#construct_seq>

use std::fmt::Display;

use nom::{bytes::complete::tag, combinator::opt, error::context, sequence::tuple};

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            expressions::arithmetic_expressions::{additive_expr, AdditiveExpr},
            expressions::primary_expressions::static_function_calls::func_data,
            recipes::Res,
            whitespace_recipes::ws,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

pub fn range_expr(input: &str) -> Res<&str, RangeExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-RangeExpr

    context(
        "range_expr",
        tuple((additive_expr, opt(ws((tag("to"), additive_expr))))),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            RangeExpr {
                expr: Box::new(res.0),
                to_expr: res.1.map(|res| Box::new(res.1)),
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct RangeExpr {
    pub expr: Box<AdditiveExpr>,
    pub to_expr: Option<Box<AdditiveExpr>>,
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

impl RangeExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's no `to` clause, return the base expression's eval.
        let to_expr = match &self.to_expr {
            Some(expr) => expr,
            None => return Ok(result),
        };

        // Atomize both operands to get integer values.
        let start = Self::atomize_to_integer(&result, context, "start")?;
        let to_result = to_expr.eval(context)?;
        let end = Self::atomize_to_integer(&to_result, context, "end")?;

        // If start > end, the result is an empty sequence (per XPath spec).
        if start > end {
            return Ok(XpathItemSet::new());
        }

        let mut items = XpathItemSet::new();
        for i in start..=end {
            items.extend(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(i)
            )]);
        }
        Ok(items)
    }

    fn atomize_to_integer<'tree>(
        result: &XpathItemSet<'tree>,
        context: &XpathExpressionContext<'tree>,
        operand_name: &str,
    ) -> Result<i64, ExpressionApplyError> {
        let atomized = func_data(result, context.item_tree)?;
        if atomized.len() != 1 {
            return Err(ExpressionApplyError {
                msg: format!(
                    "range: {} operand must be a single value, got {}",
                    operand_name,
                    atomized.len()
                ),
            });
        }
        match &atomized[0] {
            AnyAtomicType::Integer(n) => Ok(*n),
            other => Err(ExpressionApplyError {
                msg: format!(
                    "range: {} operand must be an integer, got {:?}",
                    operand_name, other
                ),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn range_expr_should_parse() {
        // arrange
        let input = "1 to 4";

        // act
        let (next_input, res) = range_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "1 to 4");
    }
}
