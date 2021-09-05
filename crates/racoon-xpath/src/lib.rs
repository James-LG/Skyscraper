pub mod parse;
mod tokenizer;

use std::{collections::HashMap, error::Error};

use indextree::NodeId;
use racoon_core::RDocument;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct XpathQuery {
    identifier: String,
    attributes: HashMap<String, String>
}

impl XpathQuery {
    pub fn new(identifier: String) -> XpathQuery {
        XpathQuery {
            identifier,
            attributes: HashMap::new(),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum XpathElement {
    SearchRoot,
    SearchAll,
    Element(XpathQuery)
}

pub struct Xpath {
    elements: Vec<XpathElement>
}

impl Xpath {
    pub fn apply(&self, document: &RDocument) -> Result<Vec<NodeId>, Box<dyn Error>> {
        for element in &self.elements {
            match element {
                XpathElement::SearchRoot => todo!(),
                XpathElement::SearchAll => todo!(),
                XpathElement::Element(_) => todo!(),
            }
        }
        Ok(Vec::new())
    }
}

pub fn search_root(query: &XpathQuery, document: &RDocument, cur_node: NodeId) -> Vec<NodeId> {
    search_internal(false, query, document, cur_node)
}

pub fn search_all(query: &XpathQuery, document: &RDocument, cur_node: NodeId) -> Vec<NodeId> {
    search_internal(true, query, document, cur_node)
}

fn search_internal(recursive: bool, query: &XpathQuery, document: &RDocument, cur_node: NodeId) -> Vec<NodeId> {
    let mut matches = Vec::new();
    
    for node_id in cur_node.children(&document.arena) {
        if let Some(node) = document.arena.get(node_id) {
            match node.get() {
                racoon_core::RNode::Tag(rtag) => {
                    if rtag.name == query.identifier {
                        matches.push(node_id);
                    }

                    if recursive {
                        let mut sub_matches = search_all(query, document, node_id);
                        matches.append(&mut sub_matches);
                    }
                },
                racoon_core::RNode::Text(_) => continue,
            }
        }
    }
    
    return matches;
}

#[cfg(test)]
mod test {
    use racoon_core::{RNode, RTag};

    use super::*;

    #[test]
    fn search_root_works() {
        let text =r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a/></b>
            <c><b><a/></b></c>
        </root>
        "###;

        let document = racoon_html::parse(text).unwrap();

        let query = XpathQuery::new(String::from("a"));
        let result = search_root(&query, &document, document.root_key);

        assert!(result.len() == 1);
        let node = document.arena.get(result[0]).unwrap();

        match node.get() {
            RNode::Tag(tag) => {
                assert_eq!("a", tag.name.as_str());
            },
            RNode::Text(_) => panic!("Expected tag"),
        }
    }

    #[test]
    fn search_all_works() {
        let text =r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a/></b>
            <c><b><a/></b></c>
        </root>
        "###;

        let document = racoon_html::parse(text).unwrap();

        let query = XpathQuery::new(String::from("a"));
        let result = search_all(&query, &document, document.root_key);

        assert!(result.len() == 3);

        for r in result {
            let node = document.arena.get(r).unwrap();

            match node.get() {
                RNode::Tag(tag) => {
                    assert_eq!("a", tag.name.as_str());
                },
                RNode::Text(_) => panic!("Expected tag"),
            }
        }
        
    }

    #[test]
    fn xpath_apply_works() {
        let text = r###"<!DOCTYPE html>
        <root>
            <node></node>
        </root>
        "###;

        let document = racoon_html::parse(text).unwrap();

        let xpath = crate::parse::parse("/root/node").unwrap();

        let nodes = xpath.apply(&document).unwrap();

        let node = document.arena.get(nodes[0]).unwrap().get();

        match node {
            RNode::Tag(t) => assert_eq!("node", t.name),
            RNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }
}
