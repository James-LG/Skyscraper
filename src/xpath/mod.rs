pub mod parse;
mod tokenizer;

use std::{ops::Index, iter::FromIterator};

use indexmap::{IndexSet, indexset};
use thiserror::Error;
use crate::html::{HtmlDocument, HtmlNode, HtmlTag, DocumentNode};

pub use crate::xpath::parse::parse;

#[derive(Debug, PartialEq, Default)]
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
    Index(usize),
    Axis(XpathAxes)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum XpathAxes {
    Child,
    DescendantOrSelf,
    Parent
}

impl Default for XpathAxes {
    fn default() -> Self { XpathAxes::Child }
}

#[derive(Debug, PartialEq)]
pub enum XpathSearchNodeType {
    Element(String),
    Any
}

impl Default for XpathSearchNodeType {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct XpathSearchItem {
    pub axis: XpathAxes,
    pub search_node_type: XpathSearchNodeType,
    pub query: Option<XpathQuery>,
    pub index: Option<usize>
}

pub struct Xpath {
    items: Vec<XpathSearchItem>
}

#[derive(Error, Debug)]
pub enum ApplyError {
    #[error("missing trailing tag (e.g. `//<TAG_HERE>`)")]
    TagMissing
}

#[derive(Default)]
pub struct DocumentNodeSet {
    index_set: IndexSet<DocumentNode>
}

impl DocumentNodeSet {
    pub fn new() -> DocumentNodeSet {
        DocumentNodeSet { index_set: Default::default() }
    }

    pub fn from(index_set: IndexSet<DocumentNode>) -> DocumentNodeSet {
        DocumentNodeSet { index_set }
    }

    pub fn insert(&mut self, value: DocumentNode) -> bool {
        self.index_set.insert(value)
    }

    pub fn pop(&mut self) -> Option<DocumentNode> {
        self.index_set.pop()
    }

    pub fn append(&mut self, other: DocumentNodeSet) {
        for node in other.into_iter() {
            self.insert(node);
        }
    }

    pub fn insert_all(&mut self, vec: impl Iterator<Item = DocumentNode>) {
        for node in vec {
            self.insert(node);
        }
    }

    pub fn retain<F>(&mut self, f: F)
    where F: FnMut(&DocumentNode) -> bool {
        self.index_set.retain(f)
    }

    pub fn iter(&self) -> indexmap::set::Iter<DocumentNode> {
        self.index_set.iter()
    }

    pub fn contains(&self, node: &DocumentNode) -> bool {
        self.index_set.contains(node)
    }

    pub fn len(&self) -> usize {
        self.index_set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for DocumentNodeSet {
    type Item = DocumentNode;

    type IntoIter = indexmap::set::IntoIter<DocumentNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.index_set.into_iter()
    }
}

impl<'a> IntoIterator for &'a DocumentNodeSet {
    type Item = &'a DocumentNode;

    type IntoIter = indexmap::set::Iter<'a, DocumentNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.index_set.iter()
    }
}

impl Index<usize> for DocumentNodeSet {
    type Output = DocumentNode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.index_set[index]
    }
}

impl FromIterator<DocumentNode> for DocumentNodeSet {
    fn from_iter<T: IntoIterator<Item = DocumentNode>>(iter: T) -> Self {
        let index_set = IndexSet::from_iter(iter);
        DocumentNodeSet::from(index_set)
    }
}

impl Xpath {
    /// Search the given HTML document according to this Xpath expression.
    pub fn apply(&self, document: &HtmlDocument) -> Result<Vec<DocumentNode>, ApplyError> {
        let searchable_nodes: DocumentNodeSet = DocumentNodeSet::from(indexset![document.root_key]);
        self.internal_apply(document, searchable_nodes)
    }

    /// Search the the descendents of the given node in the given HTML document
    /// according to this Xpath expression.
    pub fn apply_to_node(&self, document: &HtmlDocument, doc_node: DocumentNode) -> Result<Vec<DocumentNode>, ApplyError> {
        let searchable_nodes: DocumentNodeSet = get_all_children(document, &DocumentNodeSet::from(indexset![doc_node]));
        self.internal_apply(document, searchable_nodes)
    }

