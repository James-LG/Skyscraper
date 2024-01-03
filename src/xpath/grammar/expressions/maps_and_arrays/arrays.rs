//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-arrays

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    multi::many0,
};

use crate::xpath::grammar::{
    expressions::{
        expr_single,
        primary_expressions::enclosed_expressions::{enclosed_expr, EnclosedExpr},
        ExprSingle,
    },
    recipes::Res,
    whitespace_recipes::ws,
};

pub fn array_constructor(input: &str) -> Res<&str, ArrayConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ArrayConstructor

    fn square_array_constructor_map(input: &str) -> Res<&str, ArrayConstructor> {
        square_array_constructor(input)
            .map(|(next_input, res)| (next_input, ArrayConstructor::SquareArrayConstructor(res)))
    }

    fn curly_array_constructor_map(input: &str) -> Res<&str, ArrayConstructor> {
        curly_array_constructor(input)
            .map(|(next_input, res)| (next_input, ArrayConstructor::CurlyArrayConstructor(res)))
    }

    context(
        "array_constructor",
        alt((square_array_constructor_map, curly_array_constructor_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ArrayConstructor {
    SquareArrayConstructor(SquareArrayConstructor),
    CurlyArrayConstructor(CurlyArrayConstructor),
}

impl Display for ArrayConstructor {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt ArrayConstructor")
    }
}

fn square_array_constructor(input: &str) -> Res<&str, SquareArrayConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SquareArrayConstructor
    context(
        "square_array_constructor",
        ws((
            char('['),
            opt(ws((expr_single, many0(ws((char(','), expr_single)))))),
            char(']'),
        )),
    )(input)
    .map(|(next_input, res)| {
        let mut entries = Vec::new();
        if let Some(res) = res.1 {
            entries.push(res.0);
            let extras = res.1.into_iter().map(|res| res.1);
            entries.extend(extras);
        }
        (next_input, SquareArrayConstructor { entries })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SquareArrayConstructor {
    pub entries: Vec<ExprSingle>,
}

impl Display for SquareArrayConstructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", entry)?;
        }
        write!(f, "]")
    }
}

fn curly_array_constructor(input: &str) -> Res<&str, CurlyArrayConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-CurlyArrayConstructor
    context("curly_array_constructor", ws((tag("array"), enclosed_expr)))(input)
        .map(|(next_input, res)| (next_input, CurlyArrayConstructor(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct CurlyArrayConstructor(EnclosedExpr);

impl Display for CurlyArrayConstructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "array {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_array_constructor_should_parse_lots_of_whitespace() {
        // arrange
        let input = "[ 1, 2, 5, 7 ]";

        // act
        let (next_input, res) = square_array_constructor(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "[1, 2, 5, 7]");
    }

    #[test]
    fn square_array_constructor_should_parse_no_whitespace() {
        // arrange
        let input = "[1,2,5,7]";

        // act
        let (next_input, res) = square_array_constructor(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "[1, 2, 5, 7]");
    }

    #[test]
    fn curly_array_constructor_should_parse_lots_of_whitespace() {
        // arrange
        let input = "array { $x }";

        // act
        let (next_input, res) = curly_array_constructor(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "array { $x }");
    }

    #[test]
    fn curly_array_constructor_should_parse_no_whitespace() {
        // arrange
        let input = "array{$x}";

        // act
        let (next_input, res) = curly_array_constructor(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "array { $x }");
    }
}
