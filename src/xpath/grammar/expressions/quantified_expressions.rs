//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-quantified-expressions>

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, error::context, multi::many0,
    sequence::tuple,
};

use crate::xpath::grammar::{
    expressions::{expr_single, primary_expressions::variable_references::var_name},
    recipes::Res,
    terminal_symbols::symbol_separator,
};

use super::{primary_expressions::variable_references::VarName, ExprSingle};

pub fn quantified_expr(input: &str) -> Res<&str, QuantifiedExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-QuantifiedExpr

    fn some_quantifier(input: &str) -> Res<&str, Quantifier> {
        tag("some")(input).map(|(next_input, _res)| (next_input, Quantifier::Some))
    }

    fn every_quantifier(input: &str) -> Res<&str, Quantifier> {
        tag("every")(input).map(|(next_input, _res)| (next_input, Quantifier::Every))
    }

    context(
        "quantified_expr",
        tuple((
            alt((some_quantifier, every_quantifier)),
            symbol_separator,
            char('$'),
            var_name,
            symbol_separator,
            tag("in"),
            symbol_separator,
            expr_single,
            many0(tuple((
                char(','),
                char('$'),
                var_name,
                symbol_separator,
                tag("in"),
                symbol_separator,
                expr_single,
            ))),
            symbol_separator,
            tag("satisfies"),
            symbol_separator,
            expr_single,
        )),
    )(input)
    .map(|(next_input, res)| {
        let extras = res
            .8
            .into_iter()
            .map(|r| QuantifiedExprItem {
                var: r.2,
                expr: r.6,
            })
            .collect();
        (
            next_input,
            QuantifiedExpr {
                quantifier: res.0,
                item: QuantifiedExprItem {
                    var: res.3,
                    expr: res.7,
                },
                extras,
                satisfies: res.12,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct QuantifiedExpr {
    pub quantifier: Quantifier,
    pub item: QuantifiedExprItem,
    pub extras: Vec<QuantifiedExprItem>,
    pub satisfies: ExprSingle,
}

impl Display for QuantifiedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.quantifier, self.item)?;
        for extra in &self.extras {
            write!(f, ", {}", extra)?;
        }
        write!(f, " satisfies {}", self.satisfies)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Quantifier {
    Some,
    Every,
}

impl Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Quantifier::Some => write!(f, "some"),
            Quantifier::Every => write!(f, "every"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct QuantifiedExprItem {
    pub var: VarName,
    pub expr: ExprSingle,
}

impl Display for QuantifiedExprItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${} in {}", self.var, self.expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quantified_expr_should_parse_every() {
        // arrange
        let input = "every $part in /parts/part satisfies $part/@discounted";

        // act
        let (next_input, res) = quantified_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "every $part in /parts/part satisfies $part/@discounted"
        );
    }

    #[test]
    fn quantified_expr_should_parse_some() {
        // arrange
        let input = r#"some $emp in /emps/employee
        satisfies $part/@discounted"#;

        // act
        let (next_input, res) = quantified_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "some $emp in /emps/employee satisfies $part/@discounted"
        );
    }
}
