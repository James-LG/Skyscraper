//! <https://www.w3.org/TR/xpath-datamodel-31/#intro>

use std::fmt::{Debug, Display};

use enum_extract_macro::EnumExtract;
use indextree::{Arena, NodeId};
use ordered_float::OrderedFloat;

use super::{DisplayFormatting, TextIter, XpathItemTree, XpathItemTreeNode, VOID_ELEMENTS};

/// Escape characters in an attribute value for HTML serialization.
fn escape_attribute_value(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_text_content(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#dt-item>
#[derive(PartialEq, Eq, Debug, Clone, Hash, EnumExtract)]
pub enum XpathItem<'tree> {
    /// A node in the [`XpathItemTree`].
    ///
    ///  <https://www.w3.org/TR/xpath-datamodel-31/#dt-node>
    Node(&'tree XpathItemTreeNode),

    /// A function item.
    ///
    /// <https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item>
    Function(Function),

    /// An atomic value.
    ///
    /// <https://www.w3.org/TR/xpath-datamodel-31/#dt-atomic-value>
    AnyAtomicType(AnyAtomicType),
}

impl<'tree> From<&'tree XpathItemTreeNode> for XpathItem<'tree> {
    fn from(node: &'tree XpathItemTreeNode) -> Self {
        XpathItem::Node(node)
    }
}

/// An atomic value.
///
/// <https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy>
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

/// <https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item>
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Function {
    /// A reference to a named function, e.g. `fn:abs#1`.
    Named {
        /// The function name as a string (e.g. "fn:abs").
        name: String,
        /// The arity of the function.
        arity: u32,
    },
    /// An inline function expression, e.g. `function($x) { $x + 1 }`.
    ///
    /// The body is stored as source text rather than a parsed AST to avoid
    /// circular module dependencies (`data_model` cannot import `expressions`).
    /// It is re-parsed via `expr()` each time the function is called.
    ///
    /// Note: inline functions do not currently capture closure variables from the
    /// definition scope. They evaluate using the caller's variable context.
    Inline {
        /// The parameter names.
        params: Vec<String>,
        /// The source text of the function body expression (inside the braces).
        body_source: String,
    },
    /// An XPath 3.1 map, e.g. `map { "x": 1, "y": 2 }`.
    ///
    /// Values are atomized on construction; node values are converted to their
    /// string representations.
    Map {
        /// The map entries as (key, value-sequence) pairs.
        entries: Vec<(AnyAtomicType, Vec<AnyAtomicType>)>,
    },
    /// An XPath 3.1 array, e.g. `[1, 2, 3]` or `array { 1, 2, 3 }`.
    ///
    /// Each member is a sequence of atomic values (atomized on construction).
    Array {
        /// The array members, each of which is a sequence of atomic values.
        members: Vec<Vec<AnyAtomicType>>,
    },
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Named { name, arity } => write!(f, "{}#{}", name, arity),
            Function::Inline {
                params,
                body_source,
            } => {
                write!(f, "function(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "${}", param)?;
                }
                write!(f, ") {{ {} }}", body_source)
            }
            Function::Map { entries } => {
                write!(f, "map {{")?;
                for (i, (key, values)) in entries.iter().enumerate() {
                    if i == 0 {
                        write!(f, " ")?;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: ", key)?;
                    if values.len() == 1 {
                        write!(f, "{}", values[0])?;
                    } else {
                        write!(f, "(")?;
                        for (j, v) in values.iter().enumerate() {
                            if j > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", v)?;
                        }
                        write!(f, ")")?;
                    }
                }
                write!(f, " }}")
            }
            Function::Array { members } => {
                write!(f, "[")?;
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if member.len() == 1 {
                        write!(f, "{}", member[0])?;
                    } else {
                        write!(f, "(")?;
                        for (j, v) in member.iter().enumerate() {
                            if j > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", v)?;
                        }
                        write!(f, ")")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#DocumentNode>
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct XpathDocumentNode {}

impl XpathDocumentNode {
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Get all text contained in this element and its descendants.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this document is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this document and its descendants.
    pub fn text_content<'tree>(&self, tree: &'tree XpathItemTree) -> String {
        let strings: Vec<String> = tree
            .root_node
            .children(&tree.arena)
            .into_iter()
            .map(|x| tree.get(x))
            .map(|x| x.text_content(tree))
            .collect();

        let text = strings.join("");
        text
    }

    /// Text before the first subelement. This is either a string or the value None, if there was no text.
    ///
    /// Use [`XpathDocumentNode::text_content`] to get all text _including_ text in descendant nodes.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this document is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this document.
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

    /// Get all children of the document.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the document.
    ///
    /// # Returns
    ///
    /// A vector of all children of the document.
    pub fn children<'tree>(&self, tree: &'tree XpathItemTree) -> Vec<&'tree XpathItemTreeNode> {
        tree.root_node
            .children(&tree.arena)
            .map(|x| tree.get(x))
            .collect()
    }

    pub fn display<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        formatting: DisplayFormatting,
    ) -> String {
        match formatting {
            DisplayFormatting::Raw => {
                // Raw mode: include all children (comments, text, elements)
                let child_strings: Vec<String> = tree
                    .root_node
                    .children(&tree.arena)
                    .map(|x| tree.get(x))
                    .map(|x| x.display(tree, formatting, 0))
                    .collect();
                child_strings.join("")
            }
            _ => {
                let children = self.children(tree);

                let element_strings: Vec<String> = children
                    .iter()
                    .filter_map(|x| x.as_element_node().ok())
                    .map(|x| x.display(tree, formatting, 0))
                    .collect();

                element_strings.join("\n")
            }
        }
    }
}

/// An element node such as an HTML tag.
///
/// <https://www.w3.org/TR/xpath-datamodel-31/#ElementNode>
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ElementNode {
    /// The ID of the element.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The name of the element.
    pub name: String,

    /// The namespace of the element.
    pub namespace: Option<String>,
}

impl std::fmt::Debug for ElementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore id in debug output
        f.debug_struct("ElementNode")
            .field("name", &self.name)
            .finish()
    }
}

