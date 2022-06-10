//! Skyscraper parses HTML documents into structured trees with the help of [indextree].
//! It can then traverse the tree and select nodes using standard XPath expressions either
//! created programatically or parsed from XPath string literals.
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
//! // Parse the HTML text
//! let document = html::parse(text)?;
//!
//! // Assuming your XPath string is static, it is safe to use `expect` during parsing
//! let xpath = xpath::parse("//div[@class='yes']/parent::div/div[@class='duplicate']")
//!     .expect("xpath is invalid");
//!
//! // Apply the XPath expression to our HTML document
//! let nodes = xpath.apply(&document)?;
//!
//! assert_eq!(1, nodes.len());
//!
//! // Compare the text of the first and only node returned by the XPath expression
//! let text = nodes[0].get_text(&document)
//!     .ok_or_else(|| "text is missing")?;
//!
//! assert_eq!("Good info", text);
//! # Ok(())
//! # }
//! ```
//! 
//! # Example: use [mod@lazy_static] if Xpath expressions are static
//! 
//! If your Xpath expressions are static, and you have a function that
//! parses and applies the expression every time the function is called,
//! consider using [mod@lazy_static] to prevent the expression from being
//! repeatedly parsed.
//! 
//! ```rust
//! #[macro_use]
//! extern crate lazy_static;
//! 
//! use std::error::Error;
//! use skyscraper::{html::{self, HtmlDocument}, xpath::{self, Xpath}};
//! 
//! lazy_static! {
//!     static ref SPAN_XPATH: Xpath = xpath::parse("/div/span").unwrap();
//! }
//! 
//! fn my_func(document: &HtmlDocument) -> Result<Option<String>, Box<dyn Error>> {
//!     let xpath_results = SPAN_XPATH.apply(document)?;
//!     Ok(xpath_results[0].get_text(document))
//! }
//! 
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let doc1 = html::parse("<div><span>foo</span></div>")?;
//!     let text1 = my_func(&doc1)?.expect("text missing");
//!     assert_eq!("foo", text1);
//! 
//!     let doc2 = html::parse("<div><span>bar</span></div>")?;
//!     let text2 = my_func(&doc2)?.expect("text missing");
//!     assert_eq!("bar", text2);
//! 
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;

pub mod html;
mod vecpointer;
pub mod xpath;
