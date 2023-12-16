//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-map-operator

use std::fmt::Display;

use nom::{character::complete::char, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::recipes::Res, Expression, ExpressionApplyError, XPathExpressionContext, XPathResult,
};

use super::path_expressions::{path_expr, PathExpr};

pub fn simple_map_expr(input: &str) -> Res<&str, SimpleMapExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SimpleMapExpr

    context(
        "simple_map_expr",
        tuple((path_expr, many0(tuple((char('!'), path_expr))))),
    )(input)
    .map(|(next_input, res)| {
        let expr = res.0;
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, SimpleMapExpr { expr, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleMapExpr {
    pub expr: PathExpr,
    pub items: Vec<PathExpr>,
}

impl Display for SimpleMapExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, "!{}", x)?;
        }

        Ok(())
    }
}

impl Expression for SimpleMapExpr {
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
        todo!("SimpleMapExpr::eval operator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_map_expr_should_parse1() {
        // arrange
        let input = r#"child::div1/child::para/string()!concat("id-",.)"#;

        // act
        let (next_input, res) = simple_map_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
