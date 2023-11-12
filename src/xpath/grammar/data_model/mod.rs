//! https://www.w3.org/TR/xpath-datamodel-31/#intro

use super::{types::function_test::FunctionTest, xml_names::QName};

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-item
#[derive(PartialEq, Debug)]
pub enum XpathItem {
    Node(Node),
    Function(Function),
    AnyAtomicType(AnyAtomicType),
}

/// https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
#[derive(PartialEq, Debug)]
pub enum AnyAtomicType {
    // TODO
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item
#[derive(PartialEq, Debug)]
pub struct Function {
    // TODO
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-node
#[derive(PartialEq, Debug)]
pub enum Node {
    DocumentNode(DocumentNode),
    ElementNode(ElementNode),
    AttributeNode(AttributeNode),
    TextNode(TextNode),
    NamespaceNode(NamespaceNode),
    PINode(PINode),
    CommentNode(CommentNode),
}

/// Subset of [Node] that are allowed to be children of other nodes.
#[derive(PartialEq, Debug)]
pub enum NodeChild {
    ElementNode(ElementNode),
    PINode(PINode),
    CommentNode(CommentNode),
    TextNode(TextNode),
}

impl From<NodeChild> for Node {
    fn from(value: NodeChild) -> Self {
        match value {
            NodeChild::ElementNode(node) => Node::ElementNode(node),
            NodeChild::PINode(node) => Node::PINode(node),
            NodeChild::CommentNode(node) => Node::CommentNode(node),
            NodeChild::TextNode(node) => Node::TextNode(node),
        }
    }
}

/// https://www.w3.org/TR/xpath-datamodel-31/#DocumentNode
#[derive(PartialEq, Debug)]
pub struct DocumentNode {
    pub children: Vec<NodeChild>,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ElementNode
#[derive(PartialEq, Debug)]
pub struct ElementNode {
    pub name: String,
    pub attributes: Vec<AttributeNode>,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#AttributeNode
#[derive(PartialEq, Debug)]
pub struct AttributeNode {
    pub name: String,
    pub value: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#NamespaceNode
#[derive(PartialEq, Debug)]
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
