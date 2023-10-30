//! https://www.w3.org/TR/xpath-datamodel-31/#intro

use super::{types::function_test::FunctionTest, xml_names::QName};

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-item
pub enum Item {
    Node(Node),
    Function(Function),
    AnyAtomicType(AnyAtomicType),
}

/// https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
pub enum AnyAtomicType {
    // TODO
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item
pub struct Function {
    // TODO
}

/// https://www.w3.org/TR/xpath-datamodel-31/#dt-node
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
pub enum NodeChild {
    ElementNode(ElementNode),
    PINode(PINode),
    CommentNode(CommentNode),
    TextNode(TextNode),
}

/// https://www.w3.org/TR/xpath-datamodel-31/#DocumentNode
pub struct DocumentNode {
    children: Vec<NodeChild>,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ElementNode
pub struct ElementNode {
    name: String,
    children: Vec<NodeChild>,
    string_value: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#AttributeNode
pub struct AttributeNode {
    name: String,
    string_value: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#NamespaceNode
pub struct NamespaceNode {
    prefix: String,
    uri: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#ProcessingInstructionNode
pub struct PINode {
    // TODO
}

/// https://www.w3.org/TR/xpath-datamodel-31/#CommentNode
pub struct CommentNode {
    content: String,
}

/// https://www.w3.org/TR/xpath-datamodel-31/#TextNode
pub struct TextNode {
    content: String,
}
