//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-string-concat-expr

use std::fmt::Display;

use nom::{bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{recipes::Res, whitespace_recipes::ws},
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::sequence_expressions::constructing_sequences::{range_expr, RangeExpr};

pub fn string_concat_expr(input: &str) -> Res<&str, StringConcatExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-StringConcatExpr

    context(
        "string_concat_expr",
        tuple((range_expr, many0(ws((tag("||"), range_expr))))),
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

impl StringConcatExpr {
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

        // Otherwise, do the operation.
        todo!("StringConcatExpr::eval concat operator")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn string_concat_expr_should_parse() {
        // arrange
        let input = "a||b||c";

        // act
        let (next_input, res) = string_concat_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "a || b || c");
    }

    #[test]
    fn string_concat_expr_should_parse_whitespace() {
        // arrange
        let input = "a || b || c";

        // act
        let (next_input, res) = string_concat_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "a || b || c");
    }
}
