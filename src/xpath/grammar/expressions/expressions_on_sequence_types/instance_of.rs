//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-instance-of>

use std::fmt::Display;

use nom::{bytes::complete::tag, combinator::opt, error::context};

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            recipes::Res,
            types::sequence_type::{sequence_type, SequenceType},
            whitespace_recipes::sep,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

use super::treat::{treat_expr, TreatExpr};

pub fn instanceof_expr(input: &str) -> Res<&str, InstanceofExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-InstanceofExpr

    context(
        "instanceof_expr",
        sep((
            treat_expr,
            opt(sep((tag("instance"), tag("of"), sequence_type))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let instanceof_type = res.1.map(|res| res.2);
        (
            next_input,
            InstanceofExpr {
                expr: res.0,
                instanceof_type,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct InstanceofExpr {
    pub expr: TreatExpr,
    pub instanceof_type: Option<SequenceType>,
}

impl Display for InstanceofExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(x) = &self.instanceof_type {
            write!(f, " instance of {}", x)?;
        }

        Ok(())
    }
}

impl InstanceofExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's no `instance of` clause, return the base expression's eval.
        let seq_type = match &self.instanceof_type {
            Some(t) => t,
            None => return Ok(result),
        };

        // Check if the result matches the sequence type.
        let matches = seq_type.is_match(&result, context.item_tree)?;
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Boolean(matches)
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instanceof_expr_should_parse() {
        // arrange
        let input = "fn:root() instance of integer";

        // act
        let (next_input, res) = instanceof_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
