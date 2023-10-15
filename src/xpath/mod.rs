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
//!
//! # Example: get all text from all selected nodes
//! ```rust
//! # use std::error::Error;
//! use skyscraper::{html, xpath};
//! # fn main() -> Result<(), Box<dyn Error>> {
//! // Parse the text into a document.
//! let text = r##"
//!     <parent>
//!         <div>
//!             hi
//!             <span>foo</span>
//!         </div>
//!         <div>
//!             bye
//!             <a>bar</a>
//!         </div>
//!     </parent>"##;
//!
//! let document = html::parse(text)?;
//!
//! // Create and apply the xpath.
//! let xpath = xpath::parse("//div")?;
//! let results = xpath.apply(&document)?;
//!
//! // Collect the text of all nodes.
//! let text: Vec<String> = results
//!     .into_iter()
//!     .filter_map(|n| n.get_all_text(&document))
//!     .collect();
//!
//! // Assertions
//! assert_eq!("hi foo", text[0]);
//! assert_eq!("bye bar", text[1]);
//! # Ok(())
//! # }
//! ```
//!
//! # Example: get attributes from selected node
//! ```rust
//! # use std::error::Error;
//! use skyscraper::{html, xpath};
//! # fn main() -> Result<(), Box<dyn Error>> {
//! // Parse the text into a document.
//! let text = r##"
//!     <parent>
//!         <div class="hi" id="bye"></div>
//!     </parent>"##;
//!
//! let document = html::parse(text)?;
//!
//! // Create and apply the xpath.
//! let xpath = xpath::parse("//div")?;
//! let results = xpath.apply(&document)?;
//!
//! let node = results[0];
//! let attributes = node.get_attributes(&document)
//!     .expect("should have attributes");
//!
//! assert_eq!("hi", attributes["class"]);
//! assert_eq!("bye", attributes["id"]);
//! # Ok(())
//! # }
//! ```

mod grammar;
pub mod parse;
mod search;
mod tokenizer;

use std::{iter::FromIterator, ops::Index};

use crate::html::{DocumentNode, HtmlDocument, HtmlTag};
use indexmap::{indexset, IndexSet};
use thiserror::Error;

pub use crate::xpath::{parse::parse, search::search};

/// Represents a set of additional conditions for a single XPath search.
///
/// ```text
/// //div[@class="node" @id="input"]
///       ^^^^^^^^^^^^^ ^^^^^^^^^^^
///       Predicate 1   Predicate 2
/// ```
#[derive(Debug, PartialEq, Default, Clone)]
pub struct XpathQuery {
    /// The list of conditions.
    pub predicates: Vec<XpathPredicate>,
}

/// A single condition for an XPath search.
#[derive(Debug, PartialEq, Clone)]
pub enum XpathPredicate {
    /// Asserts that an attribute has a value greater than the given `value`.
    ///
    /// Example: `price>3`
    GreaterThan {
        /// The attribute name.
        attribute: String,
        /// The value the attribute must be greater than.
        value: u64,
    },

    /// Asserts than an attribute has a value less than the given `value`.
    ///
    /// Example: `price<3`
    LessThan {
        /// The attribute name.
        attribute: String,
        /// The value the attribute must be less than.
        value: u64,
    },

    /// Asserts than an attribute has a value equal to the given `value`.
    ///
    /// Example: `id="input"`
    Equals {
        /// The attribute name.
        attribute: String,
        /// The value the attribute must be equal to.
        value: String,
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
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum XpathAxes {
    /// Get direct children.
    #[default]
    Child,

    /// Get all descendants and self.
    DescendantOrSelf,

    /// Get direct parent.
    Parent,
}

impl XpathAxes {
    /// Returns the [XpathAxes] corresponding to the given `text`, or [None] if the string
    /// is not a known axis.
    pub fn try_from_str(text: &str) -> Option<Self> {
        match text {
            "parent" => Some(XpathAxes::Parent),
            "child" => Some(XpathAxes::Child),
            "descendent-or-self" => Some(XpathAxes::DescendantOrSelf),
            _ => None,
        }
    }
}

/// A type of node to search for.
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Default, Clone)]
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
#[derive(Clone)]
pub struct Xpath {
    items: Vec<XpathSearchItem>,
}

/// An error occurring while applying an [Xpath] expression
/// to an [HtmlDocument].
///
/// **Note:** Currently empty. Exists in case items are added
/// while expanind Skyscraper feature set.
#[derive(Error, Debug)]
pub enum ApplyError {}

/// An ordered set of unique [DocumentNodes](DocumentNode).
///
/// Backed by [IndexSet].
#[derive(Default, Debug)]
pub struct DocumentNodeSet {
    index_set: IndexSet<DocumentNode>,

    /// `has_super_root` states that there is a fake node that we should pretend is included
    /// in `index_set`. It is defined as the parent of `document.root_node`. Its purpose
    /// is to facilitate the selection of `document.root_node` absolute path queries.
    has_super_root: bool,
}

