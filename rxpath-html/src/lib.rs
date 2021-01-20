#[macro_use]
extern crate lazy_static;

use slotmap::{SlotMap, DefaultKey};

mod tokenizer;

use tokenizer::Symbol;
use rxpath::{Element, Document, Node, Text};

pub type HtmlNode = Node<HtmlElement, HtmlText>;

#[derive(Clone)]
pub struct HtmlText {
    parent_key: DefaultKey,
    value: String,
}

impl Text for HtmlText {
    fn get_value(&self) -> &String {
        &self.value
    }
}

#[derive(Clone)]
pub struct HtmlElement {
    parent_key: Option<DefaultKey>,
    name: String,
    attributes: Vec<String>,
    child_keys: Vec<DefaultKey>,
}

impl HtmlElement {
    fn new(parent_key: Option<DefaultKey>, name: String) -> HtmlElement {
        HtmlElement {
            parent_key,
            name,
            attributes: Vec::new(),
            child_keys: Vec::new(),
        }
    }
}

impl Element for HtmlElement {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_attributes(&self) -> &Vec<String> {
        &self.attributes
    }
}

pub struct HtmlDocument {
    slotmap: SlotMap<DefaultKey, HtmlNode>,
    root_node_key: DefaultKey,
}

impl Document for HtmlDocument {
    type TElem = HtmlElement;
    type TText = HtmlText;

    fn get_root(&self) -> &HtmlNode {
        &self.slotmap[self.root_node_key]
    }

    fn get_children_of(&self, element: &HtmlElement) -> Vec<&HtmlNode> {
        element.child_keys.iter().map(|child_key| {
            self.slotmap.get(*child_key).expect("Slotmap key no longer pointing to value")
        }).collect()
    }

    fn get_parent_of(&self, node: &HtmlNode) -> Option<&HtmlNode> {
        match node {
            Node::Element(el) => {
                match el.parent_key {
                    Some(parent_key) => self.slotmap.get(parent_key),
                    None => None,
                }
            },
            Node::Text(txt) => self.slotmap.get(txt.parent_key),
        }
    }
}

pub fn parse(text: &str) -> Result<HtmlDocument, &'static str> {
    let tokens = tokenizer::lex(text)?;

    let mut root: Option<DefaultKey> = None;
    let mut cur_node: Option<DefaultKey> = None;

    let mut slotmap: SlotMap<DefaultKey, HtmlNode> = SlotMap::new();

    for token in tokens {
        match token {
            Symbol::StartTag(name) => {
                let parent_key = match cur_node {
                    Some(ref cur_pair) => Some(cur_pair.clone()),
                    None => None,
                };
                let cur_node_val = HtmlNode::Element(HtmlElement::new(parent_key, name));
                let cur_node_key = slotmap.insert(cur_node_val);
                cur_node = Some(cur_node_key);

                if let None = root {
                    root = cur_node;
                }
            },
            _ => continue,
        }
    }

    match root {
        Some(node) => {
            Ok(HtmlDocument {
                slotmap,
                root_node_key: node,
            })
        },
        None => Err("No root node found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_works() {
        // arrange
        let text = "<html><a></a></html>";

        // act
        let result = parse(text).unwrap();

        // assert
        match result.get_root() {
            HtmlNode::Element(element) => {
                if element.name != "html" {
                    panic!();
                }
            }
            _ => panic!(),
        }
    }
}
