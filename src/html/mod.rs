//! Parse HTML documents into [HtmlDocuments](HtmlDocument).
//! 
//! # Example: parse HTML text into a document
//! ```rust
//! use skyscraper::html::{self, parse::ParseError};
//! # fn main() -> Result<(), ParseError> {
//! let html_text = r##"
//! <html>
//!     <body>
//!         <div>Hello world</div>
//!     </body>
//! </html>"##;
//! 
//! let document = html::parse(html_text)?;
//! # Ok(())
//! # }

pub mod parse;
mod tokenizer;

use std::collections::HashMap;

use indextree::{Arena, NodeId};

pub use crate::html::parse::parse;

type TagAttributes = HashMap<String, String>;

/// An HTML tag and its attributes.
#[derive(Debug, PartialEq)]
pub struct HtmlTag {
    /// Name of the tag.
    pub name: String,
    /// Map of the tag's attributes and their corresponding values.
    /// Example: Attributes of <div class="hello" id="world"/>
    pub attributes: TagAttributes,
}

impl HtmlTag {
    /// Creates a new tag with the given name and no attributes.
    pub fn new(name: String) -> HtmlTag {
        HtmlTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

impl HtmlTag {
    /// Gets any direct HtmlNode::Text children and concatenates them into a single string
    /// separated by a space character if no whitespace already separates them.
    pub fn get_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, false)
    }

    /// Gets all HtmlNode::Text children and concatenates them into a single string separated
    /// by a space character if no whitespace already separates them.
    pub fn get_all_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, true)
    }

    fn internal_get_text(
        &self,
        doc_node: &DocumentNode,
        document: &HtmlDocument,
        recurse: bool,
    ) -> Option<String> {
        let mut o_text: Option<String> = None;
        let children = doc_node.children(document);

        // Iterate through this tag's children
        for child in children {
            let child_node = document.get_html_node(&child);
            if let Some(child_node) = child_node {
                match child_node {
                    HtmlNode::Text(text) => {
                        // If the child is a text, simply append its text.
                        o_text = Some(HtmlTag::append_text(o_text, text.to_string()));
                    }
                    HtmlNode::Tag(_) => {
                        // If the child is a tag, only append its text if recurse=true was passed,
                        // otherwise skip this node.
                        if recurse {
                            let o_child_text = child_node.internal_get_text(&child, document, true);
                            if let Some(child_text) = o_child_text {
                                o_text = Some(HtmlTag::append_text(o_text, child_text));
                            }
                        }
                    }
                }
            }
        }

        o_text
    }

    fn append_text(o_text: Option<String>, append_text: String) -> String {
        match o_text {
            Some(t) => {
                // If whitespace is already separating them, do not add another.
                if t.ends_with(|ch: char| ch.is_whitespace()) || append_text.starts_with(|ch: char| ch.is_whitespace()) {
                    format!("{}{}", t, append_text)
                } else {
                    format!("{} {}", t, append_text)
                }
                
            }
            None => append_text,
        }
    }
}

/// An HTML node can be either a tag or raw text.
pub enum HtmlNode {
    /// An HTML tag.
    Tag(HtmlTag),
    /// Text content contained within [HtmlNode::Tag].
    /// 
    /// Kept as separate enum value rather than a field on [HtmlTag] so
    /// that order can be maintained in nodes containing a mix of text
    /// and tags.
    /// 
    /// # Example: order of mixed text and tag contents is preserved
    /// ```html
    /// <div>
    ///     Hello <span style="bold">world</span>!
    /// </div>
    /// ```
    /// Where the inner contents of `div` would be: `Text("Hello ")`, `Tag(span)`, `Text("!")`.
    /// 
    Text(String),
}