    fn internal_apply(&self, document: &HtmlDocument, searchable_nodes: DocumentNodeSet) -> Result<Vec<DocumentNode>, ApplyError> {
        let items = &mut self.items.iter();
        let mut matched_nodes: DocumentNodeSet = searchable_nodes; // The nodes matched by the search query

        let mut is_first_search = true;
        while let Some(search_item) = items.next() {
            apply_axis(is_first_search, document, &search_item.axis, &mut matched_nodes);
            matched_nodes = search(search_item, document, &matched_nodes)?;

            // Apply indexing if required
            if let Some(i) = search_item.index {
                let indexed_node = matched_nodes[i];
                matched_nodes.retain(|node| *node == indexed_node);
            }

            is_first_search = false;
        }

        Ok(matched_nodes.into_iter().collect())
    }
}

fn apply_axis(is_first_search: bool, document: &HtmlDocument, axis: &XpathAxes, searchable_nodes: &mut DocumentNodeSet) {
    match axis {
        XpathAxes::Child => {
            // Child axis is implied for first search of Xpath expression.
            // For example when given the root node we should not move down to a child
            // on the first search so that we can match on the root node itself rather than
            // only on its children. E.g. /root/something
            if !is_first_search {
                *searchable_nodes = get_all_children(&document, &searchable_nodes);
            }
        },
        XpathAxes::DescendantOrSelf => {
            *searchable_nodes = get_all_descendants_or_self(document, searchable_nodes);
        },
        XpathAxes::Parent => {
            *searchable_nodes = get_all_parents(&document, &searchable_nodes);
        },
    }
}

/// Get all the children for all the given matched nodes.
fn get_all_children(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut child_nodes = DocumentNodeSet::new();
    for node_id in matched_nodes {
        let children = node_id.children(&document);
        child_nodes.insert_all(children);
    }

    child_nodes
}

/// Get all descendants and self for all the given matched nodes.
fn get_all_descendants_or_self(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut descendant_or_self_nodes: DocumentNodeSet = DocumentNodeSet::new();

    for node_id in matched_nodes {
        descendant_or_self_nodes.insert(*node_id);
        let mut children: DocumentNodeSet = node_id.children(&document).collect();
        if !children.is_empty() {
            children.append(get_all_descendants_or_self(document, &children))
        }
        descendant_or_self_nodes.insert_all(children.into_iter());
    }

    descendant_or_self_nodes
}

/// Get all the parents for all the given matched nodes.
fn get_all_parents(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut parent_nodes = DocumentNodeSet::new();
    for node_id in matched_nodes {
        if let Some(parent) = node_id.parent(&document) {
            parent_nodes.insert(parent);
        }
    }

    parent_nodes
}

