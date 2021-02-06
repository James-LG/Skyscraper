#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use indextree::{Arena, Node, NodeId};

mod tokenizer;

use tokenizer::Symbol;

pub struct HtmlTag {
    name: String,
    attributes: HashMap<String, String>,
}

impl HtmlTag {
    fn new(name: String) -> HtmlTag {
        HtmlTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

pub struct HtmlText {
    text: String,
}

pub enum HtmlNode {
    Tag(HtmlTag),
    Text(HtmlText),
}

pub struct HtmlDocument {
    arena: Arena<HtmlNode>,
    root_key: NodeId,
}

pub fn parse(text: &str) -> Result<HtmlDocument, &'static str> {
    let tokens = tokenizer::lex(text)?;

    let mut arena: Arena<HtmlNode> = Arena::new();
    let mut root_key_o: Option<NodeId> = None;
    let mut cur_key_o: Option<NodeId> = None;
    let mut has_tag_open = false;

    let mut tokens = tokens.into_iter();

    while let Some(token) = tokens.next() {
        match token {
            Symbol::StartTag(tag_name) => {
                if has_tag_open {
                    return Err("Start tag encountered before previous tag was closed.");
                }

                has_tag_open = true;

                let node = HtmlNode::Tag(HtmlTag::new(tag_name));
                let node_key = arena.new_node(node);

                if let Some(cur_key) = cur_key_o {
                    cur_key.append(node_key, &mut arena);
                }

                cur_key_o = Some(node_key);
                if let None = root_key_o {
                    root_key_o = cur_key_o;
                }
            },
            Symbol::TagClose => {
                if !has_tag_open {
                    return Err("Tag close encountered before a tag was opened.");
                }
                
                has_tag_open = false;
            },
            Symbol::EndTag(tag_name) => {
                if has_tag_open {
                    return Err("End tag encountered before previous tag was closed.");
                }

                has_tag_open = true;

                let cur_tree_node = try_get_tree_node(cur_key_o, &arena)?;
                match cur_tree_node.get() {
                    HtmlNode::Tag(cur_tag) => {
                        if cur_tag.name != tag_name {
                            return Err("End tag name mismatched open tag name.");
                        }
                    },
                    HtmlNode::Text(_) => return Err("End tag attempted to close a text node."),
                }

                // Set current key to the parent of this tag.
                cur_key_o = cur_tree_node.parent();
            },
            Symbol::Identifier(iden) => {
                if !has_tag_open {
                    return Err("Identifier encountered outside of tag.");
                }

                match tokens.next() {
                    Some(token) => {
                        match token {
                            Symbol::AssignmentSign => {
                                match tokens.next() {
                                    Some(token) => {
                                        match token {
                                            Symbol::Literal(lit) => {
                                                let cur_tree_node = try_get_mut_tree_node(cur_key_o, &mut arena)?;
                                                match cur_tree_node.get_mut() {
                                                    HtmlNode::Tag(tag) => {
                                                        tag.attributes.insert(iden, lit);
                                                    },
                                                    HtmlNode::Text(_) => return Err("Attempted to add attribute to text node."),
                                                }
                                            },
                                            _ => return Err("Expected literal after assignment sign."),
                                        }
                                    },
                                    None => return Err("Unexpected end of tokens."),
                                }
                            },
                            _ => return Err("Expected assignment sign after identifier."),
                        }
                    },
                    None => return Err("Unexpected end of tokens."),
                }
            }
            _ => (),
        }
    }

    if let Some(root_key) = root_key_o {
        return Ok(HtmlDocument {
            arena,
            root_key,
        });
    }

    Err("No root node found.")
}

fn try_get_tree_node(key: Option<NodeId>, arena: &Arena<HtmlNode>) -> Result<&Node<HtmlNode>, &'static str> {
    match key {
        Some(key) => {
            match arena.get(key) {
                Some(node) => Ok(node),
                None => Err("Could not get tree node from arena."),
            }
        },
        None => Err("Unexpected None key."),
    }
    
}

fn try_get_mut_tree_node(key: Option<NodeId>, arena: &mut Arena<HtmlNode>) -> Result<&mut Node<HtmlNode>, &'static str> {
    match key {
        Some(key) => {
            match arena.get_mut(key) {
                Some(node) => Ok(node),
                None => Err("Could not get tree node from arena."),
            }
        },
        None => Err("Unexpected None key."),
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_works() {
        // arrange
        let text = "<html><a class=\"beans\"></a><b><ba></ba></b></html>";

        // act
        let result = parse(text).unwrap();
        let root_tree_node = result.arena.get(result.root_key).unwrap();
        let root_html_node = root_tree_node.get();

        // assert
        match root_html_node {
            // Root node should be a Tag.
            HtmlNode::Tag(tag) => {
                assert_eq!(String::from("html"), tag.name);

                let mut children = result.root_key.children(&result.arena);

                // First child node should also be a Tag.
                let child1_key = children.next().unwrap();
                let child1_node = result.arena.get(child1_key).unwrap().get();
                match child1_node {
                    HtmlNode::Tag(tag) => {
                        assert_eq!(String::from("a"), tag.name);

                        assert_eq!("beans", tag.attributes.get("class").unwrap());
                    },
                    _ => assert!(matches!(child1_node, HtmlNode::Tag(_))),
                }

                // Second child node should also be a Tag.
                let child2_key = children.next().unwrap();
                let child2_node = result.arena.get(child2_key).unwrap().get();
                match child2_node {
                    HtmlNode::Tag(tag) => {
                        assert_eq!(String::from("b"), tag.name);

                        let mut children = child2_key.children(&result.arena);

                        // First child of `b` node should also be a Tag.
                        let child1_key = children.next().unwrap();
                        let child1_node = result.arena.get(child1_key).unwrap().get();
                        match child1_node {
                            HtmlNode::Tag(tag) => assert_eq!(String::from("ba"), tag.name),
                            _ => assert!(matches!(child1_node, HtmlNode::Tag(_))),
                        }
                    },
                    _ => assert!(matches!(child2_node, HtmlNode::Tag(_))),
                }
            },
            _ => assert!(matches!(root_html_node, HtmlNode::Tag(_))),
        }
    }
}
