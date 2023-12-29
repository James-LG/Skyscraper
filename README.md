# Skyscraper - HTML scraping with XPath

[![Dependency Status](https://deps.rs/repo/github/James-LG/Skyscraper/status.svg)](https://deps.rs/repo/github/James-LG/Skyscraper)
[![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/James-LG/Skyscraper/blob/master/LICENSE)
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

Skyscraper is capable of parsing XPath strings and applying them to HTML documents.

Please see the [docs](https://docs.rs/skyscraper/latest/skyscraper/xpath/index.html) for more examples.

```rust
use skyscraper::html;
use skyscraper::xpath::{self, XpathItemTree, grammar::{XpathItemTreeNodeData, data_model::{Node, XpathItem}}};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let html_text = r##"
    <html>
        <body>
            <div>Hello world</div>
        </body>
    </html>"##;

    let document = html::parse(html_text)?;
    let xpath_item_tree = XpathItemTree::from(&document);
    let xpath = xpath::parse("//div")?;
   
    let nodes = xpath.apply(&xpath_item_tree)?;
   
    assert_eq!(nodes.len(), 1);
   
    let mut nodes = nodes.into_iter();
   
    let node = nodes
        .next()
        .unwrap();

    let element = node
        .as_node()?
        .as_tree_node()?
        .data
        .as_element_node()?;

    assert_eq!(element.name, "div");
    Ok(())
}
```
