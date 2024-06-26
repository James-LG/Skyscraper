//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-literals>

use std::fmt::Display;

use nom::{branch::alt, error::context};

use crate::xpath::grammar::{
    data_model::AnyAtomicType,
    recipes::Res,
    terminal_symbols::{
        decimal_literal, double_literal, integer_literal, string_literal, StringLiteral,
    },
};

pub fn literal(input: &str) -> Res<&str, Literal> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Literal

    fn numeric_literal_map(input: &str) -> Res<&str, Literal> {
        numeric_literal(input).map(|(next_input, res)| (next_input, Literal::NumericLiteral(res)))
    }

    fn string_literal_map(input: &str) -> Res<&str, Literal> {
        string_literal(input).map(|(next_input, res)| (next_input, Literal::StringLiteral(res)))
    }

    context("literal", alt((numeric_literal_map, string_literal_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    NumericLiteral(NumericLiteral),
    StringLiteral(StringLiteral),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::NumericLiteral(x) => write!(f, "{}", x),
            Literal::StringLiteral(x) => write!(f, "{}", x),
        }
    }
}

impl Literal {
    pub(crate) fn value(&self) -> AnyAtomicType {
        match self {
            Literal::NumericLiteral(numeric) => numeric.value(),
            Literal::StringLiteral(x) => AnyAtomicType::String(x.value.clone()),
        }
    }
}

pub fn numeric_literal(input: &str) -> Res<&str, NumericLiteral> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NumericLiteral

    fn integer_literal_map(input: &str) -> Res<&str, NumericLiteral> {
        integer_literal(input).map(|(next_input, res)| (next_input, NumericLiteral::Integer(res)))
    }

    fn decimal_literal_map(input: &str) -> Res<&str, NumericLiteral> {
        decimal_literal(input).map(|(next_input, res)| (next_input, NumericLiteral::Decimal(res)))
    }

    fn double_literal_map(input: &str) -> Res<&str, NumericLiteral> {
        double_literal(input).map(|(next_input, res)| (next_input, NumericLiteral::Double(res)))
    }

    context(
        "numeric_literal",
        alt((double_literal_map, decimal_literal_map, integer_literal_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum NumericLiteral {
    Integer(u32),
    Decimal(f32),
    Double(f64),
}

impl Display for NumericLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericLiteral::Integer(x) => write!(f, "{}", x),
            NumericLiteral::Decimal(x) => write!(f, "{}", x),
            NumericLiteral::Double(x) => write!(f, "{}", x),
        }
    }
}

impl NumericLiteral {
    pub(crate) fn value(&self) -> AnyAtomicType {
        match self {
            NumericLiteral::Integer(x) => AnyAtomicType::Integer((*x).into()),
            NumericLiteral::Decimal(x) => AnyAtomicType::Float(ordered_float::OrderedFloat(*x)),
            NumericLiteral::Double(x) => AnyAtomicType::Double(ordered_float::OrderedFloat(*x)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn numeric_literal_should_match_decimal() {
        // arrange
        let input = "0.25";

        // act
        let (next_input, res) = numeric_literal(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "0.25");
    }

    #[test]
    fn numeric_literal_should_match_double() {
        // arrange
        let input = "1e+2";

        // act
        let (next_input, res) = numeric_literal(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "100");
    }

    #[test]
    fn numeric_literal_should_match_integer() {
        // arrange
        let input = "1";

        // act
        let (next_input, res) = numeric_literal(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "1");
    }
}
