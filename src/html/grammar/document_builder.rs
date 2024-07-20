use indextree::{Arena, NodeId};
use thiserror::Error;

use crate::xpath::{
    grammar::{
        data_model::{AttributeNode, ElementNode, TextNode, XpathItem},
        XpathItemTreeNode,
    },
    XpathItemTree,
};

#[derive(Debug, Error)]
#[error("DocumentBuilderError: {message}")]
pub struct DocumentBuilderError {
    message: String,
}
pub struct DocumentBuilder {
    arena: Arena<XpathItemTreeNode>,
    root_func: Option<(String, Box<dyn FnOnce(ElementBuilder) -> ElementBuilder>)>,
}

impl DocumentBuilder {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            root_func: None,
        }
    }

    pub fn with_root(
        mut self,
        tag_name: &str,
        f: impl FnOnce(ElementBuilder) -> ElementBuilder + 'static,
    ) -> Self {
        self.root_func = Some((tag_name.to_string(), Box::new(f)));

        self
    }

    pub fn build(mut self) -> Result<XpathItemTree, DocumentBuilderError> {
        let (tag_name, f) = self.root_func.ok_or(DocumentBuilderError {
            message: "DocumentBuilderError: root element is not set".to_string(),
        })?;

        let root_id = f(ElementBuilder::new(tag_name, None, &mut self.arena)).build()?;

        let document = XpathItemTree::new(self.arena, root_id);

        Ok(document)
    }
}

pub struct ElementBuilder<'arena> {
    parent_id: Option<NodeId>,
    arena: &'arena mut Arena<XpathItemTreeNode>,
    children: Vec<NodeId>,
    children_funcs: Vec<(String, Box<dyn FnOnce(ElementBuilder) -> ElementBuilder>)>,
    tag_name: String,
}

impl<'arena> ElementBuilder<'arena> {
    pub fn new(
        tag_name: String,
        parent_id: Option<NodeId>,
        arena: &'arena mut Arena<XpathItemTreeNode>,
    ) -> Self {
        Self {
            parent_id,
            arena,
            children: Vec::new(),
            children_funcs: Vec::new(),
            tag_name: tag_name.to_string(),
        }
    }

    pub fn add_element(
        mut self,
        tag_name: &str,
        f: impl FnOnce(ElementBuilder) -> ElementBuilder + 'static,
    ) -> Self {
        self.children_funcs
            .push((tag_name.to_string(), Box::new(f)));

        self
    }

    pub fn add_attribute(mut self, attribute: AttributeNode) -> Self {
        let child_id = self
            .arena
            .new_node(XpathItemTreeNode::AttributeNode(attribute));
        self.children.push(child_id);

        self
    }

    pub fn add_text(mut self, text: &str) -> Self {
        let child_id = self
            .arena
            .new_node(XpathItemTreeNode::TextNode(TextNode::new(text.to_string())));
        self.children.push(child_id);

        self
    }

    pub fn build(self) -> Result<NodeId, DocumentBuilderError> {
        let element_id = self
            .arena
            .new_node(XpathItemTreeNode::ElementNode(ElementNode::new(
                self.tag_name,
            )));

        for child_id in self.children {
            element_id.append(child_id, self.arena);
        }

        for (tag_name, f) in self.children_funcs {
            let child_id =
                f(ElementBuilder::new(tag_name, Some(element_id), self.arena)).build()?;
            element_id.append(child_id, self.arena);
        }

        if let Some(parent_id) = self.parent_id {
            parent_id.append(element_id, self.arena);
        }

        Ok(element_id)
    }
}