impl ElementNode {
    /// Create a new element node.
    pub(crate) fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            namespace: None,
        }
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
                XpathItemTreeNode::AttributeNode(attr) => Some(attr),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn attributes_arena<'arena>(
        &self,
        arena: &'arena Arena<XpathItemTreeNode>,
    ) -> Vec<&'arena AttributeNode> {
        self.children_arena(arena)
            .filter_map(|x| match x {
                XpathItemTreeNode::AttributeNode(attr) => Some(attr),
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

    pub(crate) fn add_attribute<'arena>(
        &self,
        arena: &mut Arena<XpathItemTreeNode>,
        name: String,
        value: String,
    ) -> NodeId {
        let attr = arena.new_node(XpathItemTreeNode::AttributeNode(AttributeNode::new(
            name, value,
        )));
        self.id().append(attr, arena);

        attr
    }

    /// Get all direct child nodes of the given element.
    /// Note this _does_ include attribute nodes.
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
    ) -> impl Iterator<Item = &'tree XpathItemTreeNode> {
        self.id().children(&tree.arena).map(|x| tree.get(x))
    }

    pub(crate) fn children_arena<'arena>(
        &self,
        arena: &'arena Arena<XpathItemTreeNode>,
    ) -> impl Iterator<Item = &'arena XpathItemTreeNode> {
        self.id()
            .children(arena)
            .map(|x| arena.get(x).expect("node missing from arena").get())
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
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        tree.get(self.id()).parent(tree)
    }

    /// Get an iterator over all text contained in this element and its descendants.
    ///
    /// Includes whitespace text nodes.
    /// Text nodes are split by opening and closing tags contained in the current element.
    pub fn itertext<'this, 'tree>(&'this self, tree: &'tree XpathItemTree) -> TextIter<'this>
    where
        'tree: 'this,
    {
        TextIter::new(tree, tree.get(self.id()))
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
        self.itertext(tree).collect::<Vec<String>>().join("")
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
                    XpathItemTreeNode::ElementNode(child_element) => {
                        if recurse {
                            // If this child is an element node, get all the text nodes in it.
                            v.extend(Self::get_all_text_nodes(tree, &child_element, recurse));
                        }
                    }
                    XpathItemTreeNode::TextNode(text) => {
                        // If this child is a text node, push it to the Vec.
                        v.push(text.clone());
                    }
                    _ => {}
                }
                v
            })
    }

    /// Get the [XpathItem] representation of the element.
    pub fn to_item<'tree>(&self, tree: &'tree XpathItemTree) -> XpathItem<'tree> {
        XpathItem::Node(tree.get(self.id()))
    }

    pub fn display<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        formatting: DisplayFormatting,
        indent: usize,
    ) -> String {
        match formatting {
            DisplayFormatting::Raw => self.display_raw(tree),
            _ => self.display_pretty(tree, formatting, indent),
        }
    }

    fn display_raw<'tree>(&self, tree: &'tree XpathItemTree) -> String {
        let attributes = self.attributes(tree);

        let mut display_string = String::new();

        // start tag
        display_string.push_str(&format!("<{}", self.name));
        for attr in &attributes {
            display_string.push_str(&attr.prefix);
            let escaped_value = escape_attribute_value(&attr.value);
            let display_name = attr.original_name.as_deref().unwrap_or(&attr.name);
            display_string.push_str(&format!("{}=\"{}\"", display_name, escaped_value));
        }
        display_string.push('>');

        let is_void = VOID_ELEMENTS.contains(&self.name.as_str());

        if !is_void {
            // children (all types except attributes)
            let children_without_attributes =
                self.children(tree).filter(|x| !x.is_attribute_node());
            for child in children_without_attributes {
                display_string.push_str(&child.display(tree, DisplayFormatting::Raw, 0));
            }

            // end tag
            display_string.push_str(&format!("</{}>", self.name));
        }

        display_string
    }

    fn display_pretty<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        formatting: DisplayFormatting,
        indent: usize,
    ) -> String {
        let displayed_children: Vec<String> = match formatting {
            DisplayFormatting::Pretty => {
                let children_without_attributes =
                    self.children(tree).filter(|x| !x.is_attribute_node());
                children_without_attributes
                    .map(|x| x.display(tree, formatting, indent + 1))
                    .filter(|x| !x.trim().is_empty())
                    .collect()
            }
            DisplayFormatting::NoChildren => Vec::new(),
            DisplayFormatting::Raw => unreachable!(),
        };

        let attributes = self.attributes(tree);
        let displayed_attributes: Vec<String> = attributes.iter().map(|x| x.to_string()).collect();

        // indent the element
        let indentation = "  ".repeat(indent);

        let mut display_string = String::new();

        // display the start tag
        if displayed_attributes.is_empty() {
            display_string.push_str(&format!("{}<{}>", indentation, self.name,));
        } else {
            display_string.push_str(&format!(
                "{}<{} {}>",
                indentation,
                self.name,
                displayed_attributes.join(" "),
            ));
        }

        // display the children
        if !displayed_children.is_empty() {
            display_string.push_str(&format!(
                "\n{}\n{}",
                &displayed_children.join("\n"),
                indentation
            ));
        }

        // display the end tag
        display_string.push_str(&format!("</{}>", self.name));

        return display_string;
    }
}