impl HtmlNode {
    /// Gets any direct HtmlNode::Text children and concatenates them into a single string
    /// separated by a space character if no whitespace already separates them.
    pub fn get_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, false)
    }

    /// Gets all HtmlNode::Text children and concatenates them into a single string separated
    /// by a space character if no whitespace already separates them.
    pub fn get_all_text(&self, doc_node: &DocumentNode, document: &HtmlDocument) -> Option<String> {
        self.internal_get_text(doc_node, document, true)
    }

    /// Gets any direct HtmlNode::Text children and concatenates them into a single string
    /// separated by a space character if no whitespace already separates them.
    fn internal_get_text(
        &self,
        doc_node: &DocumentNode,
        document: &HtmlDocument,
        recurse: bool,
    ) -> Option<String> {
        match self {
            HtmlNode::Tag(tag) => {
                if recurse {
                    tag.get_all_text(doc_node, document)
                } else {
                    tag.get_text(doc_node, document)
                }
            }
            HtmlNode::Text(text) => Some(text.to_string()),
        }
    }

    /// Gets attributes.
    /// If Node is a `Text` return None
    pub fn get_attributes(&self) -> Option<&TagAttributes> {
        match self {
            HtmlNode::Tag(tag) => Some(&tag.attributes),
            &HtmlNode::Text(_) => None,
        }
    }
}

/// HTML document tree represented by an indextree arena and a root node.
///
/// Documents must have a single root node to be valid.
pub struct HtmlDocument {
    arena: Arena<HtmlNode>,
    /// The root node of the document.
    pub root_node: DocumentNode,
}

impl HtmlDocument {
    /// Create a new [HtmlDocument] with the given `arena` contents and `root_node`.
    pub fn new(arena: Arena<HtmlNode>, root_node: DocumentNode) -> HtmlDocument {
        HtmlDocument { arena, root_node }
    }

    /// Get the [HtmlNode] associated with the given [DocumentNode].
    pub fn get_html_node(&self, node: &DocumentNode) -> Option<&HtmlNode> {
        self.arena.get(node.id).map(|x| x.get())
    }
}

/// A key representing a single [HtmlNode] contained in a [HtmlDocument].
/// 
/// Contains tree information such as parents and children.
/// 
/// Implements [Copy] so that it can be easily passed around, unlike its associated [HtmlNode].
/// 
/// # Example: get associated [HtmlNode]
/// 
/// ```rust
/// # use skyscraper::html::{self, DocumentNode, HtmlNode, parse::ParseError};
/// # fn main() -> Result<(), ParseError> {
/// // Parse the HTML text into a document
/// let text = r#"<div/>"#;
/// let document = html::parse(text)?;
/// 
/// // Get the root document node's associated HTML node
/// let doc_node: DocumentNode = document.root_node;
/// let html_node = document.get_html_node(&doc_node).expect("root node must be in document");
/// 
/// // Check we got the right node
/// match html_node {
///     HtmlNode::Tag(tag) => assert_eq!(String::from("div"), tag.name),
///     HtmlNode::Text(_) => panic!("expected tag, got text instead")
/// }
/// # Ok(())
/// # }
/// ```
/// 
/// # Example: get children and parents
/// 
/// ```rust
/// # use skyscraper::html::{self, DocumentNode, HtmlNode, parse::ParseError};
/// # fn main() -> Result<(), ParseError> {
/// // Parse the HTML text into a document
/// let text = r#"<parent><child/><child/></parent>"#;
/// let document = html::parse(text)?;
/// 
/// // Get the children of the root node
/// let parent_node: DocumentNode = document.root_node;
/// let children: Vec<DocumentNode> = parent_node.children(&document).collect();
/// assert_eq!(2, children.len());
/// 
/// // Get the parent of both child nodes
/// let parent_of_child0: DocumentNode = children[0].parent(&document).expect("parent of child 0 missing");
/// let parent_of_child1: DocumentNode = children[1].parent(&document).expect("parent of child 1 missing");
/// 
/// assert_eq!(parent_node, parent_of_child0);
/// assert_eq!(parent_node, parent_of_child1);
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub struct DocumentNode {
    id: NodeId,
}

impl DocumentNode {
    /// Create a new [DocumentNode] from the given arena key `id`.
    pub fn new(id: NodeId) -> DocumentNode {
        DocumentNode { id }
    }

