//! Parse and apply XPath expressions to HTML documents.

use indextree::{Arena, NodeId};
use nom::error::VerboseError;
use thiserror::Error;

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
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
    searchable_nodes: Vec<Node<'tree>>,
}

pub enum XPathResult<'tree> {
    ItemSet(Vec<XpathItem<'tree>>),
    Item(XpathItem<'tree>),
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
                    AnyAtomicType::Number(n) => n != 0,
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
