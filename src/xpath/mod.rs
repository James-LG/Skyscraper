//! Parse and apply XPath expressions to HTML documents.
//!
//! Important pages:
//!
//! - [parse] - Parse a string into an [Xpath] expression.
//! - [Xpath::apply] - Apply an [Xpath] expression to an [XpathItemTree].
//! - [XpathItemTree] - A tree of [XpathItem]s that can be searched using an [Xpath] expression.
//!
//! # Example: get links with the `/@href` xpath step
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <a href="https://example1.com">Example 1</a>
//!         <a href="https://example2.com">Example 2</a>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let xpath_item_tree = html::parse(text)?;
//!
//! let xpath = xpath::parse("//a/@href")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let attributes: Vec<&str> = items
//!     .iter()
//!     .map(|item| item
//!         .extract_as_node() // we know it's a node all attributes are on nodes
//!         .extract_as_attribute_node() // we know it's an attribute node
//!         .value
//!         .as_str()
//!     )
//!     .collect();
//!
//! assert_eq!(attributes, vec!["https://example1.com", "https://example2.com"]);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: get links programatically
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <a href="https://example1.com">Example 1</a>
//!         <a href="https://example2.com">Example 2</a>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let xpath_item_tree = html::parse(text)?;
//!
//! let xpath = xpath::parse("//a")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let attributes: Vec<&str> = items
//!     .iter()
//!     .filter_map(|item| item
//!         .extract_as_node() // we know it's a node
//!         .extract_as_element_node() // we know it's an element node
//!         .get_attribute(&xpath_item_tree, "href")
//!     )
//!     .collect();
//!
//! assert_eq!(attributes, vec!["https://example1.com", "https://example2.com"]);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: get text using the `/text()` xpath step
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <div>Example 1</div>
//!         <div>Example 2</div>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let xpath_item_tree = html::parse(text)?;
//!
//! let xpath = xpath::parse("//div/text()")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let text_contents: Vec<String> = items
//!     .iter()
//!     .map(|item| item
//!         .extract_as_node() // we know it's a node because text is a type of node
//!         .extract_as_text_node() // we know it's a text node
//!         .content
//!         .to_string()
//!     )
//!     .collect();
//!
//! assert_eq!(text_contents, vec!["Example 1", "Example 2"]);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: get text programatically
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath;
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <div>Example 1</div>
//!         <div>Example 2</div>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let xpath_item_tree = html::parse(text)?;
//!
//! let xpath = xpath::parse("//div")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let text_contents: Vec<String> = items
//!     .iter()
//!     .map(|item| item
//!         .extract_as_node() // we know it's a node because text is type of node
//!         .extract_as_element_node() // we know it's an element node
//!         .text_content(&xpath_item_tree)
//!     )
//!     .collect();
//!
//! assert_eq!(text_contents, vec!["Example 1", "Example 2"]);
//!
//! # Ok(())
//! # }
//! ```

use std::rc::Rc;

use thiserror::Error;

use self::{
    grammar::{data_model::XpathItem, xpath},
    xpath_item_set::XpathItemSet,
};

pub mod grammar;
pub mod query;
pub mod xpath_item_set;

pub use self::grammar::{Xpath, XpathItemTree};

/// Error that occurs when parsing an [Xpath] expression.
#[derive(PartialEq, Debug, Error)]
#[error("Error parsing expression: {msg}")]
pub struct ExpressionParseError {
    msg: String,
}

/// Parse a string into an [Xpath] expression.
///
/// # Example
///
/// ```rust
/// use skyscraper::xpath::parse;
///
/// let xpath = parse("//div[@class='yes']/parent::div/div[@class='duplicate']")
///    .expect("xpath is invalid");
/// ```
pub fn parse(input: &str) -> Result<Xpath, ExpressionParseError> {
    xpath(input).map(|x| x.1).map_err(|e| ExpressionParseError {
        msg: format!("{}", e),
    })
}

/// Error that occurs when applying an [Xpath] expression to an [XpathItemTree].
#[derive(PartialEq, Debug, Error)]
#[error("Error applying expression {msg}")]
pub struct ExpressionApplyError {
    msg: String,
}

impl ExpressionApplyError {
    pub(crate) fn new(msg: String) -> Self {
        Self { msg }
    }
}

/// A scope-chain node for variable bindings.
///
/// Each node holds a small set of bindings and an optional parent pointer.
/// Lookup walks the chain (O(depth), typically <10).
#[derive(Debug)]
pub(crate) struct VariableScope<'tree> {
    bindings: Vec<(String, XpathItemSet<'tree>)>,
    parent: Option<Rc<VariableScope<'tree>>>,
}

