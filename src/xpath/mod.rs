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

struct XPathExpressionContext<'tree> {
    item_tree: &'tree XpathItemTree,
    searchable_nodes: Vec<XpathItemTreeNode<'tree>>,
}

pub enum XPathResult<'tree> {
    ItemSet(Vec<XpathItem<'tree>>),
    Item(XpathItem<'tree>),
}

impl<'tree> XPathResult<'tree> {
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
