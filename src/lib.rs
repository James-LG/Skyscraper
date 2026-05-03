//! Skyscraper parses HTML documents into structured trees with the help of [indextree].
//! It can then traverse the tree and select nodes using standard XPath expressions either
//! created programatically or parsed from XPath string literals.
//!
//! This crate is split into two main modules: [html] and [xpath].
//!
//! For more information on HTML documents and nodes, including how to get text or attributes from nodes,
//! see the [html] module documentation.
//!
//! For more information on XPath expressions, see the [xpath] module documentation.
//!
//! # Example: parse an HTML document and use an XPath expression
//! ```rust
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use skyscraper::html;
//! use skyscraper::xpath;
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
//! // Parse the HTML text into an XpathItemTree
//! let tree = html::parse(text)?;
//!
//! // Assuming your XPath string is static, it is safe to use `expect` during parsing
//! let xpath = xpath::parse("//div[@class='yes']/parent::div/div[@class='duplicate']")
//!     .expect("xpath is invalid");
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&tree)?;
//!
//! assert_eq!(items.len(), 1);
//!
//! // Compare the text of the first and only node returned by the XPath expression
//! let node = items[0].extract_as_node();
//! let text = node.text(&tree).unwrap();
//!
//! assert_eq!(text, "Good info");
//!
//! // Assert that node class attribute is "duplicate" string.
//! let element = node.extract_as_element_node();
//! let attribute = element.get_attribute(&tree, "class").unwrap();
//! assert_eq!(attribute, "duplicate");
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: use LazyLock if Xpath expressions are static
//!
//! If your Xpath expressions are static, and you have a function that
//! parses and applies the expression every time the function is called,
//! consider using [`std::sync::LazyLock`] to prevent the expression from being
//! repeatedly parsed.
//!
//! ```rust
//! use std::error::Error;
//! use std::sync::LazyLock;
//! use skyscraper::{html, xpath::{self, Xpath, XpathItemTree}};
//!
//! static SPAN_XPATH: LazyLock<Xpath> = LazyLock::new(|| xpath::parse("//span").unwrap());
//!
//! fn my_func(tree: &XpathItemTree) -> Result<String, Box<dyn Error>> {
//!     let result = SPAN_XPATH.apply(tree)?;
//!
//!     let items = result;
//!     let node = items[0].extract_as_node();
//!     Ok(node.text(tree).unwrap())
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let tree1 = html::parse("<div><span>foo</span></div>")?;
//!     let text1 = my_func(&tree1)?;
//!     assert_eq!(text1, "foo");
//!
//!     let tree2 = html::parse("<div><span>bar</span></div>")?;
//!     let text2 = my_func(&tree2)?;
//!     assert_eq!(text2, "bar");
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

pub mod html;
mod vecpointer;
pub mod xpath;
