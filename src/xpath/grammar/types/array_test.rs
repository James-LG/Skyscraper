use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, character::complete::char, error::context};

use crate::xpath::grammar::{recipes::Res, whitespace_recipes::ws};

use super::sequence_type::{sequence_type, SequenceType};

pub fn array_test(input: &str) -> Res<&str, ArrayTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ArrayTest

    fn any_array_test_(input: &str) -> Res<&str, ArrayTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AnyArrayTest

        ws((tag("array"), char('('), char('*'), char(')')))(input)
            .map(|(next_input, _res)| (next_input, ArrayTest::AnyArrayTest))
    }

    fn typed_array_test_map(input: &str) -> Res<&str, ArrayTest> {
        typed_array_test(input)
            .map(|(next_input, res)| (next_input, ArrayTest::TypedArrayTest(res)))
    }

    context("array_test", alt((any_array_test_, typed_array_test_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(TypedArrayTest),
}

impl Display for ArrayTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "array(")?;
        match self {
            ArrayTest::AnyArrayTest => write!(f, "*")?,
            ArrayTest::TypedArrayTest(x) => write!(f, "{}", x)?,
        };
        write!(f, ")")
    }
}

fn typed_array_test(input: &str) -> Res<&str, TypedArrayTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-TypedArrayTest

    context(
        "typed_array_test",
        ws((tag("array"), char('('), sequence_type, char(')'))),
    )(input)
    .map(|(next_input, res)| (next_input, TypedArrayTest(res.2)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct TypedArrayTest(pub SequenceType);

impl Display for TypedArrayTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn array_test_should_parse() {
        // arrange
        let input = "array(xs:string)";

        // act
        let (next_input, res) = array_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "array(xs:string)");
    }

    #[test]
    fn array_test_should_parse_whitespace() {
        // arrange
        let input = "array ( xs:string )";

        // act
        let (next_input, res) = array_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "array(xs:string)");
    }

    #[test]
    fn array_test_should_parse_any() {
        // arrange
        let input = "array(*)";

        // act
        let (next_input, res) = array_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "array(*)");
    }

    #[test]
    fn array_test_should_parse_any_whitespace() {
        // arrange
        let input = "array ( * )";

        // act
        let (next_input, res) = array_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "array(*)");
    }
}
