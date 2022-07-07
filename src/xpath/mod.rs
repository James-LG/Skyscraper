//! Parse and apply XPath expressions to HTML documents.
//! 
//! # Example: parse and apply an XPath expression.
//! ```rust
//! # use std::error::Error;
//! use skyscraper::{html, xpath};
//! # fn main() -> Result<(), Box<dyn Error>> {
//! // Parse the html text into a document.
//! let html_text = r##"
//! <div>
//!     <div class="foo">
//!         <span>yes</span>
//!     </div>
//!     <div class="bar">
//!         <span>no</span>
//!     </div>
//! </div>
//! "##;
//! let document = html::parse(html_text)?;
//! 
//! // Parse and apply the xpath.
//! let expr = xpath::parse("//div[@class='foo']/span")?;
//! let results = expr.apply(&document)?;
//! assert_eq!(1, results.len());
//! 
//! // Get text from the node
//! let text = results[0].get_text(&document).expect("text missing");
//! assert_eq!("yes", text);
//! # Ok(())
//! # }
//! ```

pub mod parse;
mod tokenizer;

use std::{iter::FromIterator, ops::Index};

use crate::html::{DocumentNode, HtmlDocument, HtmlNode, HtmlTag};
use indexmap::{indexset, IndexSet};
use thiserror::Error;

pub use crate::xpath::parse::parse;

/// Represents a set of additional conditions for a single XPath search.
/// 
/// ```text
/// //div[@class="node" @id="input"]
///       ^^^^^^^^^^^^^ ^^^^^^^^^^^
///       Predicate 1   Predicate 2
/// ```
#[derive(Debug, PartialEq, Default)]
pub struct XpathQuery {
    /// The list of conditions.
    pub predicates: Vec<XpathPredicate>,
}

/// A single condition for an XPath search.
#[derive(Debug, PartialEq)]
pub enum XpathPredicate {
    /// Asserts that an attribute has a value greater than the given `value`.
    /// 
    /// Example: `price>3`
    GreaterThan {
        /// The attribute name.
        attribute: String,
        /// The value the attribute must be greater than.
        value: u64
    },

    /// Asserts than an attribute has a value less than the given `value`.
    /// 
    /// Example: `price<3`
    LessThan {
        /// The attribute name.
        attribute: String,
        /// The value the attribute must be less than.
        value: u64
    },

    /// Asserts than an attribute has a value equal to the given `value`.
    /// 
    /// Example: `id="input"`
    Equals {
        /// The attribute name.
        attribute: String,
        /// The value the attribute must be equal to.
        value: String
    },

    /// Combines two conditions with an 'and' relationship.
    /// 
    /// Example: `price>3 and price<5`
    And(Box<XpathPredicate>, Box<XpathPredicate>),

    ///Combines two conditions with an 'or' relationship.
    /// 
    /// Example: `price<3 or price>5`
    Or(Box<XpathPredicate>, Box<XpathPredicate>),
}

impl XpathQuery {
    /// Create a new [XpathQuery] with no predicates.
    pub fn new() -> XpathQuery {
        XpathQuery {
            predicates: Vec::new(),
        }
    }

    /// Check if the given `html_tag` satisfies this query's condtions.
    pub fn check_node(&self, html_tag: &HtmlTag) -> bool {
        for p in &self.predicates {
            match p {
                XpathPredicate::GreaterThan { .. } => todo!(),
                XpathPredicate::LessThan { .. } => todo!(),
                XpathPredicate::Equals { attribute, value } => {
                    if !html_tag.attributes.contains_key(attribute)
                        || &html_tag.attributes[attribute] != value
                    {
                        return false;
                    }
                }
                XpathPredicate::And(_, _) => todo!(),
                XpathPredicate::Or(_, _) => todo!(),
            }
        }

        true
    }
}

/// XPath axes as defined in <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#axes>.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum XpathAxes {
    /// Get direct children.
    Child,

    /// Get all descendants and self.
    DescendantOrSelf,

    /// Get direct parent.
    Parent,
}

impl Default for XpathAxes {
    fn default() -> Self {
        XpathAxes::Child
    }
}

