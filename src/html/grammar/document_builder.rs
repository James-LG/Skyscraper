use indextree::{Arena, NodeId};
use thiserror::Error;

use crate::xpath::{
    grammar::{
        data_model::{
            AttributeNode, CommentNode, DoctypeNode, ElementNode, PINode, TextNode,
            XpathDocumentNode,
        },
        XpathItemTreeNode,
    },
    XpathItemTree,
};

/// An error that occurred while building a document with [`DocumentBuilder`].
#[derive(Debug, Error)]
#[error("DocumentBuilderError: {message}")]
pub struct DocumentBuilderError {
    message: String,
}

/// A builder for programmatically constructing an [`XpathItemTree`].
///
/// Use this when you need to create a document tree in code rather than by
/// parsing HTML text. Elements, text, comments, and other node types can be
/// added via a fluent builder API.
///
/// # Example
///
/// ```rust
/// use skyscraper::html::grammar::document_builder::DocumentBuilder;
///
/// let tree = DocumentBuilder::new()
///     .add_element("html", |html| {
///         html.add_element("body", |body| {
///             body.add_element("p", |p| p.add_text("Hello"))
///         })
///     })
///     .build()
///     .unwrap();
/// ```
pub struct DocumentBuilder {
    arena: Arena<XpathItemTreeNode>,
    funcs: Vec<
        Box<
            dyn FnOnce(
                &mut Arena<XpathItemTreeNode>,
                NodeId,
            ) -> Result<NodeId, DocumentBuilderError>,
        >,
    >,
}

impl DocumentBuilder {
    /// Create a new, empty document builder.
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            funcs: Vec::new(),
        }
    }

    /// Add a child element with the given tag name.
    ///
    /// The closure receives an [`ElementBuilder`] for configuring the element's
    /// children, attributes, and text content.
    pub fn add_element(
        mut self,
        tag_name: &str,
        f: impl FnOnce(ElementBuilder) -> ElementBuilder + 'static,
    ) -> Self {
        let tag_name = tag_name.to_string();
        self.funcs.push(Box::new(move |arena, _parent_id| {
            f(ElementBuilder::new(
                tag_name.clone(),
                arena,
            ))
            .build()
        }));

        self
    }

    /// Add a comment node to the document.
    pub fn add_comment(mut self, comment: &str) -> Self {
        let comment = comment.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id =
                arena.new_node(XpathItemTreeNode::CommentNode(CommentNode::new(comment)));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_comment_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    /// Add a processing instruction node to the document.
    pub fn add_processing_instruction(mut self, target: &str, data: &str) -> Self {
        let target = target.to_string();
        let data = data.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = PINode::create(target, data, arena);
            Ok(child_id)
        }));

        self
    }

    /// Add a DOCTYPE node with the given name (e.g. `"html"`).
    pub fn add_doctype(mut self, name: &str) -> Self {
        let name = name.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = DoctypeNode::create(name, None, None, arena);
            Ok(child_id)
        }));

        self
    }

    /// Add a DOCTYPE node with optional public and system identifiers.
    pub fn add_doctype_full(
        mut self,
        name: &str,
        public_id: Option<&str>,
        system_id: Option<&str>,
    ) -> Self {
        let name = name.to_string();
        let public_id = public_id.map(|s| s.to_string());
        let system_id = system_id.map(|s| s.to_string());
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = DoctypeNode::create(name, public_id, system_id, arena);
            Ok(child_id)
        }));

        self
    }

    /// Consume the builder and produce the finished [`XpathItemTree`].
    pub fn build(mut self) -> Result<XpathItemTree, DocumentBuilderError> {
        let document_node_id = self
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));

        for func in self.funcs {
            let child_id = func(&mut self.arena, document_node_id)?;
            document_node_id.append(child_id, &mut self.arena);
        }

        let document = XpathItemTree::new(self.arena, document_node_id);

        Ok(document)
    }
}

/// A builder for constructing a single element node within a [`DocumentBuilder`].
///
/// Provides methods for adding child elements, attributes, text, and comments
/// to the element being built.
pub struct ElementBuilder<'arena> {
    arena: &'arena mut Arena<XpathItemTreeNode>,
    funcs: Vec<
        Box<
            dyn FnOnce(
                &mut Arena<XpathItemTreeNode>,
                NodeId,
            ) -> Result<NodeId, DocumentBuilderError>,
        >,
    >,
    tag_name: String,
}

impl<'arena> ElementBuilder<'arena> {
    /// Create a new element builder with the given tag name.
    pub fn new(
        tag_name: String,
        arena: &'arena mut Arena<XpathItemTreeNode>,
    ) -> Self {
        Self {
            arena,
            funcs: Vec::new(),
            tag_name,
        }
    }

    /// Add a child element with the given tag name.
    ///
    /// The closure receives an [`ElementBuilder`] for configuring the child.
    pub fn add_element(
        mut self,
        tag_name: &str,
        f: impl FnOnce(ElementBuilder) -> ElementBuilder + 'static,
    ) -> Self {
        let tag_name = tag_name.to_string();
        self.funcs.push(Box::new(move |arena, _parent_id| {
            f(ElementBuilder::new(
                tag_name.clone(),
                arena,
            ))
            .build()
        }));

        self
    }

    /// Add multiple attributes from `(name, value)` string pairs.
    pub fn add_attributes_str(mut self, attributes: Vec<(&str, &str)>) -> Self {
        for (name, value) in attributes {
            self = self.add_attribute_str(name, value);
        }

        self
    }

    /// Add a single attribute from name and value strings.
    pub fn add_attribute_str(self, name: &str, value: &str) -> Self {
        self.add_attribute(AttributeNode::new(name.to_string(), value.to_string()))
    }

    /// Add an [`AttributeNode`] to this element.
    pub fn add_attribute(mut self, attribute: AttributeNode) -> Self {
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = arena.new_node(XpathItemTreeNode::AttributeNode(attribute));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_attribute_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    /// Add a text node as a child of this element.
    pub fn add_text(mut self, text: &str) -> Self {
        let text = text.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = arena.new_node(XpathItemTreeNode::TextNode(TextNode::new(text)));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_text_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    /// Add a comment node as a child of this element.
    pub fn add_comment(mut self, comment: &str) -> Self {
        let comment = comment.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id =
                arena.new_node(XpathItemTreeNode::CommentNode(CommentNode::new(comment)));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_comment_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    /// Add a processing instruction node as a child of this element.
    pub fn add_processing_instruction(mut self, target: &str, data: &str) -> Self {
        let target = target.to_string();
        let data = data.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = PINode::create(target, data, arena);
            Ok(child_id)
        }));

        self
    }

    /// Consume the builder and insert the element (with all its children) into the arena.
    pub fn build(self) -> Result<NodeId, DocumentBuilderError> {
        let element_id = self
            .arena
            .new_node(XpathItemTreeNode::ElementNode(ElementNode::new(
                self.tag_name,
            )));

        self.arena
            .get_mut(element_id)
            .unwrap()
            .get_mut()
            .as_element_node_mut()
            .unwrap()
            .set_id(element_id);

        for func in self.funcs {
            let child_id = func(self.arena, element_id)?;
            element_id.append(child_id, self.arena);
        }

        Ok(element_id)
    }
}
