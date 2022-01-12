pub mod parse;
mod tokenizer;

use indextree::NodeId;
use thiserror::Error;
use crate::html::{HtmlDocument, HtmlNode, HtmlTag};

pub use crate::xpath::parse::parse;

#[derive(Debug, PartialEq)]
pub struct XpathQuery {
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
    pub fn new() -> XpathQuery {
        XpathQuery {
            predicates: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum XpathElement {
    SearchRoot,
    SearchAll,
    Tag(String),
    Query(XpathQuery),
    Index(usize)
}

pub struct Xpath {
    elements: Vec<XpathElement>
}

#[derive(Error, Debug)]
pub enum ApplyError {
}

impl Xpath {
    /// Search the given HTML document according to this Xpath expression.
    pub fn apply(&self, document: &HtmlDocument) -> Result<Vec<NodeId>, ApplyError> {
        let elements = &mut self.elements.iter();
        let mut matched_nodes: Vec<NodeId> = Vec::new(); // The nodes matched by the search query
        let mut found_nodes: Vec<NodeId> = vec![document.root_key]; // The list of nodes to search in (typically children of matched nodes)

        let mut is_root_search = true; // If false, then search all
        let mut cur_query: Option<&XpathQuery> = None;
        let mut cur_tag_name: Option<&String> = None; 
        let mut cur_index: Option<usize> = None;

        while let Some(element) = elements.next() {
            match element {
                XpathElement::SearchRoot | XpathElement::SearchAll => {
                    // Perform the previously defined search now that it has ended
                    perform_search(cur_tag_name, cur_query, cur_index, is_root_search, &mut matched_nodes, document, &mut found_nodes);

                    // Set parameters for next iteration
                    is_root_search = matches!(element, XpathElement::SearchRoot);
                    cur_query = None;
                    cur_tag_name = None;
                    cur_index = None;
                },
                XpathElement::Tag(identifier) => cur_tag_name = Some(identifier),
                XpathElement::Query(query) => cur_query = Some(query),
                XpathElement::Index(i) => cur_index = Some(*i),
            }
        }

        // Perform the last search now that the entire xpath expression has finished
        perform_search(cur_tag_name, cur_query, cur_index, is_root_search, &mut matched_nodes, document, &mut found_nodes);
        Ok(matched_nodes)
    }
}

fn perform_search(
    cur_tag_name: Option<&String>,
    cur_query: Option<&XpathQuery>,
    cur_index: Option<usize>,
    is_root_search: bool,
    matched_nodes: &mut Vec<NodeId>,
    document: &HtmlDocument,
    found_nodes: &mut Vec<NodeId>) {
    if let Some(tag_name) = cur_tag_name {
        if let Some(query) = cur_query {
            if is_root_search {
                *matched_nodes = search_root(tag_name, query, document, &*found_nodes);
            
            } else {
                *matched_nodes = search_all(tag_name, query, document, &*found_nodes);
            }
        } else {
            let query = XpathQuery::new();
            if is_root_search {
                *matched_nodes = search_root(tag_name, &query, document, &*found_nodes);
            
            } else {
                *matched_nodes = search_all(tag_name, &query, document, &*found_nodes);
            }
        }

        // Apply indexing if required
        if let Some(i) = cur_index {
            let indexed_node = matched_nodes[i];
            matched_nodes.retain(|node| *node == indexed_node);
        }
    
        *found_nodes = get_all_children(&document, &*matched_nodes);
    }
}

fn get_all_children(document: &HtmlDocument, matched_nodes: &Vec<NodeId>) -> Vec<NodeId> {
    let mut found_nodes: Vec<NodeId> = Vec::new();
    for node_id in matched_nodes {
        let mut children = node_id.children(&document.arena).collect();
        found_nodes.append(&mut children);
    }

    return found_nodes;
}

/// Search for an HTML tag matching the given name and query in the given list of nodes.
pub fn search_root(tag_name: &String, query: &XpathQuery, document: &HtmlDocument, nodes: &Vec<NodeId>) -> Vec<NodeId> {
    search_internal(false, tag_name, query, document, nodes)
}

/// Search for an HTML tag matching the given name and query in the given list of nodes or any children of those nodes.
pub fn search_all(tag_name: &String, query: &XpathQuery, document: &HtmlDocument, nodes: &Vec<NodeId>) -> Vec<NodeId> {
    search_internal(true, tag_name, query, document, nodes)
}

fn search_internal(recursive: bool, tag_name: &String, query: &XpathQuery, document: &HtmlDocument, nodes: &Vec<NodeId>) -> Vec<NodeId> {
    let mut matches = Vec::new();
    
    for node_id in nodes.iter() {
        if let Some(node) = document.arena.get(*node_id) {
            match node.get() {
                HtmlNode::Tag(rtag) => {
                    if &rtag.name == tag_name && is_matching_predicates(query, rtag) {
                        matches.push(*node_id);
                    }

                    if recursive {
                        let children = node_id.children(&document.arena).collect();
                        let mut sub_matches = search_all(tag_name, query, document, &children);
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

        let query = XpathQuery::new();
        let result = search_root(&String::from("root"), &query, &document, &vec![document.root_key]);

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

        let query = XpathQuery::new();
        let result = search_all(&String::from("a"), &query, &document, &vec![document.root_key]);

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
            predicates: vec![
                XpathPredicate::Equals { attribute: String::from("hello"), value: String::from("world") },
            ]
        };

        let result = search_all(&String::from("a"), &query, &document, &vec![document.root_key]);

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
            predicates: vec![
                XpathPredicate::Equals { attribute: String::from("hello"), value: String::from("world") },
                XpathPredicate::Equals { attribute: String::from("foo"), value: String::from("bar") },
            ]
        };
        
        let result = search_all(&String::from("a"), &query, &document, &vec![document.root_key]);

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
        // arrange
        let text = r###"<!DOCTYPE html>
        <root>
            <node></node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("/root/node").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(1, nodes.len());
        let node = document.arena.get(nodes[0]).unwrap().get();

        match node {
            HtmlNode::Tag(t) => assert_eq!("node", t.name),
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }

    #[test]
    fn xpath_apply_handles_indexes() {
        // arrange
        let text = r###"<!DOCTYPE html>
        <root>
            <node>0</node>
            <node>1</node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("/root/node[1]").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(1, nodes.len());
        let node_id = nodes[0];
        let node = document.arena.get(node_id).unwrap().get();

        match node {
            HtmlNode::Tag(t) => {
                assert_eq!("node", t.name);

                let children: Vec<NodeId> = node_id.children(&document.arena).collect();
                assert_eq!(1, children.len());

                let child_node = document.arena.get(children[0]).unwrap().get();
                match child_node {
                    HtmlNode::Tag(_) => panic!("expected child text, got tag instead"),
                    HtmlNode::Text(text) => assert_eq!(&String::from("1"), text),
                }
            },
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }
}