    /// Get the concatenated text of this node and all of its children.
    /// 
    /// Adds a space between elements for better readability.
    /// 
    /// # Example: get the text of a node
    /// 
    /// ```rust
    /// use skyscraper::html::{self, parse::ParseError};
    /// # fn main() -> Result<(), ParseError> {
    /// // Parse the text into a document.
    /// let text = r##"<parent>foo<child>bar</child>baz</parent>"##;
    /// let document = html::parse(text)?;
    /// 
    /// // Get all text of the root node.
    /// let doc_node = document.root_node;
    /// let text = doc_node.get_all_text(&document).expect("text missing");
    /// 
    /// assert_eq!("foo bar baz", text);
    /// # Ok(())
    /// # }
    pub fn get_all_text(&self, document: &HtmlDocument) -> Option<String> {
        match document.get_html_node(self) {
            Some(html_node) => html_node.get_all_text(self, document),
            None => None,
        }
    }

    /// Get the concatenated text of this node.
    /// 
    /// Adds a space between elements for better readability.
    /// 
    /// # Example: get the text of a node
    /// 
    /// ```rust
    /// use skyscraper::html::{self, parse::ParseError};
    /// # fn main() -> Result<(), ParseError> {
    /// // Parse the text into a document.
    /// let html_text = r##"<parent>foo<child>bar</child>baz</parent>"##;
    /// let document = html::parse(html_text)?;
    /// 
    /// // Get all text of the root node.
    /// let doc_node = document.root_node;
    /// let text = doc_node.get_text(&document).expect("text missing");
    /// 
    /// assert_eq!("foo baz", text);
    /// # Ok(())
    /// # }
    pub fn get_text(&self, document: &HtmlDocument) -> Option<String> {
        match document.get_html_node(self) {
            Some(html_node) => html_node.get_text(self, document),
            None => None,
        }
    }

    /// Get attributes.
    ///
    /// If Node is a `Text` return None
    ///
    /// ```rust
    /// use skyscraper::html::{self, parse::ParseError};
    /// # fn main() -> Result<(), ParseError> {
    /// // Parse the text into a document.
    /// let html_text = r##"<div attr1="attr1_value"></div>"##;
    /// let document = html::parse(html_text)?;
    ///
    /// // Get root node.
    /// let doc_node = document.root_node;
    /// let attributes = doc_node.get_attributes(&document).expect("No attributes");
    ///
    /// assert_eq!("attr1_value", attributes["attr1"]);
    /// # Ok(())
    /// # }
    pub fn get_attributes<'a>(&'a self, document: &'a HtmlDocument) -> Option<&'a TagAttributes> {
        match document.get_html_node(self) {
            Some(html_node) => html_node.get_attributes(),
            None => None,
        }
    }

