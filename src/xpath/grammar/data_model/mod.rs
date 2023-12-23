//! https://www.w3.org/TR/xpath-datamodel-31/#intro

use std::fmt::Display;

use ordered_float::OrderedFloat;

use super::{NonTreeXpathNode, XpathItemTreeNode};

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-item
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum XpathItem<'tree> {
    /// A node in the [`XpathItemTree`](crate::xpath::XpathItemTree).
    ///
    ///  https://www.w3.org/TR/xpath-datamodel-31/#dt-node
    Node(Node<'tree>),

    /// A function item.
    ///
    /// https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item
    Function(Function),

    /// An atomic value.
    ///
    /// https://www.w3.org/TR/xpath-datamodel-31/#dt-atomic-value
    AnyAtomicType(AnyAtomicType),
}

impl Display for XpathItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XpathItem::Node(node) => write!(f, "{}", node),
            XpathItem::Function(function) => write!(f, "{}", function),
            XpathItem::AnyAtomicType(atomic_type) => write!(f, "{}", atomic_type),
        }
    }
}

impl<'tree> XpathItem<'tree> {
    pub fn unwrap_node(self) -> Node<'tree> {
        match self {
            XpathItem::Node(node) => node,
            _ => panic!("Expected XpathItem::Node"),
        }
    }

    pub fn unwrap_node_ref(&self) -> &Node<'tree> {
        match self {
            XpathItem::Node(node) => node,
            _ => panic!("Expected XpathItem::Node"),
        }
    }
}

impl<'tree> From<Node<'tree>> for XpathItem<'tree> {
    fn from(node: Node<'tree>) -> Self {
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

/// A node in the [`XpathItemTree`](crate::xpath::XpathItemTree).
///
///  https://www.w3.org/TR/xpath-datamodel-31/#dt-node
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum Node<'tree> {
    /// A node in the [`XpathItemTree`](crate::xpath::XpathItemTree).
    TreeNode(XpathItemTreeNode<'tree>),

    /// A node that is not part of an [`XpathItemTree`](crate::xpath::XpathItemTree).
    NonTreeNode(NonTreeXpathNode),
}

impl Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::TreeNode(node) => write!(f, "{}", node),
            Node::NonTreeNode(node) => write!(f, "{}", node),
        }
    }
}

impl<'tree> Node<'tree> {
    pub fn unwrap_tree_node(self) -> XpathItemTreeNode<'tree> {
        match self {
            Node::TreeNode(node) => node,
            _ => panic!("Expected Node::TreeNode"),
        }
    }

    pub fn unwrap_tree_node_ref(&self) -> &XpathItemTreeNode<'tree> {
        match self {
            Node::TreeNode(node) => node,
            _ => panic!("Expected Node::TreeNode"),
        }
    }

    pub fn unwrap_non_tree_node(self) -> NonTreeXpathNode {
        match self {
            Node::NonTreeNode(node) => node,
            _ => panic!("Expected Node::NonTreeNode"),
        }
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#DocumentNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct XpathDocumentNode {}

impl Display for XpathDocumentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocumentNode()")
    }
}

/// An element node such as an HTML tag.
///
///  https://www.w3.org/TR/xpath-datamodel-31/#ElementNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct ElementNode {
    /// The name of the element.
    pub name: String,

    /// The attributes of the element.
    pub attributes: Vec<AttributeNode>,
}

impl Display for ElementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}", self.name)?;
        for attribute in &self.attributes {
            write!(f, " {}", attribute)?;
        }
        write!(f, "/>")
    }
}

impl ElementNode {
    /// Get the value of an attribute.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute.
    ///
    /// # Returns
    ///
    /// The value of the attribute if it exists, or `None` if it does not.
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|x| x.name == name)
            .map(|x| &*x.value)
    }
}

/// An attribute node.
///
///  https://www.w3.org/TR/xpath-datamodel-31/#AttributeNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct AttributeNode {
    /// The name of the attribute.
    pub name: String,

    /// The value of the attribute.
    pub value: String,
}

impl Display for AttributeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.name, self.value)
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#NamespaceNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct NamespaceNode {
    // TODO
}

impl Display for NamespaceNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("NamespaceNode::fmt")
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ProcessingInstructionNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct PINode {
    // TODO
}

impl Display for PINode {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("PINode::fmt")
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#CommentNode
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
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
    /// The value of the text node.
    pub content: String,

    /// Whether the text node contains only whitespace.
    pub only_whitespace: bool,
}

impl Display for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.content)
    }
}
