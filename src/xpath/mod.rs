//! Parse and apply XPath expressions to HTML documents.

use std::{fmt::Display, vec};

use indextree::{Arena, NodeId};
use nom::error::VerboseError;
use thiserror::Error;

use crate::{
    html::{HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{
        AnyAtomicType, AttributeNode, CommentNode, ElementNode, Function, NamespaceNode, Node,
        PINode, TextNode,
    },
};

use self::grammar::{data_model::XpathItem, xpath, XPath, XpathItemTreeNode};

pub mod grammar;

pub use self::grammar::XpathItemTree;

pub fn parse(input: &str) -> Result<XPath, nom::Err<VerboseError<&str>>> {
    xpath(input).map(|x| x.1)
}

#[derive(PartialEq, Debug, Error)]
#[error("Error applying expression {msg}")]
pub struct ExpressionApplyError {
    msg: String,
}

trait Expression {
    fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError>;
}

pub(crate) struct XPathExpressionContext<'tree> {
    item_tree: &'tree XpathItemTree,
    item: XpathItem<'tree>,
    position: usize,
    size: usize,
}

impl<'tree> XPathExpressionContext<'tree> {
    pub fn new(
        item_tree: &'tree XpathItemTree,
        items: &Vec<XpathItem<'tree>>,
        position: usize,
    ) -> Self {
        Self {
            item_tree,
            item: items[position - 1].clone(), // Position is 1-based
            position: position,
            size: items.len(),
        }
    }

    pub fn new_single(item_tree: &'tree XpathItemTree, item: XpathItem<'tree>) -> Self {
        Self {
            item_tree,
            item,
            position: 1,
            size: 1,
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum XPathResult<'tree> {
    ItemSet(Vec<XpathItem<'tree>>),
    Item(XpathItem<'tree>),
}

impl Display for XPathResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XPathResult::ItemSet(items) => {
                write!(f, "[")?;
                for item in items {
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            XPathResult::Item(item) => write!(f, "{}", item),
        }
    }
}

impl<'tree> XPathResult<'tree> {
    /// Return the effective boolean value of the result.
    ///
    /// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-ebv
    pub fn boolean(self) -> bool {
        match self {
            XPathResult::ItemSet(items) => !items.is_empty(),
            XPathResult::Item(item) => match item {
                XpathItem::Node(_) => true,
                XpathItem::Function(_) => true,
                XpathItem::AnyAtomicType(atomic_type) => match atomic_type {
                    AnyAtomicType::Boolean(b) => b,
                    AnyAtomicType::Integer(n) => n != 0,
                    AnyAtomicType::Float(n) => n != 0.0,
                    AnyAtomicType::Double(n) => n != 0.0,
                    AnyAtomicType::String(s) => !s.is_empty(),
                },
            },
        }
    }

    pub fn unwrap_item(self) -> XpathItem<'tree> {
        match self {
            XPathResult::Item(item) => item,
            _ => panic!("Expected XPathResult::Item"),
        }
    }

    pub fn unwrap_item_set(self) -> Vec<XpathItem<'tree>> {
        match self {
            XPathResult::ItemSet(items) => items,
            _ => panic!("Expected XPathResult::ItemSet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_should_handle_multiple_double_slashes() {
        // arrange
        let xpath_text = r###"//hello//world"###;

        // act
        let xpath = parse(xpath_text).unwrap();

        // assert
        assert_eq!(xpath.to_string(), xpath_text);
    }
}
