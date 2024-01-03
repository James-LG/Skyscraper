//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-function-test

use std::fmt::Display;

use crate::xpath::grammar::{recipes::Res, whitespace_recipes::ws};

use super::sequence_type::{sequence_type, SequenceType};

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    multi::many0,
};

pub fn function_test(input: &str) -> Res<&str, FunctionTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-FunctionTest

    fn any_function_test(input: &str) -> Res<&str, FunctionTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AnyFunctionTest

        ws((tag("function"), char('('), char('*'), char(')')))(input)
            .map(|(next_input, _res)| (next_input, FunctionTest::AnyFunctionTest))
    }

    fn typed_function_test_map(input: &str) -> Res<&str, FunctionTest> {
        typed_function_test(input)
            .map(|(next_input, res)| (next_input, FunctionTest::TypedFunctionTest(res)))
    }

    context(
        "function_test",
        alt((any_function_test, typed_function_test_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(TypedFunctionTest),
}

impl Display for FunctionTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionTest::AnyFunctionTest => write!(f, "function(*)"),
            FunctionTest::TypedFunctionTest(x) => write!(f, "{}", x),
        }
    }
}

pub fn typed_function_test(input: &str) -> Res<&str, TypedFunctionTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-TypedFunctionTest

    context(
        "typed_function_test",
        ws((
            tag("function"),
            char('('),
            opt(ws((sequence_type, many0(ws((char(','), sequence_type)))))),
            char(')'),
            tag("as"),
            sequence_type,
        )),
    )(input)
    .map(|(next_input, res)| {
        let mut params = Vec::new();
        if let Some(p) = res.2 {
            params.push(p.0);
            for q in p.1 {
                params.push(q.1);
            }
        }
        (
            next_input,
            TypedFunctionTest {
                params,
                ret_val: res.5,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct TypedFunctionTest {
    pub params: Vec<SequenceType>,
    pub ret_val: SequenceType,
}

impl Display for TypedFunctionTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function(")?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        write!(f, ") as {}", self.ret_val)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn function_test_should_parse_any() {
        // arrange
        let input = "function(*)";

        // act
        let (next_input, res) = function_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "function(*)");
    }

    #[test]
    fn function_test_should_parse_any_whitespace() {
        // arrange
        let input = "function ( * )";

        // act
        let (next_input, res) = function_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "function(*)");
    }

    #[test]
    fn function_test_should_parse_typed_name() {
        // arrange
        let input = "function(int,int) as int";

        // act
        let (next_input, res) = function_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "function(int, int) as int");
    }

    #[test]
    fn function_test_should_parse_typed_whitespace() {
        // arrange
        let input = "function ( int, int ) as int";

        // act
        let (next_input, res) = function_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "function(int, int) as int");
    }
}
