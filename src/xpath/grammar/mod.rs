// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md
// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar

pub mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod xml_names;

use std::fmt::Display;

pub use expressions::{xpath, XPath};

use indextree::{Arena, NodeId};

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{
        AttributeNode, CommentNode, ElementNode, NamespaceNode, PINode, TextNode, XpathDocumentNode,
    },
};

/// Subset of [Node] that are not allowed to have child nodes.
/// Should be disjoint with [XpathItemTreeNodeData].
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum NonTreeXpathNode {
    AttributeNode(AttributeNode),
    NamespaceNode(NamespaceNode),
}

impl Display for NonTreeXpathNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NonTreeXpathNode::AttributeNode(node) => write!(f, "{}", node),
            NonTreeXpathNode::NamespaceNode(node) => write!(f, "{}", node),
        }
    }
}

/// Subset of [Node] that are allowed to have child nodes.
/// Should be disjoint with [NonTreeXpathNode].
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum XpathItemTreeNodeData {
    DocumentNode(XpathDocumentNode),
    ElementNode(ElementNode),
    PINode(PINode),
    CommentNode(CommentNode),
    TextNode(TextNode),
}

impl XpathItemTreeNodeData {
    pub fn unwrap_element_ref(&self) -> &ElementNode {
        match self {
            XpathItemTreeNodeData::ElementNode(node) => node,
            _ => panic!("tree item is not an element"),
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct XpathItemTreeNode<'a> {
    id: NodeId,
    pub data: &'a XpathItemTreeNodeData,
}

impl Display for XpathItemTreeNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data {
            XpathItemTreeNodeData::DocumentNode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::ElementNode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::PINode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::CommentNode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::TextNode(node) => write!(f, "{}", node),
        }
    }
}

impl<'a> XpathItemTreeNode<'a> {
    pub fn children(&self, tree: &'a XpathItemTree) -> impl Iterator<Item = XpathItemTreeNode<'a>> {
        self.id
            .children(&tree.arena)
            .into_iter()
            .map(move |id| tree.get(id))
    }

    pub fn parent(&self, tree: &'a XpathItemTree) -> Option<XpathItemTreeNode<'a>> {
        tree.get_index_node(self.id).parent().map(|id| tree.get(id))
    }

    pub fn text(&self, tree: &'a XpathItemTree) -> String {
        fn get_all_text_nodes(tree: &XpathItemTree, node: &XpathItemTreeNode) -> Vec<TextNode> {
            node
                // Get all children of the given node.
                .children(tree)
                // Combine all the direct and indirect children into a Vec.
                .fold(Vec::new(), |mut v, child| {
                    // If this child is a text node, push it to the Vec.
                    if let XpathItemTreeNodeData::TextNode(text) = child.data {
                        v.push(text.clone());
                    }
                    // Otherwise, get all the text nodes descending from this child.
                    else {
                        v.extend(get_all_text_nodes(tree, &child));
                    }
                    v
                })
        }

        let strings: Vec<String> =
            // Get all children.
            get_all_text_nodes(tree, self)
            .into_iter()
            // Filter out all whitespace-only text nodes
            .filter_map(|x| {
                if x.only_whitespace {
                    None
                } else {
                    Some(x.content)
                }
            })
            .collect();

        // Merge all text into a single string.
        // Space delimited.
        let text = strings.join(" ");

        text
    }
}

pub struct XpathItemTree {
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

    pub fn get(&self, id: NodeId) -> XpathItemTreeNode<'_> {
        let indextree_node = self.get_index_node(id);

        let data = indextree_node.get();
        XpathItemTreeNode { id, data }
    }

    pub fn root(&self) -> XpathItemTreeNode<'_> {
        self.get(self.root_node)
    }
}

impl XpathItemTree {
    pub fn from_html_document(html_document: &HtmlDocument) -> Self {
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
                    content: text.value.to_string(),
                    only_whitespace: text.only_whitespace,
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
            item_arena.new_node(XpathItemTreeNodeData::DocumentNode(XpathDocumentNode {}));
        let first_child =
            internal_from_html_document(&html_document.root_node, &html_document, &mut item_arena);
        root_node_id.append(first_child, &mut item_arena);

        XpathItemTree {
            arena: item_arena,
            root_node: root_node_id,
        }
    }
}
