pub mod parse;
mod tokenizer;

use std::ops::Index;

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
    TreeSelector {
        axis: XpathAxes,
        tag_name: String
    }
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
struct WeakXpathSearchParams<'a> {
    is_first_search: bool,
    tag_name: Option<&'a str>,
    query: &'a XpathQuery,
    index: Option<usize>,
    axis: XpathAxes,
}

#[derive(Debug, PartialEq)]
pub struct XpathSearchParams<'a> {
    tag_name: &'a str,
    query: &'a XpathQuery,
    index: Option<usize>,
    axis: XpathAxes,
}

pub struct Xpath {
    elements: Vec<XpathElement>
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

    pub fn insert_all(&mut self, vec: Vec<DocumentNode>) {
        for node in vec.into_iter() {
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
        let elements = &mut self.elements.iter();
        let mut matched_nodes: DocumentNodeSet = DocumentNodeSet::new(); // The nodes matched by the search query
        let mut searchable_nodes: DocumentNodeSet = searchable_nodes; // The list of nodes to search in (typically children of matched nodes)

        let default_query = XpathQuery::new();

        let mut search_params = WeakXpathSearchParams {
            is_first_search: true,
            query: &default_query,
            tag_name: None,
            index: None,
            axis: XpathAxes::Child
        };

        let mut first_element = true;
        while let Some(element) = elements.next() {
            match element {
                XpathElement::SearchRoot | XpathElement::SearchAll => {
                    if !first_element {
                        // Perform the previously defined search now that it has ended
                        perform_search(&search_params, &mut matched_nodes, document, &mut searchable_nodes)?;
                        search_params.is_first_search = false;
                    }

                    // Set parameters for next iteration
                    search_params.axis = if matches!(element, XpathElement::SearchRoot) {
                        XpathAxes::Child
                    } else {
                        XpathAxes::DescendantOrSelf
                    };
                    search_params.query = &default_query;
                    search_params.tag_name = None;
                    search_params.index = None;
                },
                XpathElement::Tag(identifier) => search_params.tag_name = Some(identifier),
                XpathElement::Query(query) => search_params.query = query,
                XpathElement::Index(i) => search_params.index = Some(*i),
                XpathElement::TreeSelector { axis, tag_name } => {
                    search_params.axis = *axis;
                    search_params.tag_name = Some(tag_name)
                }
            }
            first_element = false;
        }

        // Perform the last search now that the entire xpath expression has finished
        perform_search(&search_params, &mut matched_nodes, document, &mut searchable_nodes)?;
        Ok(matched_nodes.into_iter().collect())
    }
}

fn perform_search(
    weak_search: &WeakXpathSearchParams,
    matched_nodes: &mut DocumentNodeSet,
    document: &HtmlDocument,
    searchable_nodes: &mut DocumentNodeSet) -> Result<(), ApplyError> {

    let tag_name = weak_search.tag_name.ok_or_else(|| ApplyError::TagMissing)?;

    let search_params = XpathSearchParams {
        tag_name,
        query: weak_search.query,
        axis: weak_search.axis,
        index: weak_search.index,
    };

    if !weak_search.is_first_search {
        match search_params.axis {
            XpathAxes::Child => {
                *searchable_nodes = get_all_children(&document, &matched_nodes);
            },
            XpathAxes::DescendantOrSelf => {
                *searchable_nodes = get_all_descendants_or_self(document, &matched_nodes);
            }
            XpathAxes::Parent => {
                *searchable_nodes = get_all_parents(&document, &matched_nodes);
            }
        }
    }

    *matched_nodes = search(&search_params, document, &searchable_nodes)?;

    // Apply indexing if required
    if let Some(i) = search_params.index {
        let indexed_node = matched_nodes[i];
        matched_nodes.retain(|node| *node == indexed_node);
    }

    Ok(())
}

fn apply_axis(axis: &XpathAxes, searchable_nodes: &mut DocumentNodeSet) {
    match axis {
        XpathAxes::Child => todo!(),
        XpathAxes::DescendantOrSelf => todo!(),
        XpathAxes::Parent => todo!(),
    }
}

/// Get all the children for all the given matched nodes.
fn get_all_children(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut child_nodes = DocumentNodeSet::new();
    for node_id in matched_nodes {
        let children: Vec<DocumentNode> = node_id.children(&document).collect();
        child_nodes.insert_all(children);
    }

    child_nodes
}

/// Get all descendants and self for all the given matched nodes.
fn get_all_descendants_or_self(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut descendant_or_self_nodes: DocumentNodeSet = DocumentNodeSet::new();
    for node_id in matched_nodes {
        let children: Vec<DocumentNode> = node_id.children(&document).collect();
        descendant_or_self_nodes.insert_all(children);
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
pub fn search(search_params: &XpathSearchParams, document: &HtmlDocument, searchable_nodes: &DocumentNodeSet) -> Result<DocumentNodeSet, ApplyError> {
    let mut matches = DocumentNodeSet::new();

    for node_id in searchable_nodes.iter() {
        if let Some(node) = document.get_html_node(node_id) {
            match node {
                HtmlNode::Tag(rtag) => {
                    if &rtag.name == search_params.tag_name && is_matching_predicates(search_params.query, rtag) && !matches.contains(node_id) {
                        matches.insert(*node_id);
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

        let query:XpathQuery = Default::default();
        let search_params = XpathSearchParams {
            tag_name: "root",
            query: &query,
            axis: Default::default(),
            index: Default::default()
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

        let query:XpathQuery = Default::default();
        let search_params = XpathSearchParams {
            tag_name: "a",
            query: &query,
            axis: XpathAxes::DescendantOrSelf,
            index: Default::default()
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
        let search_params = XpathSearchParams {
            tag_name: "a",
            query: &query,
            axis: XpathAxes::DescendantOrSelf,
            index: Default::default()
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
        let search_params = XpathSearchParams {
            tag_name: "a",
            query: &query,
            axis: XpathAxes::DescendantOrSelf,
            index: Default::default()
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
