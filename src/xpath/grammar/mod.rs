// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md
// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar

mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod xml_names;

pub use expressions::{xpath, XPath};
use indextree::{Arena, NodeId};
use thiserror::Error;

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{AttributeNode, ElementNode, Node, TextNode},
};

use self::data_model::{AnyAtomicType, CommentNode, Function, NamespaceNode, PINode, XpathItem};

use super::DocumentNodeSet;

#[derive(PartialEq, Debug, Error)]
#[error("Error applying expression {msg}")]
pub struct ExpressionApplyError {
    msg: String,
}

trait Expression {
    fn apply<'tree>(
        &self,
        item_tree: &'tree XpathItemTree,
        searchable_nodes: &Vec<XpathItemTreeNode<'tree>>,
    ) -> Result<Vec<XpathItemTreeNode<'tree>>, ExpressionApplyError>;
}

pub struct ExpressionResultItem<'tree> {
    tree_nodes: Vec<XpathItemTreeNode<'tree>>,
    non_tree_items: Vec<NonTreeXpathItem>,
}

/// Subset of [Node] that are not allowed to be children of other nodes.
/// Should be disjoint with [XpathItemTreeNodeData].
pub enum NonTreeXpathNode {
    DocumentNode(DocumentNode),
    AttributeNode(AttributeNode),
    NamespaceNode(NamespaceNode),
}

/// Subset of [XpathItem] that are not allowed to be in the tree.
pub enum NonTreeXpathItem {
    Node(NonTreeXpathNode),
    Function(Function),
    AnyAtomicType(AnyAtomicType),
}

/// Subset of [Node] that are allowed to be children of other nodes.
/// Should be disjoint with [NonTreeXpathNode].
#[derive(PartialEq, Debug)]
pub enum XpathItemTreeNodeData {
    ElementNode(ElementNode),
    PINode(PINode),
    CommentNode(CommentNode),
    TextNode(TextNode),
}

impl From<XpathItemTreeNodeData> for Node {
    fn from(value: XpathItemTreeNodeData) -> Self {
        match value {
            XpathItemTreeNodeData::ElementNode(node) => Node::ElementNode(node),
            XpathItemTreeNodeData::PINode(node) => Node::PINode(node),
            XpathItemTreeNodeData::CommentNode(node) => Node::CommentNode(node),
            XpathItemTreeNodeData::TextNode(node) => Node::TextNode(node),
        }
    }
}

pub struct XpathItemTreeNode<'a> {
    id: NodeId,
    data: &'a XpathItemTreeNodeData,
}

impl<'a> XpathItemTreeNode<'a> {
    pub fn children(&self, tree: &'a XpathItemTree) -> impl Iterator<Item = XpathItemTreeNode<'a>> {
        self.id
            .children(&tree.arena)
            .into_iter()
            .map(move |id| tree.get(id))
    }
}

pub struct XpathItemTree {
    arena: Arena<XpathItemTreeNodeData>,
    /// The root node of the document.
    root_node: NodeId,
}

impl XpathItemTree {
    fn get(&self, id: NodeId) -> XpathItemTreeNode<'_> {
        let indextree_node = self
            .arena
            .get(id)
            .expect("xpath item node missing from tree");

        let data = indextree_node.get();
        XpathItemTreeNode { id, data }
    }

    pub fn root(&self) -> XpathItemTreeNode<'_> {
        self.get(self.root_node)
    }
}

impl XpathItemTree {
    fn from_html_document(html_document: &HtmlDocument) -> Self {
        fn internal_from_html_document(
            current_html_node: &DocumentNode,
            html_document: &HtmlDocument,
            item_arena: &mut Arena<XpathItemTreeNodeData>,
        ) -> NodeId {
            let html_node = html_document
                .get_html_node(&current_html_node)
                .expect("html document missing expected node");
            let root_item = match html_node {
                HtmlNode::Tag(tag) => {
                    let attributes = tag
                        .attributes
                        .iter()
                        .map(|a| AttributeNode {
                            name: a.0.to_string(),
                            value: a.1.to_string(),
                        })
                        .collect();
                    XpathItemTreeNodeData::ElementNode(ElementNode {
                        name: tag.name.to_string(),
                        attributes,
                    })
                }
                HtmlNode::Text(text) => XpathItemTreeNodeData::TextNode(TextNode {
                    content: text.to_string(),
                }),
            };

            let root_item_id = item_arena.new_node(root_item);

            for child in current_html_node.children(&html_document) {
                let child_node = internal_from_html_document(&child, html_document, item_arena);
                root_item_id.append(child_node, item_arena);
            }

            root_item_id
        }

        let mut item_arena = Arena::<XpathItemTreeNodeData>::new();
        let root_node_id =
            internal_from_html_document(&html_document.root_node, &html_document, &mut item_arena);

        XpathItemTree {
            arena: item_arena,
            root_node: root_node_id,
        }
    }
}
