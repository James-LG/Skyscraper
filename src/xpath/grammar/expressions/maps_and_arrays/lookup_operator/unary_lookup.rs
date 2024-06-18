//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-unary-lookup>

use std::fmt::Display;

use nom::{branch::alt, character::complete::char, error::context, sequence::tuple};

use crate::xpath::grammar::{
    expressions::primary_expressions::parenthesized_expressions::{
        parenthesized_expr, ParenthesizedExpr,
    },
    recipes::Res,
    terminal_symbols::integer_literal,
    xml_names::nc_name,
};

pub fn unary_lookup(input: &str) -> Res<&str, UnaryLookup> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-UnaryLookup
    context("unary_lookup", tuple((char('?'), key_specifier)))(input)
        .map(|(next_input, res)| (next_input, UnaryLookup(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnaryLookup(pub KeySpecifier);

impl Display for UnaryLookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}", self.0)
    }
}

pub fn key_specifier(input: &str) -> Res<&str, KeySpecifier> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-KeySpecifier

    fn nc_name_map(input: &str) -> Res<&str, KeySpecifier> {
        nc_name(input).map(|(next_input, res)| (next_input, KeySpecifier::Name(res.to_string())))
    }

    fn integer_literal_map(input: &str) -> Res<&str, KeySpecifier> {
        integer_literal(input).map(|(next_input, res)| (next_input, KeySpecifier::Integer(res)))
    }

    fn parenthesized_expr_map(input: &str) -> Res<&str, KeySpecifier> {
        parenthesized_expr(input)
            .map(|(next_input, res)| (next_input, KeySpecifier::ParenthesizedExpr(res)))
    }

    fn wildcard_map(input: &str) -> Res<&str, KeySpecifier> {
        char('*')(input).map(|(next_input, _res)| (next_input, KeySpecifier::Wildcard))
    }

    context(
        "key_specifier",
        alt((
            nc_name_map,
            integer_literal_map,
            parenthesized_expr_map,
            wildcard_map,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum KeySpecifier {
    Name(String),
    Integer(u32),
    ParenthesizedExpr(ParenthesizedExpr),
    Wildcard,
}

impl Display for KeySpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeySpecifier::Name(x) => write!(f, "{}", x),
            KeySpecifier::Integer(x) => write!(f, "{}", x),
            KeySpecifier::ParenthesizedExpr(x) => write!(f, "{}", x),
            KeySpecifier::Wildcard => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unary_lookup_should_parse() {
        // arrange
        let input = "?2";

        // act
        let (next_input, res) = unary_lookup(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
