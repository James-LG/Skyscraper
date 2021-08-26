mod tokenizer;

use std::{collections::HashMap, error::Error};

use indextree::{Node, NodeId};

use crate::{RDocument, RNode, html};

use self::tokenizer::Symbol;

#[derive(Debug)]
#[derive(PartialEq)]
struct XpathQuery {
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
enum XpathElement {
    SearchRoot,
    SearchAll,
    Element(XpathQuery)
}

pub struct Xpath {
    elements: Vec<XpathElement>
}

impl Xpath {
    pub fn apply(&self, document: &RDocument) -> Result<Vec<NodeId>, Box<dyn Error>> {
        // Xpath::apply_internal(document, document.root_key, &self.elements)
        Ok(Vec::new())
    }

    fn apply_internal(document: &RDocument, root: NodeId, symbols: &Vec<Symbol>) -> Result<Vec<NodeId>, Box<dyn Error>> {
        Ok(Vec::new())
    }

    fn search_identifier(document: &RDocument, root: NodeId, identifier: &String) -> Vec<NodeId> {
        let matched_node_ids = Vec::new();
        
        for child in root.children(&document.arena) {
            if let Some(child_node) = document.arena.get(child) {
                match child_node.get() {
                    RNode::Tag(tag) => todo!(),
                    RNode::Text(text) => todo!()
                }
            }
        }
        
        matched_node_ids
    }
}

pub fn parse(text: &str) -> Result<Xpath, Box<dyn Error>> {
    let symbols = tokenizer::lex(text)?;
    let mut elements: Vec<XpathElement> = Vec::new();

    for symbol in symbols {
        match symbol {
            Symbol::Slash => elements.push(XpathElement::SearchRoot),
            Symbol::DoubleSlash => elements.push(XpathElement::SearchAll),
            Symbol::OpenSquareBracket => todo!(),
            Symbol::CloseSquareBracket => todo!(),
            Symbol::OpenBracket => todo!(),
            Symbol::CloseBracket => todo!(),
            Symbol::Wildcard => todo!(),
            Symbol::Dot => todo!(),
            Symbol::DoubleDot => todo!(),
            Symbol::AssignmentSign => todo!(),
            Symbol::AtSign => todo!(),
            Symbol::MinusSign => todo!(),
            Symbol::AddSign => todo!(),
            Symbol::GreaterThanSign => todo!(),
            Symbol::LessThanSign => todo!(),
            Symbol::Identifier(identifier) => elements.push(XpathElement::Element(XpathQuery::new(identifier))),
            Symbol::Text(_) => todo!(),
            Symbol::Number(_) => todo!(),
        }
    }

    Ok(Xpath { elements })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_works() {
        let text = "//book/title";

        let result = parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Element(XpathQuery::new(String::from("book"))),
            XpathElement::SearchRoot,
            XpathElement::Element(XpathQuery::new(String::from("title")))
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result.elements) {
            assert_eq!(e, r);
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

        let xpath = parse("/root/node").unwrap();

        let nodes = xpath.apply(&document).unwrap();

        let node = document.arena.get(nodes[0]).unwrap().get();

        match node {
            RNode::Tag(t) => assert_eq!("node", t.name),
            RNode::Text(_) => panic!("expected tag, got text instead")
        }
    }
}