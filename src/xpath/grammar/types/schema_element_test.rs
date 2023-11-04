//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-schema-element-test

use std::fmt::Display;

use crate::xpath::grammar::{recipes::Res, types::common::element_name};

use super::common::ElementName;

use nom::{bytes::complete::tag, character::complete::char, error::context, sequence::tuple};

pub fn schema_element_test(input: &str) -> Res<&str, SchemaElementTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SchemaElementTest

    context(
        "schema_element_test",
        tuple((
            tag("schema-element"),
            char('('),
            element_declaration,
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| (next_input, SchemaElementTest(res.2)))
}

#[derive(PartialEq, Debug)]
pub struct SchemaElementTest(pub ElementDeclaration);

impl Display for SchemaElementTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt SchemaElementTest")
    }
}

fn element_declaration(input: &str) -> Res<&str, ElementDeclaration> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ElementDeclaration

    context("element_declaration", element_name)(input)
        .map(|(next_input, res)| (next_input, ElementDeclaration(res)))
}

#[derive(PartialEq, Debug)]
pub struct ElementDeclaration(pub ElementName);