impl<'tree> VariableScope<'tree> {
    fn empty() -> Self {
        Self {
            bindings: Vec::new(),
            parent: None,
        }
    }

    fn get(&self, name: &str) -> Option<&XpathItemSet<'tree>> {
        for (k, v) in self.bindings.iter().rev() {
            if k == name {
                return Some(v);
            }
        }
        if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct XpathExpressionContext<'tree> {
    item_tree: &'tree XpathItemTree,
    item: XpathItem<'tree>,
    position: usize,

    // size is part of the XPath expression context spec, and will be used eventually
    #[allow(unused)]
    size: usize,

    /// `true` if this is the initial step of a path expression evaluation;
    /// `false` for subsequent steps within a relative path.
    ///
    /// This determines how leading `/` and `//` are expanded.
    is_initial_step: bool,

    /// Variable bindings in scope (e.g. from `for` or `let` expressions).
    /// Uses a scope-chain so that adding a variable is O(1) instead of O(n).
    variables: Rc<VariableScope<'tree>>,
}

impl<'tree> XpathExpressionContext<'tree> {
    pub fn new_single(
        item_tree: &'tree XpathItemTree,
        item: XpathItem<'tree>,
        is_initial_step: bool,
    ) -> Self {
        Self {
            item_tree,
            item,
            position: 1,
            size: 1,
            is_initial_step,
            variables: Rc::new(VariableScope::empty()),
        }
    }

    /// Create a new context that inherits variable bindings from this context,
    /// with a new item and position derived from an item set.
    pub fn new_with_variables(
        &self,
        items: &XpathItemSet<'tree>,
        position: usize,
        is_initial_step: bool,
    ) -> Self {
        Self {
            item_tree: self.item_tree,
            item: items[position - 1].clone(),
            position,
            size: items.len(),
            is_initial_step,
            variables: Rc::clone(&self.variables),
        }
    }

    /// Create a new context that inherits variable bindings from this context,
    /// with a directly specified item, position, and size.
    ///
    /// This avoids the need to create an intermediate `XpathItemSet` when the
    /// item and positional information are already known (e.g., during grouped
    /// descendant predicate evaluation).
    pub fn new_with_item_and_size(
        &self,
        item: XpathItem<'tree>,
        position: usize,
        size: usize,
        is_initial_step: bool,
    ) -> Self {
        Self {
            item_tree: self.item_tree,
            item,
            position,
            size,
            is_initial_step,
            variables: Rc::clone(&self.variables),
        }
    }

    /// Create a new context that inherits variable bindings from this context,
    /// with a single item as the context item.
    pub fn new_single_with_variables(
        &self,
        item: XpathItem<'tree>,
        is_initial_step: bool,
    ) -> Self {
        Self {
            item_tree: self.item_tree,
            item,
            position: 1,
            size: 1,
            is_initial_step,
            variables: Rc::clone(&self.variables),
        }
    }

    /// Create a new context with an additional variable binding.
    /// Inherits all existing variables plus the new one.
    pub fn with_variable(
        &self,
        name: String,
        value: XpathItemSet<'tree>,
    ) -> Self {
        Self {
            item_tree: self.item_tree,
            item: self.item.clone(),
            position: self.position,
            size: self.size,
            is_initial_step: self.is_initial_step,
            variables: Rc::new(VariableScope {
                bindings: vec![(name, value)],
                parent: Some(Rc::clone(&self.variables)),
            }),
        }
    }

    /// Look up a variable binding by name.
    pub fn get_variable(&self, name: &str) -> Option<&XpathItemSet<'tree>> {
        self.variables.get(name)
    }

    /// Create a new context with multiple additional variable bindings.
    pub fn with_variables_iter(
        &self,
        bindings: impl IntoIterator<Item = (String, XpathItemSet<'tree>)>,
    ) -> Self {
        Self {
            item_tree: self.item_tree,
            item: self.item.clone(),
            position: self.position,
            size: self.size,
            is_initial_step: self.is_initial_step,
            variables: Rc::new(VariableScope {
                bindings: bindings.into_iter().collect(),
                parent: Some(Rc::clone(&self.variables)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_should_handle_multiple_double_slashes() {
        // arrange
        let xpath_text = r###"//hello//world"###;

        // act
        let xpath = parse(xpath_text).unwrap();

        // assert
        assert_eq!(xpath.to_string(), xpath_text);
    }

    #[test]
    fn parse_should_handle_reverse_step_after_double_slash() {
        // arrange
        let xpath_text = r###"//hello//parent::world"###;

        // act
        let xpath = parse(xpath_text).unwrap();

        // assert
        assert_eq!(xpath.to_string(), xpath_text);
    }
}
