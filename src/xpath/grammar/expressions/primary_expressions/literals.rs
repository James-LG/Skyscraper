//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-literals

use nom::branch::alt;

use crate::xpath::grammar::{
    recipes::Res,
    terminal_symbols::{decimal_literal, double_literal, integer_literal, string_literal},
};

pub fn literal(input: &str) -> Res<&str, Literal> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Literal

    fn numeric_literal_map(input: &str) -> Res<&str, Literal> {
        numeric_literal(input).map(|(next_input, res)| (next_input, Literal::NumericLiteral(res)))
    }

    fn string_literal_map(input: &str) -> Res<&str, Literal> {
        string_literal(input)
            .map(|(next_input, res)| (next_input, Literal::StringLiteral(res.to_string())))
    }

    alt((numeric_literal_map, string_literal_map))(input)
}

pub enum Literal {
    NumericLiteral(NumericLiteral),
    StringLiteral(String),
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

    alt((integer_literal_map, decimal_literal_map, double_literal_map))(input)
}

pub enum NumericLiteral {
    Integer(u32),
    Decimal(f32),
    Double(f64),
}
