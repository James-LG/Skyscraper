pub mod parse;
mod tokenizer;

use std::error::Error;

use indextree::NodeId;
use crate::html::{HtmlDocument, HtmlNode, HtmlTag};

pub use crate::xpath::parse::parse;

#[derive(Debug, PartialEq)]
pub struct XpathQuery {
    pub identifier: String,
    pub predicates: Vec<XpathPredicate>
}

#[derive(Debug, PartialEq)]
pub enum XpathPredicate {
    GreaterThan { attribute: String, value: u64 },
    LessThan { attribute: String, value: u64 },
    Equals { attribute: String, value: String },
    And(Box<XpathPredicate>, Box<XpathPredicate>),
    Or(Box<XpathPredicate>, Box<XpathPredicate>),
}

impl XpathQuery {
    pub fn new(identifier: String) -> XpathQuery {
        XpathQuery {
            identifier,
            predicates: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum XpathElement {
    SearchRoot,
    SearchAll,
    Query(XpathQuery)
}

pub struct Xpath {
    elements: Vec<XpathElement>
}

impl Xpath {
    pub fn apply(&self, document: &HtmlDocument) -> Result<Vec<NodeId>, Box<dyn Error>> {
        let elements = &mut self.elements.iter();
        let mut matched_nodes: Vec<NodeId> = Vec::new();
        let mut found_nodes: Vec<NodeId> = vec![document.root_key];

        while let Some(element) = elements.next() {
            match element {
                XpathElement::SearchRoot => {
                    if let Some(XpathElement::Query(query)) = elements.next() {
                        matched_nodes = search_root(query, document, &found_nodes);

                        found_nodes = Vec::new();
                        for node_id in &matched_nodes {
                            let mut children = node_id.children(&document.arena).collect();
                            found_nodes.append(&mut children);
                        }
                    }
                },
                XpathElement::SearchAll => {
                    if let Some(XpathElement::Query(query)) = elements.next() {
                        matched_nodes = search_all(query, document, &found_nodes);

                        found_nodes = Vec::new();
                        for node_id in &matched_nodes {
                            let mut children = node_id.children(&document.arena).collect();
                            found_nodes.append(&mut children);
                        }
                    }
                },
                XpathElement::Query(_) => todo!(),
            }
        }

        Ok(matched_nodes)
    }
}

pub fn search_root(query: &XpathQuery, document: &HtmlDocument, nodes: &Vec<NodeId>) -> Vec<NodeId> {
    search_internal(false, query, document, nodes)
}

pub fn search_all(query: &XpathQuery, document: &HtmlDocument, nodes: &Vec<NodeId>) -> Vec<NodeId> {
    search_internal(true, query, document, nodes)
}

fn search_internal(recursive: bool, query: &XpathQuery, document: &HtmlDocument, nodes: &Vec<NodeId>) -> Vec<NodeId> {
    let mut matches = Vec::new();
    
    for node_id in nodes.iter() {
        if let Some(node) = document.arena.get(*node_id) {
            match node.get() {
                HtmlNode::Tag(rtag) => {
                    if rtag.name == query.identifier && is_matching_predicates(query, rtag) {
                        matches.push(*node_id);
                    }

                    if recursive {
                        let children = node_id.children(&document.arena).collect();
                        let mut sub_matches = search_all(query, document, &children);
                        matches.append(&mut sub_matches);
                    }
                },
                HtmlNode::Text(_) => continue,
            }
        }
    }
    
    return matches;
}

fn is_matching_predicates(query: &XpathQuery, rtag: &HtmlTag) -> bool {
    for p in &query.predicates {
        match p {
            XpathPredicate::GreaterThan { .. } => todo!(),
            XpathPredicate::LessThan { .. } => todo!(),
            XpathPredicate::Equals { attribute, value } => {
                if !rtag.attributes.contains_key(attribute) || &rtag.attributes[attribute] != value {
                    return false;
                }
            },
            XpathPredicate::And(_, _) => todo!(),
            XpathPredicate::Or(_, _) => todo!(),
        }
    }

    return true;
}

#[cfg(test)]
mod test {
    use crate::{xpath, html};

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

        let document = html::parse(text).unwrap();

        let query = XpathQuery::new(String::from("root"));
        let result = search_root(&query, &document, &vec![document.root_key]);

        assert_eq!(1, result.len());
        let node = document.arena.get(result[0]).unwrap();

        match node.get() {
            HtmlNode::Tag(tag) => {
                assert_eq!("root", tag.name.as_str());
            },
            HtmlNode::Text(_) => panic!("Expected tag"),
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

        let document = html::parse(text).unwrap();

        let query = XpathQuery::new(String::from("a"));
        let result = search_all(&query, &document, &vec![document.root_key]);

        assert_eq!(3, result.len());

        for r in result {
            let node = document.arena.get(r).unwrap();

            match node.get() {
                HtmlNode::Tag(tag) => {
                    assert_eq!("a", tag.name.as_str());
                },
                HtmlNode::Text(_) => panic!("Expected tag"),
            }
        }
    }

    #[test]
    fn search_all_should_only_select_with_matching_attribute_predicate() {
        let text =r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a/></b>
            <c><b><a hello="world"/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let query = XpathQuery {
            identifier: String::from("a"),
            predicates: vec![
                XpathPredicate::Equals { attribute: String::from("hello"), value: String::from("world") },
            ]
        };

        let result = search_all(&query, &document, &vec![document.root_key]);

        assert_eq!(1, result.len());

        let node = document.arena.get(result[0]).unwrap();

        match node.get() {
            HtmlNode::Tag(tag) => {
                assert_eq!("a", tag.name.as_str());
                assert!(tag.attributes["hello"] == String::from("world"));
            },
            HtmlNode::Text(_) => panic!("Expected tag"),
        }
    }

    #[test]
    fn search_all_should_only_select_with_multiple_matching_attribute_predicate() {
        let text =r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a hello="world"/></b>
            <c><b><a hello="world" foo="bar"/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let query = XpathQuery {
            identifier: String::from("a"),
            predicates: vec![
                XpathPredicate::Equals { attribute: String::from("hello"), value: String::from("world") },
                XpathPredicate::Equals { attribute: String::from("foo"), value: String::from("bar") },
            ]
        };
        
        let result = search_all(&query, &document, &vec![document.root_key]);

        assert_eq!(1, result.len());

        let node = document.arena.get(result[0]).unwrap();

        match node.get() {
            HtmlNode::Tag(tag) => {
                assert_eq!("a", tag.name.as_str());
                assert!(tag.attributes["hello"] == String::from("world"));
            },
            HtmlNode::Text(_) => panic!("Expected tag"),
        }
    }

    #[test]
    fn xpath_apply_works() {
        let text = r###"<!DOCTYPE html>
        <root>
            <node></node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse::parse("/root/node").unwrap();

        let nodes = xpath.apply(&document).unwrap();

        let node = document.arena.get(nodes[0]).unwrap().get();

        match node {
            HtmlNode::Tag(t) => assert_eq!("node", t.name),
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }
}
