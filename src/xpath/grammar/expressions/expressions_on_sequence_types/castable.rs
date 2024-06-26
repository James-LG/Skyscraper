//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-castable>

use std::fmt::Display;

use nom::{bytes::complete::tag, combinator::opt, error::context};

use crate::xpath::{
    grammar::{recipes::Res, whitespace_recipes::sep},
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::cast::{cast_expr, single_type, CastExpr, SingleType};

pub fn castable_expr(input: &str) -> Res<&str, CastableExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-CastableExpr

    context(
        "castable_expr",
        sep((
            cast_expr,
            opt(sep((tag("castable"), tag("as"), single_type))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let cast_type = res.1.map(|res| res.2);
        (
            next_input,
            CastableExpr {
                expr: res.0,
                cast_type,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct CastableExpr {
    pub expr: CastExpr,
    pub cast_type: Option<SingleType>,
}

impl Display for CastableExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(x) = &self.cast_type {
            write!(f, " castable as {}", x)?;
        }

        Ok(())
    }
}

impl CastableExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.cast_type.is_none() {
            return Ok(result);
        }

        // Otherwise, do the operation.
        todo!("CastableExpr::eval treat operator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castable_expr_should_parse() {
        // arrange
        let input = "fn:root() castable as integer";

        // act
        let (next_input, res) = castable_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
