//! This module contains the grammar for the XPath language.
//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar

// Helpful links:
// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md

pub mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod whitespace_recipes;
mod xml_names;

use enum_extract_macro::EnumExtract;
pub(crate) use expressions::xpath;
pub use expressions::Xpath;

use indextree::{Arena, NodeId};

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{
        AttributeNode, CommentNode, ElementNode, PINode, TextNode, XpathDocumentNode,
    },
};

/// Nodes that are part of the [`XpathItemTree`].
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, EnumExtract, Clone)]
pub enum XpathItemTreeNodeData {
    /// The root node of the document.
    DocumentNode(XpathDocumentNode),

    /// An element node.
    ///
    /// HTML tags are represented as element nodes.
    ElementNode(ElementNode),

    /// A processing instruction node.
    PINode(PINode),

    /// A comment node.
    CommentNode(CommentNode),

    /// A text node.
    TextNode(TextNode),

    /// An attribute node.
    AttributeNode(AttributeNode),
}

impl XpathItemTreeNodeData {
    /// Get all children of the document.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the document.
    ///
    /// # Returns
    ///
    /// A vector of all children of the document.
    pub fn children<'tree>(&self, tree: &'tree XpathItemTree) -> Vec<&'tree XpathItemTreeNodeData> {
        match self {
            XpathItemTreeNodeData::DocumentNode(node) => node.children(tree),
            XpathItemTreeNodeData::ElementNode(node) => node.children(tree).collect(),
            XpathItemTreeNodeData::PINode(_) => vec![],
            XpathItemTreeNodeData::CommentNode(_) => vec![],
            XpathItemTreeNodeData::TextNode(_) => vec![],
            XpathItemTreeNodeData::AttributeNode(_) => vec![],
        }
    }

    /// Get the parent of the element.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the element.
    ///
    /// # Returns
    ///
    /// The parent of the element if it exists, or `None` if it does not.
    pub fn parent<'tree>(
        &self,
        tree: &'tree XpathItemTree,
    ) -> Option<&'tree XpathItemTreeNodeData> {
        let id = match self {
            XpathItemTreeNodeData::ElementNode(e) => Some(e.id()),
            XpathItemTreeNodeData::TextNode(t) => Some(t.id()),
            XpathItemTreeNodeData::AttributeNode(a) => Some(a.id()),
            _ => None,
        };

        id.and_then(|id| {
            let parent_id = tree.arena.get(id).unwrap().parent()?;
            Some(tree.get(parent_id))
        })
    }

    /// Get all text contained in this element and its descendants.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this element is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this element and its descendants.
    pub fn text_content<'tree>(&self, tree: &'tree XpathItemTree) -> String {
        match self {
            XpathItemTreeNodeData::DocumentNode(node) => node.text_content(tree),
            XpathItemTreeNodeData::ElementNode(node) => node.text_content(tree),
            XpathItemTreeNodeData::PINode(_) => String::from(""),
            XpathItemTreeNodeData::CommentNode(_) => String::from(""),
            XpathItemTreeNodeData::TextNode(node) => node.content.to_string(),
            XpathItemTreeNodeData::AttributeNode(_) => String::from(""),
        }
    }

    /// Text before the first subelement. This is either a string or the value None, if there was no text.
    ///
    /// Use [`ElementNode::text_content`] to get all text _including_ text in descendant nodes.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this element is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this element.
    pub fn text<'tree>(&self, tree: &'tree XpathItemTree) -> Option<String> {
        match self {
            XpathItemTreeNodeData::DocumentNode(node) => node.text(tree),
            XpathItemTreeNodeData::ElementNode(node) => node.text(tree),
            XpathItemTreeNodeData::PINode(_) => None,
            XpathItemTreeNodeData::CommentNode(_) => None,
            XpathItemTreeNodeData::TextNode(node) => Some(node.content.to_string()),
            XpathItemTreeNodeData::AttributeNode(_) => None,
        }
    }
}

/// A tree of [`XpathItemTreeNode`]s.
pub struct XpathItemTree {
    /// The index tree that stores the nodes.
    arena: Arena<XpathItemTreeNodeData>,

    /// The root node of the document.
    root_node: NodeId,
}

impl XpathItemTree {
    fn get_index_node(&self, id: NodeId) -> &indextree::Node<XpathItemTreeNodeData> {
        self.arena
            .get(id)
            .expect("xpath item node missing from tree")
    }

    fn get(&self, id: NodeId) -> &XpathItemTreeNodeData {
        let indextree_node = self.get_index_node(id);

        indextree_node.get()
    }

    fn root(&self) -> &XpathItemTreeNodeData {
        self.get(self.root_node)
    }

    /// Get an iterator over all nodes in the tree.
    pub fn iter(&self) -> impl Iterator<Item = &XpathItemTreeNodeData> {
        self.arena.iter().map(|node| {
            let id = self.arena.get_node_id(node).unwrap();
            self.get(id)
        })
    }
}

impl From<&HtmlDocument> for XpathItemTree {
    fn from(html_document: &HtmlDocument) -> Self {
        fn internal_from(
            current_html_node: &DocumentNode,
            html_document: &HtmlDocument,
            item_arena: &mut Arena<XpathItemTreeNodeData>,
        ) -> NodeId {
            let html_node = html_document
                .get_html_node(&current_html_node)
                .expect("html document missing expected node");

            let root_item_id = match html_node {
                HtmlNode::Tag(tag) => {
                    let node =
                        XpathItemTreeNodeData::ElementNode(ElementNode::new(tag.name.to_string()));

                    let item_id = item_arena.new_node(node);
                    item_arena
                        .get_mut(item_id)
                        .unwrap()
                        .get_mut()
                        .as_element_node_mut()
                        .unwrap()
                        .set_id(item_id);

                    let attributes: Vec<AttributeNode> = tag
                        .attributes
                        .iter()
                        .map(|(name, value)| {
                            AttributeNode::new(name.to_string(), value.to_string())
                        })
                        .collect();

                    for attribute in attributes {
                        let attribute_node = XpathItemTreeNodeData::AttributeNode(attribute);
                        let attribute_id = item_arena.new_node(attribute_node);

                        item_id.append(attribute_id, item_arena);

                        item_arena
                            .get_mut(attribute_id)
                            .unwrap()
                            .get_mut()
                            .as_attribute_node_mut()
                            .unwrap()
                            .set_id(attribute_id);
                    }

                    item_id
                }
                HtmlNode::Text(text) => {
                    let node = XpathItemTreeNodeData::TextNode(TextNode::new(
                        text.value.to_string(),
                        text.only_whitespace,
                    ));

                    let item_id = item_arena.new_node(node);
                    item_arena
                        .get_mut(item_id)
                        .unwrap()
                        .get_mut()
                        .as_text_node_mut()
                        .unwrap()
                        .set_id(item_id);

                    item_id
                }
            };

            for child in current_html_node.children(&html_document) {
                let child_node = internal_from(&child, html_document, item_arena);
                root_item_id.append(child_node, item_arena);
            }

            root_item_id
        }

        let mut item_arena = Arena::<XpathItemTreeNodeData>::new();
        let root_node_id =
            item_arena.new_node(XpathItemTreeNodeData::DocumentNode(XpathDocumentNode {}));
        let first_child = internal_from(&html_document.root_node, &html_document, &mut item_arena);
        root_node_id.append(first_child, &mut item_arena);

        XpathItemTree {
            arena: item_arena,
            root_node: root_node_id,
        }
    }
}
