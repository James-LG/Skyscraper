//! https://www.w3.org/TR/xpath-datamodel-31/#intro

use std::{fmt::Display, iter};

use enum_extract_macro::EnumExtract;
use indextree::NodeId;
use ordered_float::OrderedFloat;

use super::{XpathItemTree, XpathItemTreeNodeData};

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-item
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash, EnumExtract)]
pub enum XpathItem<'tree> {
    /// A node in the [`XpathItemTree`](crate::xpath::XpathItemTree).
    ///
    ///  https://www.w3.org/TR/xpath-datamodel-31/#dt-node
    Node(&'tree XpathItemTreeNodeData),

    /// A function item.
    ///
    /// https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item
    Function(Function),

    /// An atomic value.
    ///
    /// https://www.w3.org/TR/xpath-datamodel-31/#dt-atomic-value
    AnyAtomicType(AnyAtomicType),
}

impl<'tree> From<&'tree XpathItemTreeNodeData> for XpathItem<'tree> {
    fn from(node: &'tree XpathItemTreeNodeData) -> Self {
        XpathItem::Node(node)
    }
}

///  An atomic value.
///
///  https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum AnyAtomicType {
    /// A boolean value.
    Boolean(bool),

    /// An integer value.
    Integer(i64),

    /// A float value.
    Float(OrderedFloat<f32>),

    /// A double precision float value.
    Double(OrderedFloat<f64>),

    /// A string value.
    String(String),
}

impl Display for AnyAtomicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyAtomicType::Boolean(b) => write!(f, "{}", b),
            AnyAtomicType::Integer(i) => write!(f, "{}", i),
            AnyAtomicType::Float(fl) => write!(f, "{}", fl),
            AnyAtomicType::Double(d) => write!(f, "{}", d),
            AnyAtomicType::String(s) => write!(f, "{}", s),
        }
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct Function {
    // TODO
}

impl Display for Function {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Function::fmt")
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#DocumentNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct XpathDocumentNode {}

impl XpathDocumentNode {
    pub fn all_text<'tree>(&self, tree: &'tree XpathItemTree) -> String {
        let strings: Vec<String> = tree
            .root_node
            .children(&tree.arena)
            .into_iter()
            .map(|x| tree.get(x))
            .map(|x| x.all_text(tree))
            .collect();

        let text = strings.join("");
        text
    }

    pub fn text<'tree>(&self, tree: &'tree XpathItemTree) -> Option<String> {
        let strings: Vec<String> = tree
            .root_node
            .children(&tree.arena)
            .into_iter()
            .map(|x| tree.get(x))
            .map(|x| x.text(tree))
            .filter_map(|x| x.map(|x| x.to_string()))
            .collect();

        strings.into_iter().next()
    }

    pub fn children<'tree>(&self, tree: &'tree XpathItemTree) -> Vec<&'tree XpathItemTreeNodeData> {
        tree.root_node
            .children(&tree.arena)
            .map(|x| tree.get(x))
            .collect()
    }
}
impl Display for XpathDocumentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocumentNode()")
    }
}

/// An element node such as an HTML tag.
///
///  https://www.w3.org/TR/xpath-datamodel-31/#ElementNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct ElementNode {
    /// The ID of the element.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The name of the element.
    pub name: String,
}

impl ElementNode {
    /// Create a new element node.
    pub(crate) fn new(name: String) -> Self {
        Self { id: None, name }
    }

    /// Set the ID of the element.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the element.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }

