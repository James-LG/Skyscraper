//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-paren-expressions>

use std::fmt::Display;

use nom::{character::complete::char, combinator::opt, error::context};

use crate::xpath::{
    grammar::{
        expressions::{expr, Expr},
        recipes::Res,
        whitespace_recipes::ws,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

pub fn parenthesized_expr(input: &str) -> Res<&str, ParenthesizedExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ParenthesizedExpr
    context("parenthesized_expr", ws((char('('), opt(expr), char(')'))))(input)
        .map(|(next_input, res)| (next_input, ParenthesizedExpr(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct ParenthesizedExpr(pub Option<Expr>);

impl Display for ParenthesizedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        if let Some(x) = &self.0 {
            write!(f, "{}", x)?;
        }
        write!(f, ")")
    }
}

impl ParenthesizedExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        if let Some(expr) = &self.0 {
            expr.eval(context)
        } else {
            Ok(XpathItemSet::new())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parenthesized_expr_should_parse() {
        // arrange
        let input = "(chapter|appendix)";

        // act
        let (next_input, res) = parenthesized_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn parenthesized_expr_should_parse_whitespace() {
        // arrange
        let input = "( chapter | appendix )";

        // act
        let (next_input, res) = parenthesized_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "(chapter|appendix)");
    }
}
