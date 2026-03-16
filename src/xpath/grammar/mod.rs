//! This module contains the grammar for the XPath language.
//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar>

// Helpful links:
// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md

pub mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod whitespace_recipes;
mod xml_names;

use std::fmt::Display;

use enum_extract_macro::EnumExtract;
pub(crate) use expressions::xpath;
pub use expressions::Xpath;

use indextree::{Arena, NodeId};

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{
        AttributeNode, CommentNode, DoctypeNode, ElementNode, PINode, TextNode,
        XpathDocumentNode,
    },
};

/// Nodes that are part of the [`XpathItemTree`].
#[derive(PartialEq, Eq, Debug, Hash, EnumExtract, Clone)]
pub enum XpathItemTreeNode {
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

    /// A document type node (e.g. `<!DOCTYPE html>`).
    DoctypeNode(DoctypeNode),
}

impl XpathItemTreeNode {
    /// Get all children of this node.
    ///
    /// Leaf nodes (text, attribute, comment, PI, doctype) return an empty vector.
    pub fn children<'tree>(&self, tree: &'tree XpathItemTree) -> Vec<&'tree XpathItemTreeNode> {
        match self {
            XpathItemTreeNode::DocumentNode(node) => node.children(tree),
            XpathItemTreeNode::ElementNode(node) => node.children(tree).collect(),
            XpathItemTreeNode::PINode(_) => vec![],
            XpathItemTreeNode::CommentNode(_) => vec![],
            XpathItemTreeNode::TextNode(_) => vec![],
            XpathItemTreeNode::AttributeNode(_) => vec![],
            XpathItemTreeNode::DoctypeNode(_) => vec![],
        }
    }

    /// Get an iterator over all descendants of this node.
    pub fn descendants<'tree>(
        &'tree self,
        tree: &'tree XpathItemTree,
    ) -> impl Iterator<Item = &'tree XpathItemTreeNode> + 'tree {
        let start_id = self.node_id().unwrap_or(tree.root_node);
        start_id
            .descendants(&tree.arena)
            .map(|node_id| tree.get(node_id))
    }

    /// Get the [`NodeId`] of this node, if it has one.
    ///
    /// Document nodes do not have a `NodeId` and return `None`.
    pub(crate) fn node_id(&self) -> Option<NodeId> {
        match self {
            XpathItemTreeNode::ElementNode(e) => Some(e.id()),
            XpathItemTreeNode::TextNode(t) => Some(t.id()),
            XpathItemTreeNode::AttributeNode(a) => Some(a.id()),
            XpathItemTreeNode::CommentNode(c) => Some(c.id()),
            XpathItemTreeNode::PINode(p) => Some(p.id()),
            XpathItemTreeNode::DoctypeNode(d) => Some(d.id()),
            XpathItemTreeNode::DocumentNode(_) => None,
        }
    }

    /// Get the parent of this node, or `None` for the document root.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        self.node_id().and_then(|id| {
            let parent_id = tree.arena.get(id).expect("xpath item node missing from tree").parent()?;
            Some(tree.get(parent_id))
        })
    }

    /// Get an iterator over all text contained in this node and its descendants.
    ///
    /// Includes whitespace text nodes.
    /// Text nodes are split by opening and closing tags contained in the current node.
    pub fn itertext(&self, tree: &XpathItemTree) -> TextIter {
        TextIter::new(tree, self)
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
            XpathItemTreeNode::DocumentNode(node) => node.text_content(tree),
            XpathItemTreeNode::ElementNode(node) => node.text_content(tree),
            XpathItemTreeNode::PINode(node) => node.data.clone(),
            XpathItemTreeNode::CommentNode(c) => c.content.clone(),
            XpathItemTreeNode::TextNode(node) => node.content.to_string(),
            XpathItemTreeNode::AttributeNode(_) => String::from(""),
            XpathItemTreeNode::DoctypeNode(_) => String::from(""),
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
            XpathItemTreeNode::DocumentNode(node) => node.text(tree),
            XpathItemTreeNode::ElementNode(node) => node.text(tree),
            XpathItemTreeNode::PINode(node) => Some(node.data.clone()),
            XpathItemTreeNode::CommentNode(_) => None,
            XpathItemTreeNode::TextNode(node) => Some(node.content.to_string()),
            XpathItemTreeNode::AttributeNode(_) => None,
            XpathItemTreeNode::DoctypeNode(_) => None,
        }
    }

    /// Render this node as an HTML string with the given formatting and indentation level.
    pub fn display(
        &self,
        tree: &XpathItemTree,
        formatting: DisplayFormatting,
        indent: usize,
    ) -> String {
        match self {
            XpathItemTreeNode::DocumentNode(node) => node.display(tree, formatting),
            XpathItemTreeNode::ElementNode(node) => node.display(tree, formatting, indent),
            XpathItemTreeNode::PINode(node) => node.to_string(),
            XpathItemTreeNode::CommentNode(node) => node.to_string(),
            XpathItemTreeNode::TextNode(node) => node.display(tree, formatting, indent),
            XpathItemTreeNode::AttributeNode(node) => node.to_string(),
            XpathItemTreeNode::DoctypeNode(node) => node.to_string(),
        }
    }
}

/// Controls how an [`XpathItemTreeNode`] is rendered to an HTML string.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DisplayFormatting {
    /// Pretty-print with indentation; whitespace-only text nodes are trimmed.
    Pretty,
    /// Render only the opening tag of an element, without its children.
    NoChildren,
    /// Raw mode: preserve original whitespace, include all node types,
    /// omit closing tags for void elements.
    Raw,
}

