//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-map-operator>

use std::fmt::Display;

use nom::{character::complete::char, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{recipes::Res, whitespace_recipes::ws},
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::path_expressions::{path_expr, PathExpr};

pub fn simple_map_expr(input: &str) -> Res<&str, SimpleMapExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SimpleMapExpr

    context(
        "simple_map_expr",
        tuple((path_expr, many0(ws((char('!'), path_expr))))),
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

impl SimpleMapExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let mut result = self.expr.eval(context)?;

        // If there are no map items, return the base expression's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // For each map item (E1 ! E2 ! E3 ...):
        // evaluate the RHS for each item in the LHS result,
        // using that item as the context item.
        for map_expr in &self.items {
            let mut next_result = XpathItemSet::new();
            let size = result.len();
            for (i, item) in result.iter().enumerate() {
                let inner_context =
                    context.new_with_item_and_size(item.clone(), i + 1, size, false);
                let inner_result = map_expr.eval(&inner_context)?;
                next_result.extend(inner_result);
            }
            result = next_result;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_map_expr_should_parse() {
        // arrange
        let input = "a!b!c";

        // act
        let (next_input, res) = simple_map_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "a!b!c");
    }

    #[test]
    fn simple_map_expr_should_parse_whitespace() {
        // arrange
        let input = "a ! b ! c";

        // act
        let (next_input, res) = simple_map_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "a!b!c");
    }

    #[test]
    fn simple_map_expr_should_parse1() {
        // arrange
        let input = r#"child::div1/child::para/string()!concat("id-", .)"#;

        // act
        let (next_input, res) = simple_map_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
