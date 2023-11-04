// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md
// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar

mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod xml_names;

pub use expressions::{xpath, XPath};

use crate::html::{DocumentNode, HtmlDocument};

use super::DocumentNodeSet;

pub struct ExpressionApplyError {}

trait Expression {
    fn apply(
        &self,
        document: &HtmlDocument,
        searchable_nodes: DocumentNodeSet,
    ) -> Result<Vec<DocumentNode>, ExpressionApplyError>;
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