/// A type of node to search for.
#[derive(Debug, PartialEq)]
pub enum XpathSearchNodeType {
    /// Search for a tag with the given name.
    Element(String),
    /// Search for all nodes.
    Any,
}

impl Default for XpathSearchNodeType {
    fn default() -> Self {
        Self::Any
    }
}

/// Everything needed for a single XPath search
/// 
/// ```text
/// //div[@class="node"]/span
/// ^-------------------^----
/// Search Item 1       Search item 2
/// ```
#[derive(Debug, PartialEq, Default)]
pub struct XpathSearchItem {
    /// The axis to search on.
    pub axis: XpathAxes,
    /// The node to search for.
    pub search_node_type: XpathSearchNodeType,
    /// The query conditions if required.
    pub query: Option<XpathQuery>,
    /// The index to apply if required.
    pub index: Option<usize>,
}

/// A full XPath expression.
/// 
/// # Example: parse and apply an XPath expression.
/// ```rust
/// # use std::error::Error;
/// use skyscraper::{html, xpath};
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // Parse the html text into a document.
/// let html_text = r##"
/// <div>
///     <span>Hello</span>
///     <span>world</span>
/// </div>
/// "##;
/// let document = html::parse(html_text)?;
/// 
/// // Parse and apply the xpath.
/// let expr = xpath::parse("/div/span")?;
/// let results = expr.apply(&document)?;
/// 
/// assert_eq!(2, results.len());
/// # Ok(())
/// # }
/// ```
pub struct Xpath {
    items: Vec<XpathSearchItem>,
}

/// An error occurring while applying an [Xpath] expression
/// to an [HtmlDocument].
/// 
/// **Note:** Currently empty. Exists in case items are added
/// while expanind Skyscraper feature set.
#[derive(Error, Debug)]
pub enum ApplyError {
}

/// An ordered set of unique [DocumentNodes](DocumentNode).
///
/// Backed by [IndexSet].
#[derive(Default)]
pub struct DocumentNodeSet {
    index_set: IndexSet<DocumentNode>,
}

impl DocumentNodeSet {
    /// Create a new empty [DocumentNodeSet].
    pub fn new() -> DocumentNodeSet {
        DocumentNodeSet {
            index_set: Default::default(),
        }
    }

    /// Create a new [DocumentNodeSet] with the items in the given [IndexSet].
    pub fn from(index_set: IndexSet<DocumentNode>) -> DocumentNodeSet {
        DocumentNodeSet { index_set }
    }

    /// Insert a [DocumentNode].
    ///
    /// See [IndexSet] for more information.
    pub fn insert(&mut self, value: DocumentNode) -> bool {
        self.index_set.insert(value)
    }

    /// Remove the last [DocumentNode].
    ///
    /// See [IndexSet] for more information.
    pub fn pop(&mut self) -> Option<DocumentNode> {
        self.index_set.pop()
    }

    /// Insert all [DocumentNodes](DocumentNode) from `iter` to self.
    pub fn insert_all(&mut self, iter: impl Iterator<Item = DocumentNode>) {
        for node in iter {
            self.insert(node);
        }
    }

    /// Scan through each value in the set and keep those where the closure `keep` returns `true`.
    ///
    /// See [IndexSet] for more information.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&DocumentNode) -> bool,
    {
        self.index_set.retain(f)
    }

    /// Return an iterator over the [DocumentNodes](DocumentNode) of the set, in their order.
    ///
    /// See [IndexSet] for more information.
    pub fn iter(&self) -> indexmap::set::Iter<DocumentNode> {
        self.index_set.iter()
    }

    /// Return `true` if an equivalent to `node` exists in the set.
    ///
    /// See [IndexSet] for more information.
    pub fn contains(&self, node: &DocumentNode) -> bool {
        self.index_set.contains(node)
    }

