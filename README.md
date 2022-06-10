# Skyscraper - HTML scraping with XPath

[![Dependency Status](https://deps.rs/repo/github/James-LG/Skyscraper/status.svg)](https://deps.rs/repo/github/James-LG/Skyscraper)
[![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/James-LG/Skyscraper/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/skyscraper.svg)](https://crates.io/crates/skyscraper)
[![doc.rs](https://docs.rs/skyscraper/badge.svg)](https://docs.rs/skyscraper)

Rust library to scrape HTML documents with XPath expressions.

## HTML Parsing

Skyscraper has its own HTML parser implementation. The parser outputs a
tree structure that can be traversed manually with parent/child relationships.

### Example: Simple HTML Parsing

```rust
use skyscraper::html::{self, parse::ParseError};
let html_text = r##"
<html>
    <body>
        <div>Hello world</div>
    </body>
</html>"##;
 
let document = html::parse(html_text)?;
```

### Example: Traversing Parent/Child Relationships

```rust
// Parse the HTML text into a document
let text = r#"<parent><child/><child/></parent>"#;
let document = html::parse(text)?;
 
// Get the children of the root node
let parent_node: DocumentNode = document.root_node;
let children: Vec<DocumentNode> = parent_node.children(&document).collect();
assert_eq!(2, children.len());
 
// Get the parent of both child nodes
let parent_of_child0: DocumentNode = children[0].parent(&document).expect("parent of child 0 missing");
let parent_of_child1: DocumentNode = children[1].parent(&document).expect("parent of child 1 missing");
 
assert_eq!(parent_node, parent_of_child0);
assert_eq!(parent_node, parent_of_child1);
```

## XPath Expressions

Skyscraper is capable of parsing XPath strings and applying them to HTML
documents.

```rust
use skyscraper::{html, xpath};
// Parse the html text into a document.
let html_text = r##"
<div>
    <div class="foo">
        <span>yes</span>
    </div>
    <div class="bar">
        <span>no</span>
    </div>
</div>
"##;
let document = html::parse(html_text)?;
 
// Parse and apply the xpath.
let expr = xpath::parse("//div[@class='foo']/span")?;
let results = expr.apply(&document)?;
assert_eq!(1, results.len());
 
// Get text from the node
let text = results[0].get_text(&document).expect("text missing");
assert_eq!("yes", text);
```
