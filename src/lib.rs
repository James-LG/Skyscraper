//! Skyscraper parses HTML documents into structured trees with the help of [indextree].
//! It can then traverse the tree and select nodes using standard XPath expressions either
//! created programatically or parsed from XPath string literals.
//!
//! # Example: parse an HTML document and use an XPath expression
//! ```rust
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use skyscraper::html;
//! use skyscraper::xpath::{self, XpathItemTree};
//!
//! let text = r##"
//! <html>
//!     <body>
//!         <div>
//!             <div class="no"></div>
//!             <div class="duplicate">Bad info</div>
//!         </div>
//!         <div>
//!             <div class="yes"></div>
//!             <div class="duplicate">Good info</div>
//!         </div>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let document = html::parse(text)?;
//! let xpath_item_tree = XpathItemTree::from(&document);
//!
//! // Assuming your XPath string is static, it is safe to use `expect` during parsing
//! let xpath = xpath::parse("//div[@class='yes']/parent::div/div[@class='duplicate']")
//!     .expect("xpath is invalid");
//!
//! // Apply the XPath expression to our HTML document
//! let result = xpath.apply(&xpath_item_tree)?;
//!
//! // The xpath expression that was used always returns an item set.
//! let items = result;
//!
//! assert_eq!(items.len(), 1);
//!
//! // Compare the text of the first and only node returned by the XPath expression
//! let node = items[0].extract_as_node().extract_as_tree_node();
//! let text = node.text(&xpath_item_tree).unwrap();
//!
//! assert_eq!(text, "Good info");
//!
//! // Assert that node class attribute is "duplicate" string.
//! let element = node.data.extract_as_element_node();
//! let attribute = element.get_attribute("class").unwrap();
//! assert_eq!(attribute, "duplicate");
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: use once_cell if Xpath expressions are static
//!
//! If your Xpath expressions are static, and you have a function that
//! parses and applies the expression every time the function is called,
//! consider using [mod@once_cell] to prevent the expression from being
//! repeatedly parsed.
//!
//! ```rust
//! use std::error::Error;
//! use skyscraper::{html::{self, HtmlDocument}, xpath::{self, Xpath, XpathItemTree}};
//! use once_cell::sync::Lazy;
//!
//! static SPAN_XPATH: Lazy<Xpath> = Lazy::new(|| xpath::parse("/div/span").unwrap());
//!
//! fn my_func(document: &HtmlDocument) -> Result<String, Box<dyn Error>> {
//!     let xpath_item_tree = XpathItemTree::from(document);
//!     let result = SPAN_XPATH.apply(&xpath_item_tree)?;
//!
//!     let items = result;
//!     let node = items[0].extract_as_node().extract_as_tree_node();
//!     Ok(node.text(&xpath_item_tree).unwrap())
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let doc1 = html::parse("<div><span>foo</span></div>")?;
//!     let text1 = my_func(&doc1)?;
//!     assert_eq!(text1, "foo");
//!
//!     let doc2 = html::parse("<div><span>bar</span></div>")?;
//!     let text2 = my_func(&doc2)?;
//!     assert_eq!(text2, "bar");
//!
//!     Ok(())
//! }
//! ```
//!
//! For more information on HTML documents and nodes, including how to get text or attributes from nodes,
//! see the [html] module documentation.
//!
//! For more information on XPath expressions, see the [xpath] module documentation.

#![warn(missing_docs)]

pub mod html;
mod vecpointer;
pub mod xpath;