    /// Get all attributes of the element.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the element.
    ///
    /// # Returns
    ///
    /// A vector of all attributes of the element.
    pub fn attributes<'tree>(&self, tree: &'tree XpathItemTree) -> Vec<&'tree AttributeNode> {
        self.children(tree)
            .filter_map(|x| match x {
                XpathItemTreeNodeData::AttributeNode(attr) => Some(attr),
                _ => None,
            })
            .collect()
    }

    /// Get the value of an attribute.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute.
    ///
    /// # Returns
    ///
    /// The value of the attribute if it exists, or `None` if it does not.
    pub fn get_attribute<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        name: &str,
    ) -> Option<&'tree str> {
        self.attributes(tree)
            .iter()
            .find(|x| x.name == name)
            .map(|x| &*x.value)
    }

    /// Get all direct child nodes of the given element.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the element.
    ///
    /// # Returns
    ///
    /// An iterator over the child nodes.
    pub fn children<'tree>(
        &self,
        tree: &'tree XpathItemTree,
    ) -> impl Iterator<Item = &'tree XpathItemTreeNodeData> {
        self.id().children(&tree.arena).map(|x| tree.get(x))
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
        tree.get(self.id()).parent(tree)
    }

    /// Get an iterator over all text contained in this element and its descendants.
    ///
    /// Includes whitespace text nodes.
    /// Text nodes are split by opening and closing tags contained in the current element.
    pub fn itertext<'this, 'tree>(&'this self, tree: &'tree XpathItemTree) -> ElementTextIter<'this>
    where
        'tree: 'this,
    {
        ElementTextIter::new(tree, self)
    }

    /// Get all text contained in this element and its descendants.
    ///
    /// Use [`ElementNode::text`] to get _only_ text contained directly in this node.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this element is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this element and its descendants.
    pub fn all_text<'tree>(&self, tree: &'tree XpathItemTree) -> String {
        let strings: Vec<String> =
            // Get all children.
            Self::get_all_text_nodes(tree, self, true)
            .into_iter()
            .map(|x| x.content)
            .collect();

        let text = strings.join("");
        text
    }

    /// Get text directly contained in this element.
    ///
    /// Use [`ElementNode::all_text`] to get all text _including_ text in descendant nodes.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this element is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this element.
    pub fn text(&self, tree: &XpathItemTree) -> Option<String> {
        let strings: Vec<String> =
            // Get all children.
            Self::get_all_text_nodes(tree, self, false)
            .into_iter()
            .map(|x| x.content)
            .collect();

        strings.into_iter().next()
    }

    fn get_all_text_nodes(
        tree: &XpathItemTree,
        node: &ElementNode,
        recurse: bool,
    ) -> Vec<TextNode> {
        node
            // Get all children of the given node.
            .children(tree)
            // Combine all the direct and indirect children into a Vec.
            .fold(Vec::new(), |mut v, child| {
                match child {
                    XpathItemTreeNodeData::ElementNode(child_element) => {
                        if recurse {
                            // If this child is an element node, get all the text nodes in it.
                            v.extend(Self::get_all_text_nodes(tree, &child_element, recurse));
                        }
                    }
                    XpathItemTreeNodeData::TextNode(text) => {
                        // If this child is a text node, push it to the Vec.
                        v.push(text.clone());
                    }
                    _ => {}
                }
                v
            })
    }
}

/// An iterator over all text contained in a element and its descendants.
pub struct ElementTextIter<'a> {
    iter_chain: Box<dyn Iterator<Item = String> + 'a>,
}

impl<'a> ElementTextIter<'a> {
    pub(crate) fn new(tree: &'a XpathItemTree, node: &'a ElementNode) -> ElementTextIter<'a> {
        let mut iter_chain: Box<dyn Iterator<Item = String>> = Box::new(iter::empty());

        for child in node.children(tree) {
            match child {
                XpathItemTreeNodeData::TextNode(text) => {
                    iter_chain = Box::new(iter_chain.chain(iter::once(text.content.clone())));
                }
                XpathItemTreeNodeData::ElementNode(child_element) => {
                    iter_chain =
                        Box::new(iter_chain.chain(ElementTextIter::new(tree, child_element)));
                }
                _ => {}
            }
        }

        ElementTextIter { iter_chain }
    }
}

impl<'a> Iterator for ElementTextIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_chain.next()
    }
}

/// An attribute node.
///
///  https://www.w3.org/TR/xpath-datamodel-31/#AttributeNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct AttributeNode {
    /// The ID of the attribute.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The name of the attribute.
    pub name: String,

    /// The value of the attribute.
    pub value: String,
}

impl AttributeNode {
    /// Create a new attribute node.
    pub(crate) fn new(name: String, value: String) -> Self {
        Self {
            id: None,
            name,
            value,
        }
    }

    /// Set the ID of the attribute.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the attribute.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }
}

impl Display for AttributeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.name, self.value)
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ProcessingInstructionNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct PINode {
    // TODO
}

impl Display for PINode {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("PINode::fmt")
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#CommentNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct CommentNode {
    /// The value of the comment.
    pub content: String,
}

impl Display for CommentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!--{}-->", self.content)
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#TextNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct TextNode {
    /// The ID of the text node.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The value of the text node.
    pub content: String,

    /// Whether the text node contains only whitespace.
    pub only_whitespace: bool,
}

impl TextNode {
    /// Create a new text node.
    pub(crate) fn new(content: String, only_whitespace: bool) -> Self {
        Self {
            id: None,
            content,
            only_whitespace,
        }
    }

    /// Set the ID of the text node.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the text node.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }
}

impl Display for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.content)
    }
}