    /// Get the children of this node as an iterator.
    pub fn children<'a>(
        &self,
        document: &'a HtmlDocument,
    ) -> impl Iterator<Item = DocumentNode> + 'a {
        Box::new(self.id.children(&document.arena).map(DocumentNode::new))
    }

    /// Get the parent of this node if it exists.
    pub fn parent(&self, document: &HtmlDocument) -> Option<DocumentNode> {
        self.id
            .ancestors(&document.arena)
            .nth(1)
            .map(DocumentNode::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_node_get_text_should_work_on_text_node() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello world"));
        let text_doc_node = DocumentNode::new(arena.new_node(text_node));
        let document = HtmlDocument::new(arena, text_doc_node);

        // act
        let text_node = document.get_html_node(&text_doc_node).unwrap();
        let result = text_node.get_text(&text_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }

    #[test]
    fn html_node_get_text_should_work_on_tag_node_with_one_text_child() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello world"));
        let text_node_id = arena.new_node(text_node);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        let tag_doc_node = DocumentNode::new(tag_node_id);
        tag_node_id.append(text_node_id, &mut arena);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }

    #[test]
    fn html_node_get_text_should_work_on_tag_node_with_two_text_children() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello"));
        let text_node_id = arena.new_node(text_node);

        let text_node2 = HtmlNode::Text(String::from("world"));
        let text_node2_id = arena.new_node(text_node2);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        tag_node_id.append(text_node_id, &mut arena);
        tag_node_id.append(text_node2_id, &mut arena);
        let tag_doc_node = DocumentNode::new(tag_node_id);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }

    #[test]
    fn html_node_get_text_should_ignore_nested_text() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello"));
        let text_node_id = arena.new_node(text_node);

        let text_node2 = HtmlNode::Text(String::from("world"));
        let text_node2_id = arena.new_node(text_node2);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        tag_node_id.append(text_node_id, &mut arena);

        let tag_node2 = HtmlNode::Tag(HtmlTag::new(String::from("tag2")));
        let tag_node2_id = arena.new_node(tag_node2);
        tag_node2_id.append(text_node2_id, &mut arena);
        tag_node_id.append(tag_node2_id, &mut arena);
        let tag_doc_node = DocumentNode::new(tag_node_id);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello", result);
    }

    #[test]
    fn html_node_get_all_text_should_include_nested_text() {
        // arrange
        let mut arena = Arena::new();
        let text_node = HtmlNode::Text(String::from("hello"));
        let text_node_id = arena.new_node(text_node);

        let text_node2 = HtmlNode::Text(String::from("world"));
        let text_node2_id = arena.new_node(text_node2);

        let tag_node = HtmlNode::Tag(HtmlTag::new(String::from("tag")));
        let tag_node_id = arena.new_node(tag_node);
        tag_node_id.append(text_node_id, &mut arena);

        let tag_node2 = HtmlNode::Tag(HtmlTag::new(String::from("tag2")));
        let tag_node2_id = arena.new_node(tag_node2);
        tag_node2_id.append(text_node2_id, &mut arena);
        tag_node_id.append(tag_node2_id, &mut arena);
        let tag_doc_node = DocumentNode::new(tag_node_id);

        let document = HtmlDocument::new(arena, tag_doc_node);

        // act
        let tag_node = document.get_html_node(&tag_doc_node).unwrap();
        let result = tag_node.get_all_text(&tag_doc_node, &document).unwrap();

        // assert
        assert_eq!("hello world", result);
    }

    #[test]
    fn html_node_get_attributes_for_tag() {
        // arrange
        let node = HtmlNode::Tag(HtmlTag {
            name: "div".to_string(),
            attributes: HashMap::from([("attr_name".to_string(), "attr_value".to_string())]),
        });

        // assert
        assert!(node.get_attributes().is_some());
        assert_eq!(node.get_attributes().unwrap()["attr_name"], "attr_value");
    }

    #[test]
    fn html_node_get_attributes_for_text() {
        // arrange
        let node = HtmlNode::Text(String::from("hello world"));

        // assert
        assert!(node.get_attributes().is_none())
    }

    #[test]
    fn document_node_get_attributes_for_tag() {
        // arrange
        let mut arena = Arena::new();
        let html_node = HtmlNode::Tag(HtmlTag {
            name: "div".to_string(),
            attributes: HashMap::from([("attr_name".to_string(), "attr_value".to_string())]),
        });
        let doc_node = DocumentNode::new(arena.new_node(html_node));
        let html_document = HtmlDocument::new(arena, doc_node);

        // act
        let node = html_document.get_html_node(&doc_node).unwrap();
        let attributes = node.get_attributes();

        // assert
        assert!(attributes.is_some());
        assert_eq!(attributes.unwrap()["attr_name"], "attr_value");
    }

    #[test]
    fn document_node_get_attributes_for_text() {
        // arrange
        let mut arena = Arena::new();
        let html_node = HtmlNode::Text(String::from("hello world"));
        let doc_node = DocumentNode::new(arena.new_node(html_node));
        let html_document = HtmlDocument::new(arena, doc_node);

        // act
        let node = html_document.get_html_node(&doc_node).unwrap();
        let attributes = node.get_attributes();

        // assert
        assert!(attributes.is_none());
    }
}
