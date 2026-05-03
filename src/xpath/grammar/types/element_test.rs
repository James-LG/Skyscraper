//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-element-test>

use std::fmt::Display;

use indexmap::IndexSet;

use crate::{
    html::grammar::HTML_NAMESPACE,
    xpath::{
        grammar::{
            data_model::XpathItem, recipes::Res, types::common::element_name,
            whitespace_recipes::ws, xml_names::QName, XpathItemTreeNode,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError,
    },
};

use super::{
    common::{type_name, ElementName, TypeName},
    EQName,
};

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

impl ElementTest {
    pub(crate) fn filter<'tree>(
        &self,
        item_set: &XpathItemSet<'tree>,
    ) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
        let mut filtered_nodes = IndexSet::new();

        for item in item_set {
            if let XpathItem::Node(node) = item {
                if let XpathItemTreeNode::ElementNode(element) = node {
                    let matches = match &self.item {
                        // element() with no arguments matches any element node.
                        None => true,
                        Some(item) => {
                            let name_matches = match &item.element_name_or_wildcard {
                                ElementNameOrWildcard::Wildcard => true,
                                ElementNameOrWildcard::ElementName(name) => {
                                    match_element_name(
                                        &name.0,
                                        &element.name,
                                        element.namespace.as_deref(),
                                    )?
                                }
                            };
                            // Type names are ignored in a non-schema-aware processor.
                            name_matches
                        }
                    };

                    if matches {
                        filtered_nodes.insert(*node);
                    }
                }
            }
        }

        Ok(filtered_nodes)
    }
}

/// Match an element name from an EQName against a node's local name and namespace.
pub(crate) fn match_element_name(
    expected: &EQName,
    node_name: &str,
    node_ns: Option<&str>,
) -> Result<bool, ExpressionApplyError> {
    match expected {
        EQName::QName(qname) => match qname {
            QName::PrefixedName(p) => {
                let target_ns = super::resolve_prefix(&p.prefix)?;
                let effective_ns = node_ns.unwrap_or(HTML_NAMESPACE);
                Ok(p.local_part == node_name && effective_ns == target_ns)
            }
            QName::UnprefixedName(name) => Ok(name == node_name),
        },
        EQName::UriQualifiedName(uqn) => {
            Ok(uqn.name == node_name && node_ns.is_some_and(|ns| ns == uqn.uri))
        }
    }
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
