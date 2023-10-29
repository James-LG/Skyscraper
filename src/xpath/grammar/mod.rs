// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md
// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar

mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod xml_names;

pub use expressions::{xpath, XPath};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn xpath_should_parse() {
        // arrange
        let input = "//div[@class='BorderGrid-cell']/div[@class=' text-small']/a";

        // act
        let (next_input, xpath) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(format!("{}", xpath), input);
    }
}
