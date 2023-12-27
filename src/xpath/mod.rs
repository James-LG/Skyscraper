//! Parse and apply XPath expressions to HTML documents.

use std::fmt::Display;

use nom::error::VerboseError;
use thiserror::Error;

use crate::xpath::grammar::data_model::AnyAtomicType;

use self::{
    grammar::{data_model::XpathItem, xpath, XpathItemTreeNode},
    xpath_item_set::XpathItemSet,
};

pub mod grammar;
pub mod xpath_item_set;

pub use self::grammar::{Xpath, XpathItemTree};

/// Parse a string into an [XPath] expression.
pub fn parse(input: &str) -> Result<Xpath, nom::Err<VerboseError<&str>>> {
    xpath(input).map(|x| x.1)
}

/// Error that occurs when applying an [XPath] expression to an [XpathItemTree].
#[derive(PartialEq, Debug, Error)]
#[error("Error applying expression {msg}")]
pub struct ExpressionApplyError {
    msg: String,
}

pub(crate) struct XpathExpressionContext<'tree> {
    item_tree: &'tree XpathItemTree,
    item: XpathItem<'tree>,
    position: usize,
    size: usize,
}

impl<'tree> XpathExpressionContext<'tree> {
    pub fn new(
        item_tree: &'tree XpathItemTree,
        items: &XpathItemSet<'tree>,
        position: usize,
    ) -> Self {
        Self {
            item_tree,
            item: items[position - 1].clone(), // Position is 1-based
            position: position,
            size: items.len(),
        }
    }

    pub fn new_single(item_tree: &'tree XpathItemTree, item: XpathItem<'tree>) -> Self {
        Self {
            item_tree,
            item,
            position: 1,
            size: 1,
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
