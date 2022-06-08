mod tokenizer;
pub mod parse;

use std::collections::HashMap;

use indextree::{NodeId, Arena};

pub use crate::html::parse::parse;

/// Represents an HTML tag and it's attributes.
pub struct HtmlTag {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

impl HtmlTag {
    /// Creates a new tag with the given name and no attributes.
    pub fn new(name: String) -> HtmlTag {
        HtmlTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

impl HtmlTag {
    /// Gets any direct HtmlNode::Text children and concatenates them into a single string
    /// separated by a space character.
    pub fn get_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, false)
    }

    /// Gets all HtmlNode::Text children and concatenates them into a single string separated
    /// by a space character.
    pub fn get_all_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, true)
    }

    fn internal_get_text(&self, doc_node: &DocumentNode, document: &HtmlDocument, recurse: bool) -> Option<String> {
        let mut o_text: Option<String> = None;
        let children = doc_node.children(&document);

        // Iterate through this tag's children
        for child in children {
            let child_node = document.get_html_node(&child);
            if let Some(child_node) = child_node {
                match child_node {
                    HtmlNode::Text(text) => {
                        // If the child is a text, simply append its text.
                        o_text = Some(HtmlTag::append_text(o_text, text.to_string()));
                    },
                    HtmlNode::Tag(_) => {
                        // If the child is a tag, only append its text if recurse=true was passed,
                        // otherwise skip this node.
                        if recurse {
                            let o_child_text = child_node.internal_get_text(&child, &document, true);
                            if let Some(child_text) = o_child_text {
                                o_text = Some(HtmlTag::append_text(o_text, child_text));
                            }
                        }
                    }
                }
            }
        }

        return o_text;
    }

    fn append_text(o_text: Option<String>, append_text: String) -> String {
        match o_text {
            Some(t) => {
                format!("{} {}", t, append_text)
            },
            None => {
                append_text
            },
        }
    }
}

/// An HTML node can be either a tag or raw text.
pub enum HtmlNode {
    Tag(HtmlTag),
    Text(String),
}

impl HtmlNode {
    /// Gets any direct HtmlNode::Text children and concatenates them into a single string
    /// separated by a space character.
    pub fn get_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, false)
    }

    /// Gets all HtmlNode::Text children and concatenates them into a single string separated
    /// by a space character.
    pub fn get_all_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, true)
    }

    /// Gets any direct HtmlNode::Text children and concatenates them into a single string
    /// separated by new line characters.
    fn internal_get_text(&self, doc_node: &DocumentNode, document: &HtmlDocument, recurse: bool) -> Option<String> {
        match self {
            HtmlNode::Tag(tag) => {
                if recurse {
                    tag.get_all_text(doc_node, document)
                } else {
                    tag.get_text(doc_node, document)
                }
            },
            HtmlNode::Text(text) => Some(text.to_string()),
        }
    }
}

/// HTML document tree represented by an indextree arena and a root node.
/// 
/// Documents must have a single root node to be valid.
pub struct HtmlDocument {
    arena: Arena<HtmlNode>,
    pub root_key: DocumentNode,
}

impl HtmlDocument {
    pub fn new(arena: Arena<HtmlNode>, root_key: DocumentNode) -> HtmlDocument {
        HtmlDocument { arena, root_key }
    }

    pub fn get_html_node<'a>(&self, node: &DocumentNode) -> Option<&HtmlNode> {
        self.arena.get(node.id).map(|x| x.get())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub struct DocumentNode {
    id: NodeId
}

impl DocumentNode {
    pub fn new(id: NodeId) -> DocumentNode {
        DocumentNode { id }
    }

    pub fn get_all_text(&self, document: &HtmlDocument) -> Option<String> {
        match document.get_html_node(self) {
            Some(html_node) => html_node.get_all_text(self, document),
            None => None
        }
    }

    pub fn get_text(&self, document: &HtmlDocument) -> Option<String> {
        match document.get_html_node(self) {
            Some(html_node) => html_node.get_text(self, document),
            None => None
        }
    }

    pub fn children<'a>(&self, document: &'a HtmlDocument) -> impl Iterator<Item=DocumentNode> + 'a {
        Box::new(self.id.children(&document.arena).map(|node_id| DocumentNode::new(node_id)))
    }

    pub fn parent(&self, document: &HtmlDocument) -> Option<DocumentNode> {
        self.id.ancestors(&document.arena).skip(1).next()
            .map(|node_id| DocumentNode::new(node_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_node_get_text_should_work_on_text_node() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello world"));
        let text_doc_node = DocumentNode::new(arena.new_node(text_node));
        let document = HtmlDocument::new(arena, text_doc_node);

        // act
        let text_node = document.get_html_node(&text_doc_node).unwrap();
        let result = text_node.get_text(&text_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }

    #[test]
    fn html_node_get_text_should_work_on_tag_node_with_one_text_child() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello world"));
        let text_node_id = arena.new_node(text_node);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        let tag_doc_node = DocumentNode::new(tag_node_id);
        tag_node_id.append(text_node_id, &mut arena);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }

    #[test]
    fn html_node_get_text_should_work_on_tag_node_with_two_text_children() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello"));
        let text_node_id = arena.new_node(text_node);

        let text_node2 = HtmlNode::Text(String::from("world"));
        let text_node2_id = arena.new_node(text_node2);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        tag_node_id.append(text_node_id, &mut arena);
        tag_node_id.append(text_node2_id, &mut arena);
        let tag_doc_node = DocumentNode::new(tag_node_id);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }

    #[test]
    fn html_node_get_text_should_ignore_nested_text() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello"));
        let text_node_id = arena.new_node(text_node);

        let text_node2 = HtmlNode::Text(String::from("world"));
        let text_node2_id = arena.new_node(text_node2);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        tag_node_id.append(text_node_id, &mut arena);

        let tag_node2 = HtmlNode::Tag(HtmlTag::new(String::from("tag2")));
        let tag_node2_id = arena.new_node(tag_node2);
        tag_node2_id.append(text_node2_id, &mut arena);
        tag_node_id.append(tag_node2_id, &mut arena);
        let tag_doc_node = DocumentNode::new(tag_node_id);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello", result);
    }

    #[test]
    fn html_node_get_all_text_should_include_nested_text() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello"));
        let text_node_id = arena.new_node(text_node);

        let text_node2 = HtmlNode::Text(String::from("world"));
        let text_node2_id = arena.new_node(text_node2);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        tag_node_id.append(text_node_id, &mut arena);

        let tag_node2 = HtmlNode::Tag(HtmlTag::new(String::from("tag2")));
        let tag_node2_id = arena.new_node(tag_node2);
        tag_node2_id.append(text_node2_id, &mut arena);
        tag_node_id.append(tag_node2_id, &mut arena);
        let tag_doc_node = DocumentNode::new(tag_node_id);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_all_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }
}