impl From<IndexSet<DocumentNode>> for DocumentNodeSet {
    fn from(index_set: IndexSet<DocumentNode>) -> Self {
        DocumentNodeSet {
            index_set,
            has_super_root: false,
        }
    }
}

impl DocumentNodeSet {
    /// Create a new empty [DocumentNodeSet].
    ///
    /// Setting `has_super_root` includes a fake node dubbed the "super root" in this
    /// [DocumentNodeSet]. The super root node is defined as the parent of
    /// `document.root_node` and can be used to match the document's root node with a query.
    ///
    /// See [search] for more information on using `has_super_root`.
    pub fn new(has_super_root: bool) -> DocumentNodeSet {
        DocumentNodeSet {
            index_set: Default::default(),
            has_super_root,
        }
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
        let searchable_nodes: DocumentNodeSet = DocumentNodeSet::new(true);
        self.internal_apply(document, searchable_nodes)
    }

    /// Search the the descendents of the given node in the given HTML document
    /// according to this Xpath expression.
    pub fn apply_to_node(
        &self,
        document: &HtmlDocument,
        doc_node: DocumentNode,
    ) -> Result<Vec<DocumentNode>, ApplyError> {
        let searchable_nodes = DocumentNodeSet::from(indexset![doc_node]);
        self.internal_apply(document, searchable_nodes)
    }

    fn internal_apply(
        &self,
        document: &HtmlDocument,
        searchable_nodes: DocumentNodeSet,
    ) -> Result<Vec<DocumentNode>, ApplyError> {
        let items = &mut self.items.iter();
        let mut matched_nodes: DocumentNodeSet = searchable_nodes; // The nodes matched by the search query

        for search_item in items.by_ref() {
            matched_nodes = search(search_item, document, &matched_nodes)?;
        }

        Ok(matched_nodes.into_iter().collect())
    }
}

#[cfg(test)]
mod test {
    use crate::{html, xpath};

    use super::*;

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

        let tag = node.unwrap_tag();
        assert_eq!("node", tag.name);
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

        let tag = node.unwrap_tag();
        assert_eq!("node", tag.name);

        let children: Vec<DocumentNode> = node_id.children(&document).collect();
        assert_eq!(1, children.len());

        let child_node = document.get_html_node(&children[0]).unwrap();
        let child_text = child_node.unwrap_text();
        assert_eq!("0", child_text);
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

        let tag = node.unwrap_tag();
        assert_eq!("1", tag.attributes["id"]);

        let node_id = nodes[1];
        let node = document.get_html_node(&node_id).unwrap();

        let tag = node.unwrap_tag();
        assert_eq!("2", tag.attributes["id"]);
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
    fn xpath_apply_should_find_root_in_search() {
        // arrange
        let text = r###"<!DOCTYPE html>
        <root>
            <node>0</node>
            <node>1</node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("/root").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(1, nodes.len());
        let html_node = document.get_html_node(&nodes[0]).unwrap();

        assert_eq!("root", html_node.unwrap_tag().name);
    }

    #[test]
    fn xpath_apply_should_find_root_in_search_all() {
        // arrange
        let text = r###"<!DOCTYPE html>
        <root>
            <node>0</node>
            <node>1</node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("//root").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(1, nodes.len());
        let html_node = document.get_html_node(&nodes[0]).unwrap();

        assert_eq!("root", html_node.unwrap_tag().name);
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

        assert_eq!("node", node.unwrap_tag().name);

        // STAGE 2: Apply xpath to node 2, and check correct div was retrieved.
        // arrange
        let xpath = xpath::parse("/div[@class='duplicate']").unwrap();

        // act
        let nodes = xpath.apply_to_node(&document, doc_node).unwrap();

        // assert
        assert_eq!(1, nodes.len());
        let doc_node = nodes[0];
        let node = document.get_html_node(&doc_node).unwrap();

        let tag = node.unwrap_tag();
        assert_eq!("div", tag.name);
        assert_eq!("2", tag.get_text(&doc_node, &document).unwrap());
    }

    #[test]
    fn xpath_apply_handles_wildcards() {
        // arrange
        let text = r###"
        <root>
            <a>hi</a>
            <b>bye</b>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let xpath = xpath::parse("/root/*").unwrap();

        // act
        let nodes = xpath.apply(&document).unwrap();

        // assert
        assert_eq!(2, nodes.len());
        let mut nodes = nodes.into_iter();

        let tag = document
            .get_html_node(&nodes.next().unwrap())
            .unwrap()
            .unwrap_tag();
        assert_eq!("a", tag.name);

        let tag = document
            .get_html_node(&nodes.next().unwrap())
            .unwrap()
            .unwrap_tag();
        assert_eq!("b", tag.name);
    }
}
