//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-schema-element-test

use std::fmt::Display;

use crate::xpath::grammar::{recipes::Res, types::common::element_name, whitespace_recipes::ws};

use super::common::ElementName;

use nom::{bytes::complete::tag, character::complete::char, error::context};

pub fn schema_element_test(input: &str) -> Res<&str, SchemaElementTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SchemaElementTest

    context(
        "schema_element_test",
        ws((
            tag("schema-element"),
            char('('),
            element_declaration,
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| (next_input, SchemaElementTest(res.2)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct SchemaElementTest(pub ElementDeclaration);

impl Display for SchemaElementTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "schema-element({})", self.0)
    }
}

fn element_declaration(input: &str) -> Res<&str, ElementDeclaration> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ElementDeclaration

    context("element_declaration", element_name)(input)
        .map(|(next_input, res)| (next_input, ElementDeclaration(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct ElementDeclaration(pub ElementName);

impl Display for ElementDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn schema_element_test_should_parse() {
        // arrange
        let input = "schema-element(customer)";

        // act
        let (next_input, res) = schema_element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "schema-element(customer)");
    }

    #[test]
    fn schema_element_test_should_whitespace() {
        // arrange
        let input = "schema-element ( customer )";

        // act
        let (next_input, res) = schema_element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "schema-element(customer)");
    }
}