/// An attribute node.
///
/// <https://www.w3.org/TR/xpath-datamodel-31/#AttributeNode>
#[derive(Eq, Clone, Hash)]
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

    /// Whitespace prefix before this attribute in the original source.
    /// Used for round-trip fidelity in Raw display mode.
    pub prefix: String,

    /// Original attribute name before lowercasing (e.g. "viewBox").
    /// Used for round-trip fidelity in Raw display mode.
    pub original_name: Option<String>,

    /// The namespace URI of this attribute, if any.
    ///
    /// Set for foreign attributes like `xlink:href` (xlink namespace),
    /// `xml:lang` (XML namespace), and `xmlns` (xmlns namespace)
    /// per WHATWG 13.2.6.3.
    pub namespace: Option<String>,
}

impl Debug for AttributeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore id in debug output
        f.debug_struct("AttributeNode")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

impl AttributeNode {
    /// Create a new attribute node.
    pub(crate) fn new(name: String, value: String) -> Self {
        Self {
            id: None,
            name,
            value,
            prefix: String::from(" "),
            original_name: None,
            namespace: None,
        }
    }

    /// Create a new attribute node with a custom prefix, original name, and namespace.
    pub(crate) fn with_prefix(
        name: String,
        value: String,
        prefix: String,
        original_name: Option<String>,
        namespace: Option<String>,
    ) -> Self {
        Self {
            id: None,
            name,
            value,
            prefix,
            original_name,
            namespace,
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

    /// Get the parent of the attribute.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the attribute.
    ///
    /// # Returns
    ///
    /// The parent of the attribute if it exists, or `None` if it does not.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        tree.get(self.id()).parent(tree)
    }
}

impl Display for AttributeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.name, self.value)
    }
}

impl PartialEq for AttributeNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#ProcessingInstructionNode>
#[derive(PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct PINode {
    /// The target of the processing instruction (an NCName).
    pub target: String,

    /// The string content after the target.
    pub data: String,

    /// The ID of the processing instruction node.
    id: Option<NodeId>,
}

impl PINode {
    /// Create a new processing instruction node.
    pub(crate) fn new(target: String, data: String) -> Self {
        Self {
            target,
            data,
            id: None,
        }
    }

    pub(crate) fn create(
        target: String,
        data: String,
        arena: &mut Arena<XpathItemTreeNode>,
    ) -> NodeId {
        let node_id = arena.new_node(XpathItemTreeNode::PINode(PINode::new(target, data)));

        arena
            .get_mut(node_id)
            .unwrap()
            .get_mut()
            .as_pi_node_mut()
            .unwrap()
            .set_id(node_id);

        node_id
    }

    /// Set the ID of the processing instruction node.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the processing instruction node.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }
}

