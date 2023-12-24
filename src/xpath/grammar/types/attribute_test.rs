//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-attribute-test

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    sequence::tuple,
};

use crate::xpath::{
    grammar::{data_model::Node, recipes::Res, types::common::attribute_name},
    ExpressionApplyError,
};

use super::common::{type_name, AttributeName, TypeName};

pub fn attribute_test(input: &str) -> Res<&str, AttributeTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AttributeTest

    context(
        "attribute_test",
        tuple((
            tag("attribute"),
            char('('),
            opt(tuple((
                attrib_name_or_wildcard,
                opt(tuple((char(','), type_name, opt(char('?'))))),
            ))),
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| {
        let res = res.2.map(|tup| (tup.0, tup.1.map(|tup2| tup2.1)));
        let element_test = AttributeTest {
            pair: res.map(|tup| AttributeTestPair {
                name_or_wildcard: tup.0,
                type_name: tup.1,
            }),
        };
        (next_input, element_test)
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct AttributeTest {
    pub pair: Option<AttributeTestPair>,
}

impl Display for AttributeTest {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt AttributeTest")
    }
}

impl AttributeTest {
    pub(crate) fn is_match<'tree>(&self, node: &Node<'tree>) -> Result<bool, ExpressionApplyError> {
        match &self.pair {
            Some(pair) => pair.is_match(node),
            // No value is equivalent to a wildcard.
            None => {
                let pair = AttributeTestPair {
                    name_or_wildcard: AttribNameOrWildcard::Wildcard,
                    type_name: None,
                };
                pair.is_match(node)
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct AttributeTestPair {
    pub name_or_wildcard: AttribNameOrWildcard,
    pub type_name: Option<TypeName>,
}

impl AttributeTestPair {
    pub(crate) fn is_match<'tree>(&self, node: &Node<'tree>) -> Result<bool, ExpressionApplyError> {
        let is_match = self.name_or_wildcard.is_match(node)?;

        if let Some(_type_name) = &self.type_name {
            todo!("AttributeTestPair::eval type_name")
        }

        Ok(is_match)
    }
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

    context(
        "attrib_name_or_wildcard",
        alt((attribute_name_map, wildcard_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum AttribNameOrWildcard {
    AttributeName(AttributeName),
    Wildcard,
}

impl AttribNameOrWildcard {
    pub(crate) fn is_match<'tree>(
        &self,
        _node: &Node<'tree>,
    ) -> Result<bool, ExpressionApplyError> {
        match self {
            AttribNameOrWildcard::AttributeName(_) => {
                todo!("AttribNameOrWildcard::eval attribute_name")
            }
            AttribNameOrWildcard::Wildcard => Ok(true),
        }
    }
}
