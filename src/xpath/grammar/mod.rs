// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md
// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar

mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod xml_names;

pub use expressions::{xpath, XPath};
use indextree::{Arena, NodeId};
use thiserror::Error;

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{AttributeNode, ElementNode, Node, TextNode},
};

use self::data_model::{NodeChild, XpathItem};

use super::DocumentNodeSet;

#[derive(PartialEq, Debug, Error)]
#[error("Error applying expression {msg}")]
pub struct ExpressionApplyError {
    msg: String,
}

trait Expression {
    fn apply(
        &self,
        item_tree: &XpathItemTree,
        searchable_nodes: &Vec<NodeId>,
    ) -> Result<Vec<XpathItem>, ExpressionApplyError>;
}

struct XpathItemTree {
    arena: Arena<NodeChild>,
    /// The root node of the document.
    pub root_node: NodeId,
}

impl XpathItemTree {
    fn from_html_document(html_document: &HtmlDocument) -> Self {
        fn internal_from_html_document(
            current_html_node: &DocumentNode,
            html_document: &HtmlDocument,
            item_arena: &mut Arena<NodeChild>,
        ) -> NodeId {
            let html_node = html_document
                .get_html_node(&current_html_node)
                .expect("html document missing expected node");
            let root_item = match html_node {
                HtmlNode::Tag(tag) => {
                    let attributes = tag
                        .attributes
                        .iter()
                        .map(|a| AttributeNode {
                            name: a.0.to_string(),
                            value: a.1.to_string(),
                        })
                        .collect();
                    NodeChild::ElementNode(ElementNode {
                        name: tag.name.to_string(),
                        attributes,
                    })
                }
                HtmlNode::Text(text) => NodeChild::TextNode(TextNode {
                    content: text.to_string(),
                }),
            };

            let root_item_id = item_arena.new_node(root_item);

            for child in current_html_node.children(&html_document) {
                let child_node = internal_from_html_document(&child, html_document, item_arena);
                root_item_id.append(child_node, item_arena);
            }

            root_item_id
        }

        let mut item_arena = Arena::<NodeChild>::new();
        let root_item =
            internal_from_html_document(&html_document.root_node, &html_document, &mut item_arena);

        XpathItemTree {
            arena: item_arena,
            root_node: root_item,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn xpath_should_parse1() {
        // arrange
        let input = "//div[@class='BorderGrid-cell']/div[@class=' text-small']/a";

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(res.to_string(), input);
        assert_eq!(next_input, "");
    }

    #[test]
    fn xpath_should_parse2() {
        // arrange
        let input = r#"fn:doc("bib.xml")/books/book[fn:count(./author)>1]"#;

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn xpath_should_parse3() {
        // arrange
        let input = "book/(chapter|appendix)/section";

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn xpath_should_parse4() {
        // arrange
        let input = "$products[price gt 100]";

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
