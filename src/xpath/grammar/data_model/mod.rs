//! https://www.w3.org/TR/xpath-datamodel-31/#intro

use super::{
    types::function_test::FunctionTest, xml_names::QName, NonTreeXpathNode, XpathItemTreeNode,
};

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-item
#[derive(PartialEq, Debug)]
pub enum XpathItem<'tree> {
    Node(Node<'tree>),
    Function(Function),
    AnyAtomicType(AnyAtomicType),
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

/// https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
#[derive(PartialEq, Debug)]
pub enum AnyAtomicType {
    Boolean(bool),
    Number(i32), // TODO: Other types of numbers?
    String(String),
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item
#[derive(PartialEq, Debug)]
pub struct Function {
    // TODO
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-node
#[derive(PartialEq, Debug, Clone)]
pub enum Node<'tree> {
    TreeNode(XpathItemTreeNode<'tree>),
    NonTreeNode(NonTreeXpathNode),
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
#[derive(PartialEq, Debug)]
pub struct DocumentNode {}

/// https://www.w3.org/TR/xpath-datamodel-31/#ElementNode
#[derive(PartialEq, Debug)]
pub struct ElementNode {
    pub name: String,
    pub attributes: Vec<AttributeNode>,
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
#[derive(PartialEq, Debug, Clone)]
pub struct AttributeNode {
    pub name: String,
    pub value: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#NamespaceNode
#[derive(PartialEq, Debug, Clone)]
pub struct NamespaceNode {
    pub prefix: String,
    pub uri: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ProcessingInstructionNode
#[derive(PartialEq, Debug)]
pub struct PINode {
    // TODO
}

/// https://www.w3.org/TR/xpath-datamodel-31/#CommentNode
#[derive(PartialEq, Debug)]
pub struct CommentNode {
    pub content: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#TextNode
#[derive(PartialEq, Debug)]
pub struct TextNode {
    pub content: String,
}
