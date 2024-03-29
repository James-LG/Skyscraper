# Skyscraper - HTML scraping with XPath

[![Dependency Status](https://deps.rs/repo/github/James-LG/Skyscraper/status.svg)](https://deps.rs/repo/github/James-LG/Skyscraper)
[![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/James-LG/Skyscraper/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/skyscraper.svg)](https://crates.io/crates/skyscraper)
[![doc.rs](https://docs.rs/skyscraper/badge.svg)](https://docs.rs/skyscraper)

Rust library to scrape HTML documents with XPath expressions.

> This library is major-version 0 because there are still `todo!` calls for many xpath features.
>If you encounter one that you feel should be prioritized, open an issue on [GitHub](https://github.com/James-LG/Skyscraper/issues).
>
> See the [Supported XPath Features](#supported-xpath-features) section for details.

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

Below is a basic xpath example. Please see the [docs](https://docs.rs/skyscraper/latest/skyscraper/xpath/index.html) for more examples.

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
   
    let item_set = xpath.apply(&xpath_item_tree)?;
   
    assert_eq!(item_set.len(), 1);
   
    let mut items = item_set.into_iter();
   
    let item = items
        .next()
        .unwrap();

    let element = item
        .as_node()?
        .as_tree_node()?
        .data
        .as_element_node()?;

    assert_eq!(element.name, "div");
    Ok(())
}
```

### Supported XPath Features

Below is a non-exhaustive list of all the features that are currently supported.

1. Basic xpath steps: `/html/body/div`, `//div/table//span`
1. Attribute selection: `//div/@class`
1. Text selection: `//div/text()`
1. Wildcard node selection: `//body/*`
1. Predicates:
    1. Attributes: `//div[@class='hi']`
    1. Indexing: `//div[1]`
1. Functions:
    1. `fn:root()`
    1. `contains(haystack, needle)`
1. Forward axes:
    1. Child: `child::*`
    1. Descendant: `descendant::*`
    1. Attribute: `attribute::*`
    1. DescendentOrSelf: `descendant-or-self::*`
    1. (more coming soon)
1. Reverse axes:
    1. Parent:  `parent::*`
    1. (more coming soon)
1. Treat expressions: `/html treat as node()`

This should cover most XPath use-cases.
If your use case requires an unimplemented feature,
please open an issue on [GitHub](https://github.com/James-LG/Skyscraper/issues).