    /// Return the number of [DocumentNodes](DocumentNode) in the set.
    ///
    /// See [IndexSet] for more information.
    pub fn len(&self) -> usize {
        self.index_set.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// See [IndexSet] for more information.
    pub fn is_empty(&self) -> bool {
        self.index_set.is_empty()
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
        let searchable_nodes: DocumentNodeSet = DocumentNodeSet::from(indexset![document.root_node]);
        self.internal_apply(document, searchable_nodes)
    }

    /// Search the the descendents of the given node in the given HTML document
    /// according to this Xpath expression.
    pub fn apply_to_node(
        &self,
        document: &HtmlDocument,
        doc_node: DocumentNode,
    ) -> Result<Vec<DocumentNode>, ApplyError> {
        let searchable_nodes: DocumentNodeSet =
            get_all_children(document, &DocumentNodeSet::from(indexset![doc_node]));
        self.internal_apply(document, searchable_nodes)
    }

    fn internal_apply(
        &self,
        document: &HtmlDocument,
        searchable_nodes: DocumentNodeSet,
    ) -> Result<Vec<DocumentNode>, ApplyError> {
        let items = &mut self.items.iter();
        let mut matched_nodes: DocumentNodeSet = searchable_nodes; // The nodes matched by the search query

        let mut is_first_search = true;
        for search_item in items.by_ref() {
            apply_axis(
                is_first_search,
                document,
                &search_item.axis,
                &mut matched_nodes,
            );
            matched_nodes = search(search_item, document, &matched_nodes)?;

            // Apply indexing if required
            if let Some(i) = search_item.index {
                let indexed_node = matched_nodes[i - 1];
                matched_nodes.retain(|node| *node == indexed_node);
            }

            is_first_search = false;
        }

        Ok(matched_nodes.into_iter().collect())
    }
}

fn apply_axis(
    is_first_search: bool,
    document: &HtmlDocument,
    axis: &XpathAxes,
    searchable_nodes: &mut DocumentNodeSet,
) {
    match axis {
        XpathAxes::Child => {
            // Child axis is implied for first search of Xpath expression.
            // For example when given the root node we should not move down to a child
            // on the first search so that we can match on the root node itself rather than
            // only on its children. E.g. in `/root/something`, we shouldn't move down on the
            // leading slash or else we will have passed the root node already.
            if !is_first_search {
                *searchable_nodes = get_all_children(document, searchable_nodes);
            }
        }
        XpathAxes::DescendantOrSelf => {
            *searchable_nodes = get_all_descendants_or_self(document, searchable_nodes);
        }
        XpathAxes::Parent => {
            *searchable_nodes = get_all_parents(document, searchable_nodes);
        }
    }
}

/// Get all the children for all the given matched nodes.
fn get_all_children(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut child_nodes = DocumentNodeSet::new();
    for node_id in matched_nodes {
        let children = node_id.children(document);
        child_nodes.insert_all(children);
    }

    child_nodes
}

/// Get all descendants and self for all the given matched nodes.
fn get_all_descendants_or_self(
    document: &HtmlDocument,
    matched_nodes: &DocumentNodeSet,
) -> DocumentNodeSet {
    let mut descendant_or_self_nodes: DocumentNodeSet = DocumentNodeSet::new();

    for node_id in matched_nodes {
        descendant_or_self_nodes.insert(*node_id);
        let mut children: DocumentNodeSet = node_id.children(document).collect();
        if !children.is_empty() {
            children.insert_all(get_all_descendants_or_self(document, &children).into_iter())
        }
        descendant_or_self_nodes.insert_all(children.into_iter());
    }

    descendant_or_self_nodes
}

/// Get all the parents for all the given matched nodes.
fn get_all_parents(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut parent_nodes = DocumentNodeSet::new();
    for node_id in matched_nodes {
        if let Some(parent) = node_id.parent(document) {
            parent_nodes.insert(parent);
        }
    }

    parent_nodes
}

/// Search for an HTML tag matching the given search parameters in the given list of nodes.
pub fn search(
    search_params: &XpathSearchItem,
    document: &HtmlDocument,
    searchable_nodes: &DocumentNodeSet,
) -> Result<DocumentNodeSet, ApplyError> {
    let mut matches = DocumentNodeSet::new();

    for node_id in searchable_nodes.iter() {
        if let Some(node) = document.get_html_node(node_id) {
            match node {
                HtmlNode::Tag(rtag) => match &search_params.search_node_type {
                    XpathSearchNodeType::Element(tag_name) => {
                        if &rtag.name == tag_name {
                            if let Some(query) = &search_params.query {
                                if query.check_node(rtag) {
                                    matches.insert(*node_id);
                                }
                            } else {
                                matches.insert(*node_id);
                            }
                        }
                    }
                    XpathSearchNodeType::Any => {
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
            }
            _ => None,
        };

        if let Some(sub_matches) = sub_matches {
            for sub_match in sub_matches {
                matches.insert(sub_match);
            }
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod test {
    use crate::{html, xpath};

    use super::*;

    #[test]
    fn search_root_works() {
        let text = r###"<!DOCTYPE html>
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
        let result = search(
            &search_params,
            &document,
            &DocumentNodeSet::from(indexset![document.root_node]),
        )
        .unwrap();

        assert_eq!(1, result.len());
        let node = document.get_html_node(&result[0]).unwrap();

        match node {
            HtmlNode::Tag(tag) => {
                assert_eq!("root", tag.name.as_str());
            }
            HtmlNode::Text(_) => panic!("Expected tag"),
        }
    }

    #[test]
    fn search_all_works() {
        let text = r###"<!DOCTYPE html>
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
        let result = search(
            &search_params,
            &document,
            &DocumentNodeSet::from(indexset![document.root_node]),
        )
        .unwrap();

        assert_eq!(3, result.len());

        for r in result {
            let node = document.get_html_node(&r).unwrap();

            match node {
                HtmlNode::Tag(tag) => {
                    assert_eq!("a", tag.name.as_str());
                }
                HtmlNode::Text(_) => panic!("Expected tag"),
            }
        }
    }

    #[test]
    fn search_all_should_only_select_with_matching_attribute_predicate() {
        let text = r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a/></b>
            <c><b><a hello="world"/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let query = XpathQuery {
            predicates: vec![XpathPredicate::Equals {
                attribute: String::from("hello"),
                value: String::from("world"),
            }],
        };
        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            query: Some(query),
            ..Default::default()
        };
        let result = search(
            &search_params,
            &document,
            &DocumentNodeSet::from(indexset![document.root_node]),
        )
        .unwrap();

        assert_eq!(1, result.len());

        let node = document.get_html_node(&result[0]).unwrap();

        match node {
            HtmlNode::Tag(tag) => {
                assert_eq!("a", tag.name.as_str());
                assert!(tag.attributes["hello"] == String::from("world"));
            }
            HtmlNode::Text(_) => panic!("Expected tag"),
        }
    }

    #[test]
    fn search_all_should_only_select_with_multiple_matching_attribute_predicate() {
        let text = r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a hello="world"/></b>
            <c><b><a hello="world" foo="bar"/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let query = XpathQuery {
            predicates: vec![
                XpathPredicate::Equals {
                    attribute: String::from("hello"),
                    value: String::from("world"),
                },
                XpathPredicate::Equals {
                    attribute: String::from("foo"),
                    value: String::from("bar"),
                },
            ],
        };
        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            query: Some(query),
            ..Default::default()
        };
        let result = search(
            &search_params,
            &document,
            &DocumentNodeSet::from(indexset![document.root_node]),
        )
        .unwrap();

        assert_eq!(1, result.len());

        let node = document.get_html_node(&result[0]).unwrap();

        match node {
            HtmlNode::Tag(tag) => {
                assert_eq!("a", tag.name.as_str());
                assert!(tag.attributes["hello"] == String::from("world"));
            }
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
                    HtmlNode::Text(text) => assert_eq!(&String::from("0"), text),
                }
            }
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
            }
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }

        let node_id = nodes[1];
        let node = document.get_html_node(&node_id).unwrap();

        match node {
            HtmlNode::Tag(t) => {
                assert_eq!(String::from("2"), t.attributes["id"])
            }
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
            }
            HtmlNode::Text(_) => panic!("expected tag, got text instead"),
        }
    }
}
