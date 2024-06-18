//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-enclosed-expr>

use nom::{character::complete::char, combinator::opt, error::context};

use crate::xpath::grammar::{
    expressions::{expr, Expr},
    recipes::Res,
    whitespace_recipes::ws,
};

pub fn enclosed_expr(input: &str) -> Res<&str, EnclosedExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-EnclosedExpr
    context("enclosed_expr", ws((char('{'), opt(expr), char('}'))))(input)
        .map(|(next_input, res)| (next_input, EnclosedExpr(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct EnclosedExpr(Option<Expr>);

impl std::fmt::Display for EnclosedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(expr) = &self.0 {
            write!(f, "{{ {} }}", expr)
        } else {
            write!(f, "{{ }}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enclosed_expr_should_parse_lots_of_whitespace() {
        // arrange
        let input = "{ $x }";

        // act
        let (next_input, res) = enclosed_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "{ $x }");
    }

    #[test]
    fn enclosed_expr_should_parse_no_whitespace() {
        // arrange
        let input = "{$x}";

        // act
        let (next_input, res) = enclosed_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "{ $x }");
    }
}