/// HTML void elements that cannot have content or closing tags.
pub(crate) static VOID_ELEMENTS: [&str; 15] = [
    "meta", "link", "img", "input", "br", "hr", "col", "area", "base", "embed", "keygen",
    "param", "source", "track", "wbr",
];

/// An iterator over all text contained in an element and its descendants.
pub struct TextIter {
    texts: Vec<String>,
    pos: usize,
}

impl TextIter {
    pub(crate) fn new(tree: &XpathItemTree, node: &XpathItemTreeNode) -> TextIter {
        let mut texts = Vec::new();
        Self::collect_texts(tree, node, &mut texts);
        TextIter { texts, pos: 0 }
    }

    fn collect_texts(tree: &XpathItemTree, node: &XpathItemTreeNode, out: &mut Vec<String>) {
        for child in node.children(tree) {
            match child {
                XpathItemTreeNode::TextNode(text) => {
                    out.push(text.content.clone());
                }
                XpathItemTreeNode::ElementNode(_) => {
                    Self::collect_texts(tree, child, out);
                }
                _ => {}
            }
        }
    }
}

impl Iterator for TextIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.texts.len() {
            let s = self.texts[self.pos].clone();
            self.pos += 1;
            Some(s)
        } else {
            None
        }
    }
}

/// A tree of [`XpathItemTreeNode`]s.
///
/// This tree can be searched using an [`Xpath`] expression.
///
/// Created by [`html::parse`](crate::html::parse) (preferred) or by converting
/// from an [`HtmlDocument`].
///
/// # Example
///
/// ```rust
/// use skyscraper::html;
///
/// let text = "<html></html>";
///
/// let tree = html::parse(text).unwrap();
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct XpathItemTree {
    /// The index tree that stores the nodes.
    pub(crate) arena: Arena<XpathItemTreeNode>,

    /// The root node of the document.
    pub(crate) root_node: NodeId,

    /// The document's quirks mode, as determined by the DOCTYPE.
    pub(crate) quirks_mode: crate::html::grammar::QuirksMode,
}

impl XpathItemTree {
    pub(crate) fn new(arena: Arena<XpathItemTreeNode>, root_node: NodeId) -> Self {
        Self::new_with_quirks_mode(arena, root_node, crate::html::grammar::QuirksMode::NoQuirks)
    }

    pub(crate) fn new_with_quirks_mode(
        arena: Arena<XpathItemTreeNode>,
        root_node: NodeId,
        quirks_mode: crate::html::grammar::QuirksMode,
    ) -> Self {
        XpathItemTree {
            arena,
            root_node,
            quirks_mode,
        }
    }

    /// Get the document's quirks mode.
    pub fn quirks_mode(&self) -> crate::html::grammar::QuirksMode {
        self.quirks_mode
    }

    pub(crate) fn get_index_node(&self, id: NodeId) -> &indextree::Node<XpathItemTreeNode> {
        self.arena
            .get(id)
            .expect("xpath item node missing from tree")
    }

    pub(crate) fn get(&self, id: NodeId) -> &XpathItemTreeNode {
        let indextree_node = self.get_index_node(id);

        indextree_node.get()
    }

    /// Get the document's root node.
    pub fn root(&self) -> &XpathItemTreeNode {
        self.get(self.root_node)
    }

    /// Get an iterator over all nodes in the tree.
    pub fn iter(&self) -> impl Iterator<Item = &XpathItemTreeNode> {
        self.arena.iter().map(|node| {
            let id = self.arena.get_node_id(node).expect("arena iterator yielded node without NodeId");
            self.get(id)
        })
    }
}

impl Display for XpathItemTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.root().display(self, DisplayFormatting::Raw, 0)
        )
    }
}

impl From<&HtmlDocument> for XpathItemTree {
    fn from(html_document: &HtmlDocument) -> Self {
        fn internal_from(
            current_html_node: &DocumentNode,
            html_document: &HtmlDocument,
            item_arena: &mut Arena<XpathItemTreeNode>,
        ) -> NodeId {
            let html_node = html_document
                .get_html_node(&current_html_node)
                .expect("html document missing expected node");

            let root_item_id = match html_node {
                HtmlNode::Tag(tag) => {
                    let node =
                        XpathItemTreeNode::ElementNode(ElementNode::new(tag.name.to_string()));

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
                        let attribute_node = XpathItemTreeNode::AttributeNode(attribute);
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
                    let node = XpathItemTreeNode::TextNode(TextNode::new(text.value.to_string()));

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
                HtmlNode::Comment(comment) => {
                    CommentNode::create(comment.value.clone(), item_arena)
                }
                HtmlNode::ProcessingInstruction(pi) => {
                    PINode::create(pi.target.clone(), pi.data.clone(), item_arena)
                }
                HtmlNode::Doctype(doctype) => DoctypeNode::create(
                    doctype.name.clone(),
                    doctype.public_id.clone(),
                    doctype.system_id.clone(),
                    item_arena,
                ),
            };

            for child in current_html_node.children(&html_document) {
                let child_node = internal_from(&child, html_document, item_arena);
                root_item_id.append(child_node, item_arena);
            }

            root_item_id
        }

        let mut item_arena = Arena::<XpathItemTreeNode>::new();
        let root_node_id =
            item_arena.new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode {}));
        let first_child = internal_from(&html_document.root_node, &html_document, &mut item_arena);
        root_node_id.append(first_child, &mut item_arena);

        XpathItemTree {
            arena: item_arena,
            root_node: root_node_id,
            quirks_mode: crate::html::grammar::QuirksMode::NoQuirks,
        }
    }
}
