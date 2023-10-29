//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-element-test

use std::fmt::Display;

use crate::xpath::grammar::{recipes::Res, types::common::element_name};

use super::common::{type_name, ElementName, TypeName};

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, sequence::tuple,
};

pub fn element_test(input: &str) -> Res<&str, ElementTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ElementTest

    tuple((
        tag("element"),
        char('('),
        opt(tuple((
            element_name_or_wildcard,
            opt(tuple((char(','), type_name, opt(char('?'))))),
        ))),
        char(')'),
    ))(input)
    .map(|(next_input, res)| {
        let res = res.2.map(|tup| (tup.0, tup.1.map(|tup2| tup2.1)));
        let element_test = match res {
            Some((name_or_wildcard, Some(type_name))) => ElementTest {
                name_or_wildcard: Some(name_or_wildcard),
                type_name: Some(type_name),
            },
            Some((name_or_wildcard, None)) => ElementTest {
                name_or_wildcard: Some(name_or_wildcard),
                type_name: None,
            },
            None => ElementTest {
                name_or_wildcard: None,
                type_name: None,
            },
        };
        (next_input, element_test)
    })
}

#[derive(PartialEq, Debug)]
pub struct ElementTest {
    pub name_or_wildcard: Option<ElementNameOrWildcard>,
    pub type_name: Option<TypeName>,
}

impl Display for ElementTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt ElementTest")
    }
}

pub fn element_name_or_wildcard(input: &str) -> Res<&str, ElementNameOrWildcard> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ElementNameOrWildcard

    fn element_name_map(input: &str) -> Res<&str, ElementNameOrWildcard> {
        element_name(input)
            .map(|(next_input, res)| (next_input, ElementNameOrWildcard::ElementName(res)))
    }

    fn wildcard_map(input: &str) -> Res<&str, ElementNameOrWildcard> {
        char('*')(input).map(|(next_input, _res)| (next_input, ElementNameOrWildcard::Wildcard))
    }

    alt((element_name_map, wildcard_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum ElementNameOrWildcard {
    ElementName(ElementName),
    Wildcard,
}