impl PartialEq for PINode {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.data == other.data
    }
}

impl Display for PINode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.data.is_empty() {
            write!(f, "<?{}?>", self.target)
        } else {
            write!(f, "<?{} {}?>", self.target, self.data)
        }
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#CommentNode>
#[derive(PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct CommentNode {
    /// The value of the comment.
    pub content: String,

    /// The ID of the comment node.
    id: Option<NodeId>,
}

impl CommentNode {
    /// Create a new comment node.
    pub(crate) fn new(content: String) -> Self {
        Self { content, id: None }
    }

    pub(crate) fn create(content: String, arena: &mut Arena<XpathItemTreeNode>) -> NodeId {
        let node_id = arena.new_node(XpathItemTreeNode::CommentNode(CommentNode::new(content)));

        arena
            .get_mut(node_id)
            .unwrap()
            .get_mut()
            .as_comment_node_mut()
            .unwrap()
            .set_id(node_id);

        node_id
    }

    /// Set the ID of the comment node.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the comment node.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }

    /// Get the parent of the comment.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the comment.
    ///
    /// # Returns
    ///
    /// The parent of the comment if it exists, or `None` if it does not.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        tree.get(self.id()).parent(tree)
    }
}

impl Display for CommentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!--{}-->", self.content)
    }
}

impl PartialEq for CommentNode {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

/// A document type node representing `<!DOCTYPE ...>`.
///
/// Note: DOCTYPE is not a valid XPath 3.1 node type. It is kept in the tree
/// for serialization fidelity but is excluded from `node()` kind tests and
/// axis traversal results.
#[derive(PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct DoctypeNode {
    /// The name of the document type (e.g. "html").
    pub name: String,

    /// The public identifier, if present.
    pub public_id: Option<String>,

    /// The system identifier, if present.
    pub system_id: Option<String>,

    /// The ID of the doctype node in the arena.
    id: Option<NodeId>,
}

impl DoctypeNode {
    pub(crate) fn new(name: String, public_id: Option<String>, system_id: Option<String>) -> Self {
        Self {
            name,
            public_id,
            system_id,
            id: None,
        }
    }

    pub(crate) fn create(
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
        arena: &mut Arena<XpathItemTreeNode>,
    ) -> NodeId {
        let node_id = arena.new_node(XpathItemTreeNode::DoctypeNode(DoctypeNode::new(
            name, public_id, system_id,
        )));

        arena
            .get_mut(node_id)
            .unwrap()
            .get_mut()
            .as_doctype_node_mut()
            .unwrap()
            .set_id(node_id);

        node_id
    }

    /// Set the ID of the doctype node.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the doctype node.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }
}

impl PartialEq for DoctypeNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.public_id == other.public_id
            && self.system_id == other.system_id
    }
}

impl Display for DoctypeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.public_id, &self.system_id) {
            (Some(public), Some(system)) => {
                write!(
                    f,
                    r#"<!DOCTYPE {} PUBLIC "{}" "{}">"#,
                    self.name, public, system
                )
            }
            (Some(public), None) => {
                write!(f, r#"<!DOCTYPE {} PUBLIC "{}">"#, self.name, public)
            }
            (None, Some(system)) => {
                write!(f, r#"<!DOCTYPE {} SYSTEM "{}">"#, self.name, system)
            }
            (None, None) => {
                write!(f, "<!DOCTYPE {}>", self.name)
            }
        }
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#TextNode>
#[derive(Eq, Hash, Clone)]
pub struct TextNode {
    /// The ID of the text node.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The value of the text node.
    pub content: String,
}

impl Debug for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore id in debug output
        f.debug_struct("TextNode")
            .field("content", &self.content)
            .finish()
    }
}

impl TextNode {
    /// Create a new text node.
    pub(crate) fn new(content: String) -> Self {
        Self { id: None, content }
    }

    /// Set the ID of the text node.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the text node.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }

    /// Whether the text contains only whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get the parent of the text.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the text.
    ///
    /// # Returns
    ///
    /// The parent of the text if it exists, or `None` if it does not.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        tree.get(self.id()).parent(tree)
    }

    pub fn display(
        &self,
        _tree: &XpathItemTree,
        formatting: DisplayFormatting,
        indent: usize,
    ) -> String {
        match formatting {
            DisplayFormatting::Raw => escape_text_content(&self.content),
            DisplayFormatting::NoChildren => {
                let indentation = "  ".repeat(indent);
                format!("{}{}", indentation, self.content)
            }
            DisplayFormatting::Pretty => {
                let indentation = "  ".repeat(indent);
                format!("{}{}", indentation, self.content.trim())
            }
        }
    }
}

impl PartialEq for TextNode {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl Display for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}
