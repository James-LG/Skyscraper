//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-attribute-test>

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
};

use crate::xpath::{
    grammar::{
        recipes::Res, types::common::attribute_name, whitespace_recipes::ws, XpathItemTreeNode,
    },
    ExpressionApplyError,
};

use super::common::{type_name, AttributeName, TypeName};

pub fn attribute_test(input: &str) -> Res<&str, AttributeTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AttributeTest

    context(
        "attribute_test",
        ws((
            tag("attribute"),
            char('('),
            opt(ws((
                attrib_name_or_wildcard,
                opt(ws((char(','), type_name, opt(char('?'))))),
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "attribute(")?;
        if let Some(pair) = &self.pair {
            write!(f, "{}", pair)?;
        }
        write!(f, ")")
    }
}

impl AttributeTest {
    pub(crate) fn is_match<'tree>(
        &self,
        node: &'tree XpathItemTreeNode,
    ) -> Result<bool, ExpressionApplyError> {
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

impl Display for AttributeTestPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name_or_wildcard)?;
        if let Some(type_name) = &self.type_name {
            write!(f, ", {}", type_name)?;
        }
        Ok(())
    }
}

impl AttributeTestPair {
    pub(crate) fn is_match<'tree>(
        &self,
        node: &'tree XpathItemTreeNode,
    ) -> Result<bool, ExpressionApplyError> {
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

impl Display for AttribNameOrWildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttribNameOrWildcard::AttributeName(x) => write!(f, "{}", x),
            AttribNameOrWildcard::Wildcard => write!(f, "*"),
        }
    }
}

impl AttribNameOrWildcard {
    pub(crate) fn is_match<'tree>(
        &self,
        _node: &'tree XpathItemTreeNode,
    ) -> Result<bool, ExpressionApplyError> {
        match self {
            AttribNameOrWildcard::AttributeName(_) => {
                todo!("AttribNameOrWildcard::eval attribute_name")
            }
            AttribNameOrWildcard::Wildcard => Ok(true),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn attribute_test_should_parse() {
        // arrange
        let input = "attribute()";

        // act
        let (next_input, res) = attribute_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "attribute()");
    }

    #[test]
    fn attribute_test_should_parse_whitespace() {
        // arrange
        let input = "attribute ( )";

        // act
        let (next_input, res) = attribute_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "attribute()");
    }

    #[test]
    fn attribute_test_should_parse_any() {
        // arrange
        let input = "attribute(*)";

        // act
        let (next_input, res) = attribute_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "attribute(*)");
    }

    #[test]
    fn attribute_test_should_parse_any_whitespace() {
        // arrange
        let input = "attribute ( * )";

        // act
        let (next_input, res) = attribute_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "attribute(*)");
    }

    #[test]
    fn attribute_test_should_parse_attrib_name() {
        // arrange
        let input = "attribute(price,currency)";

        // act
        let (next_input, res) = attribute_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "attribute(price, currency)");
    }

    #[test]
    fn attribute_test_should_parse_attrib_name_whitespace() {
        // arrange
        let input = "attribute ( price, currency )";

        // act
        let (next_input, res) = attribute_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "attribute(price, currency)");
    }
}
