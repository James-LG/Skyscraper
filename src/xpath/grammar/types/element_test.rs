//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-element-test

use std::fmt::Display;

use crate::xpath::grammar::{recipes::Res, types::common::element_name, whitespace_recipes::ws};

use super::common::{type_name, ElementName, TypeName};

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
};

pub fn element_test(input: &str) -> Res<&str, ElementTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ElementTest

    context(
        "element_test",
        ws((
            tag("element"),
            char('('),
            opt(ws((
                element_name_or_wildcard,
                opt(ws((char(','), type_name, opt(char('?'))))),
            ))),
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| {
        let item = res
            .2
            .map(|(element_name_or_wildcard, type_name)| ElementTestItem {
                element_name_or_wildcard,
                type_name: type_name.map(|tup| tup.1),
            });
        (next_input, ElementTest { item })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct ElementTest {
    pub item: Option<ElementTestItem>,
}

impl Display for ElementTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "element(")?;
        if let Some(item) = &self.item {
            write!(f, "{}", item)?;
        }
        write!(f, ")")
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ElementTestItem {
    pub element_name_or_wildcard: ElementNameOrWildcard,
    pub type_name: Option<TypeName>,
}

impl Display for ElementTestItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.element_name_or_wildcard)?;
        if let Some(type_name) = &self.type_name {
            write!(f, ", {}", type_name)?;
        }
        Ok(())
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

    context(
        "element_name_or_wildcard",
        alt((element_name_map, wildcard_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ElementNameOrWildcard {
    ElementName(ElementName),
    Wildcard,
}

impl Display for ElementNameOrWildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementNameOrWildcard::ElementName(x) => write!(f, "{}", x),
            ElementNameOrWildcard::Wildcard => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn element_test_should_parse() {
        // arrange
        let input = "element()";

        // act
        let (next_input, res) = element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "element()");
    }

    #[test]
    fn element_test_should_parse_whitespace() {
        // arrange
        let input = "element ( )";

        // act
        let (next_input, res) = element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "element()");
    }

    #[test]
    fn element_test_should_parse_any() {
        // arrange
        let input = "element(*)";

        // act
        let (next_input, res) = element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "element(*)");
    }

    #[test]
    fn element_test_should_parse_any_whitespace() {
        // arrange
        let input = "element ( * )";

        // act
        let (next_input, res) = element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "element(*)");
    }

    #[test]
    fn element_test_should_parse_attrib_name() {
        // arrange
        let input = "element(price,currency)";

        // act
        let (next_input, res) = element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "element(price, currency)");
    }

    #[test]
    fn element_test_should_parse_attrib_name_whitespace() {
        // arrange
        let input = "element ( price, currency )";

        // act
        let (next_input, res) = element_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "element(price, currency)");
    }
}
