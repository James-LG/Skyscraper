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
    pub fn get_text(&self, node_id: NodeId, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(node_id, document, false)
    }

    /// Gets all HtmlNode::Text children and concatenates them into a single string separated
    /// by a space character.
    pub fn get_all_text(&self, node_id: NodeId, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(node_id, document, true)
    }

    fn internal_get_text(&self, node_id: NodeId, document: &HtmlDocument, recurse: bool) -> Option<String> {
        let mut o_text: Option<String> = None;
        let children = node_id.children(&document.arena);

        // Iterate through this tag's children
        for child in children {
            let child_node = document.arena.get(child);
            if let Some(child_node) = child_node {
                let child_html_node = child_node.get();
                match child_html_node {
                    HtmlNode::Text(text) => {
                        // If the child is a text, simply append its text.
                        o_text = Some(HtmlTag::append_text(o_text, text.to_string()));
                    },
                    HtmlNode::Tag(_) => {
                        // If the child is a tag, only append its text if recurse=true was passed,
                        // otherwise skip this node.
                        if recurse {
                            let o_child_text = child_html_node.internal_get_text(child, &document, true);
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
    pub fn get_text(&self, node_id: NodeId, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(node_id, document, false)
    }

    /// Gets all HtmlNode::Text children and concatenates them into a single string separated
    /// by a space character.
    pub fn get_all_text(&self, node_id: NodeId, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(node_id, document, true)
    }

    /// Gets any direct HtmlNode::Text children and concatenates them into a single string
    /// separated by new line characters.
    fn internal_get_text(&self, node_id: NodeId, document: &HtmlDocument, recurse: bool) -> Option<String> {
        match self {
            HtmlNode::Tag(tag) => {
                if recurse {
                    tag.get_all_text(node_id, document)
                } else {
                    tag.get_text(node_id, document)
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
    pub arena: Arena<HtmlNode>,
    pub root_key: NodeId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_node_get_text_should_work_on_text_node() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello world"));
        let text_node_id = arena.new_node(text_node);
        let document = HtmlDocument { arena, root_key: text_node_id };

        // act
        let text_node = document.arena.get(text_node_id).unwrap().get();
        let result = text_node.get_text(text_node_id, &document).unwrap();

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
        tag_node_id.append(text_node_id, &mut arena);

        let document = HtmlDocument { arena, root_key: tag_node_id };

        // act
        let tag_node = document.arena.get(tag_node_id).unwrap().get();
        let result = tag_node.get_text(tag_node_id, &document).unwrap();

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

        let document = HtmlDocument { arena, root_key: tag_node_id };

        // act
        let tag_node = document.arena.get(tag_node_id).unwrap().get();
        let result = tag_node.get_text(tag_node_id, &document).unwrap();

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

        let document = HtmlDocument { arena, root_key: tag_node_id };

        // act
        let tag_node = document.arena.get(tag_node_id).unwrap().get();
        let result = tag_node.get_text(tag_node_id, &document).unwrap();

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

        let document = HtmlDocument { arena, root_key: tag_node_id };

        // act
        let tag_node = document.arena.get(tag_node_id).unwrap().get();
        let result = tag_node.get_all_text(tag_node_id, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }
}
