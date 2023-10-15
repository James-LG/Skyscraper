//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-attribute-test

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, sequence::tuple,
};

use crate::xpath::grammar::{recipes::Res, types::common::attribute_name};

use super::common::{type_name, AttributeName, TypeName};

pub fn attribute_test(input: &str) -> Res<&str, AttributeTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AttributeTest

    tuple((
        tag("attribute"),
        char('('),
        opt(tuple((
            attrib_name_or_wildcard,
            opt(tuple((char(','), type_name, opt(char('?'))))),
        ))),
    ))(input)
    .map(|(next_input, res)| {
        let res = res.2.map(|tup| (tup.0, tup.1.map(|tup2| tup2.1)));
        let element_test = match res {
            Some((name_or_wildcard, Some(type_name))) => AttributeTest {
                name_or_wildcard: Some(name_or_wildcard),
                type_name: Some(type_name),
            },
            Some((name_or_wildcard, None)) => AttributeTest {
                name_or_wildcard: Some(name_or_wildcard),
                type_name: None,
            },
            None => AttributeTest {
                name_or_wildcard: None,
                type_name: None,
            },
        };
        (next_input, element_test)
    })
}

#[derive(PartialEq, Debug)]
pub struct AttributeTest {
    pub name_or_wildcard: Option<AttribNameOrWildcard>,
    pub type_name: Option<TypeName>,
}

pub fn attrib_name_or_wildcard(input: &str) -> Res<&str, AttribNameOrWildcard> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AttribNameOrWildcard

    fn attribute_name_map(input: &str) -> Res<&str, AttribNameOrWildcard> {
        attribute_name(input)
            .map(|(next_input, res)| (next_input, AttribNameOrWildcard::AttributeName(res)))
    }

    fn wildcard_map(input: &str) -> Res<&str, AttribNameOrWildcard> {
        char('*')(input).map(|(next_input, _res)| (next_input, AttribNameOrWildcard::Wildcard))
    }

    alt((attribute_name_map, wildcard_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum AttribNameOrWildcard {
    AttributeName(AttributeName),
    Wildcard,
}
