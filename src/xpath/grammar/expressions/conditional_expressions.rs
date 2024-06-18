//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-conditionals>

use std::fmt::Display;

use nom::{
    bytes::complete::tag,
    character::complete::{char, multispace0},
    error::context,
    sequence::tuple,
};

use crate::xpath::grammar::{
    recipes::Res,
    whitespace_recipes::{sep, ws},
};

use super::{expr, expr_single, Expr, ExprSingle};

pub fn if_expr(input: &str) -> Res<&str, IfExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-IfExpr

    context(
        "if_expr",
        tuple((
            ws((tag("if"), char('('), expr, char(')'))),
            multispace0,
            sep((tag("then"), expr_single, tag("else"), expr_single)),
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            IfExpr {
                condition: res.0 .2,
                then: res.2 .1,
                else_expr: res.2 .3,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct IfExpr {
    pub condition: Expr,
    pub then: ExprSingle,
    pub else_expr: ExprSingle,
}

impl Display for IfExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "if ({})", self.condition)?;
        writeln!(f, "  then {}", self.then)?;
        writeln!(f, "  else {}", self.else_expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn if_expr_should_parse() {
        // arrange
        let input = r#"if($widget1/unit-cost<$widget2/unit-cost)then $widget1 else $widget2"#;

        // act
        let (next_input, res) = if_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            indoc::indoc!(
                r#"
                if ($widget1/unit-cost<$widget2/unit-cost)
                  then $widget1
                  else $widget2
                "#
            )
        );
    }

    #[test]
    fn if_expr_should_parse_whitespace() {
        // arrange
        let input = r#"if ($widget1/unit-cost < $widget2/unit-cost)
              then $widget1
              else $widget2"#;

        // act
        let (next_input, res) = if_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            indoc::indoc!(
                r#"
                if ($widget1/unit-cost<$widget2/unit-cost)
                  then $widget1
                  else $widget2
                "#
            )
        );
    }
}
