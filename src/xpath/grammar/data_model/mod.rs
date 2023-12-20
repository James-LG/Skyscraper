//! https://www.w3.org/TR/xpath-datamodel-31/#intro

use std::fmt::Display;

use super::{
    types::function_test::FunctionTest, xml_names::QName, NonTreeXpathNode, XpathItemTree,
    XpathItemTreeNode,
};

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-item
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum XpathItem<'tree> {
    Node(Node<'tree>),
    Function(Function),
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

/// https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum AnyAtomicType {
    Boolean(bool),
    Integer(i64),
    Float(f32),
    Double(f64),
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
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct Function {
    // TODO
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Function::fmt")
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-node
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Node<'tree> {
    TreeNode(XpathItemTreeNode<'tree>),
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
#[derive(PartialEq, PartialOrd, Debug)]
pub struct XpathDocumentNode {}

impl Display for XpathDocumentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocumentNode()")
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ElementNode
#[derive(PartialEq, PartialOrd, Debug)]
pub struct ElementNode {
    pub name: String,
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
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|x| x.name == name)
            .map(|x| &*x.value)
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#AttributeNode
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct AttributeNode {
    pub name: String,
    pub value: String,
}

impl Display for AttributeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.name, self.value)
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#NamespaceNode
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct NamespaceNode {
    pub prefix: String,
    pub uri: String,
}

impl Display for NamespaceNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamespaceNode({}:{})", self.prefix, self.uri)
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ProcessingInstructionNode
#[derive(PartialEq, PartialOrd, Debug)]
pub struct PINode {
    // TODO
}

impl Display for PINode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("PINode::fmt")
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#CommentNode
#[derive(PartialEq, PartialOrd, Debug)]
pub struct CommentNode {
    pub content: String,
}

impl Display for CommentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!--{}-->", self.content)
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#TextNode
#[derive(PartialEq, PartialOrd, Debug)]
pub struct TextNode {
    pub content: String,
}

impl Display for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.content)
    }
}