/// Search for an HTML tag matching the given search parameters in the given list of nodes.
pub fn search(search_params: &XpathSearchItem, document: &HtmlDocument, searchable_nodes: &DocumentNodeSet) -> Result<DocumentNodeSet, ApplyError> {
    let mut matches = DocumentNodeSet::new();

    for node_id in searchable_nodes.iter() {
        if let Some(node) = document.get_html_node(node_id) {
            match node {
                HtmlNode::Tag(rtag) => {
                    match &search_params.search_node_type {
                        XpathSearchNodeType::Element(tag_name) => {
                            if &rtag.name == tag_name {
                                if let Some(query) = &search_params.query {
                                    if is_matching_predicates(query, rtag) {
                                        matches.insert(*node_id);
                                    }
                                } else {
                                    matches.insert(*node_id);
                                }
                            }
                        },
                        XpathSearchNodeType::Any => {
                            matches.insert(*node_id);
                        },
                    }
                    
                },
                HtmlNode::Text(_) => continue,
            }
        }
    }

    // Some axes require recursion
    if !searchable_nodes.is_empty() {
        let sub_matches = match search_params.axis {
            XpathAxes::DescendantOrSelf => {
                let children = get_all_children(document, searchable_nodes);
                if !children.is_empty() {
                    Some(search(search_params, document, &children)?)
                } else {
                    None
                }
            },
            _ => None
        };

        if let Some(sub_matches) = sub_matches {
            for sub_match in sub_matches {
                matches.insert(sub_match);
            }
        }
    }

    Ok(matches)
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

    true
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

        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("root")),
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::from(indexset![document.root_key])).unwrap();

        assert_eq!(1, result.len());
        let node = document.get_html_node(&result[0]).unwrap();

        match node {
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

        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::from(indexset![document.root_key])).unwrap();

        assert_eq!(3, result.len());

        for r in result {
            let node = document.get_html_node(&r).unwrap();

            match node {
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
        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            query: Some(query),
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::from(indexset![document.root_key])).unwrap();

        assert_eq!(1, result.len());

        let node = document.get_html_node(&result[0]).unwrap();

        match node {
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
        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            query: Some(query),
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::from(indexset![document.root_key])).unwrap();

        assert_eq!(1, result.len());

        let node = document.get_html_node(&result[0]).unwrap();

        match node {
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
        let node = document.get_html_node(&nodes[0]).unwrap();

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
        let node = document.get_html_node(&node_id).unwrap();

        match node {
            HtmlNode::Tag(t) => {
                assert_eq!("node", t.name);

                let children: Vec<DocumentNode> = node_id.children(&document).collect();
                assert_eq!(1, children.len());

                let child_node = document.get_html_node(&children[0]).unwrap();
                match child_node {
                    HtmlNode::Tag(_) => panic!("expected child text, got tag instead"),
                    HtmlNode::Text(text) => assert_eq!(&String::from("1"), text),
                }
            },
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }

    #[test]
    fn xpath_apply_handles_parent_axis() {
        // arrange
        let text = r###"<!DOCTYPE html>
        <root>
            <node id='1'>
                <apple/>
            </node>
            <node id='2'>
                <apple/>
            </node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("//apple/parent::node").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(2, nodes.len());

        let node_id = nodes[0];
        let node = document.get_html_node(&node_id).unwrap();

        match node {
            HtmlNode::Tag(t) => {
                assert_eq!(String::from("1"), t.attributes["id"])
            },
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }

        let node_id = nodes[1];
        let node = document.get_html_node(&node_id).unwrap();

        match node {
            HtmlNode::Tag(t) => {
                assert_eq!(String::from("2"), t.attributes["id"])
            },
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }

    #[test]
    fn xpath_apply_handles_single_expression_query() {
        // arrange
        let text = r###"<!DOCTYPE html>
        <root>
            <node>0</node>
            <node>1</node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("//node").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(2, nodes.len());
        let node1_text = nodes[0].get_text(&document).unwrap();

        assert_eq!("0", node1_text);

        let node2_text = nodes[1].get_text(&document).unwrap();

        assert_eq!("1", node2_text);
    }

    #[test]
    fn xpath_apply_to_node_matches_only_given_node_descendents() {
        // STAGE 1: Get second node
        // arrange
        let text = r###"<!DOCTYPE html>
        <root>
            <node id='1'>
                <div class='duplicate'>1</div>
            </node>
            <node id='2'>
                <div class='duplicate'>2</div>
            </node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("/root/node[@id='2']").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(1, nodes.len());
        let doc_node = nodes[0];
        let node = document.get_html_node(&doc_node).unwrap();

        match node {
            HtmlNode::Tag(t) => assert_eq!("node", t.name),
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }

        // STAGE 2: Apply xpath to node 2, and check correct div was retrieved.
        // arrange
        let xpath = xpath::parse("/div[@class='duplicate']").unwrap();

        // act
        let nodes = xpath.apply_to_node(&document, doc_node).unwrap();

        // assert
        assert_eq!(1, nodes.len());
        let doc_node = nodes[0];
        let node = document.get_html_node(&doc_node).unwrap();

        match node {
            HtmlNode::Tag(t) => {
                assert_eq!("div", t.name);
                assert_eq!("2", t.get_text(&doc_node, &document).unwrap());
            },
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }
}
