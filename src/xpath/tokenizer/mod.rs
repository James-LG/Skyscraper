mod helpers;
mod tokens;

use crate::vecpointer::VecPointerRef;
use thiserror::Error;
pub use tokens::Token;

#[derive(Error, Debug)]
pub enum LexError {}

/// Tokenize an Xpath string to symbols for used in parsing later.
pub fn lex(text: &str) -> Result<Vec<Token>, LexError> {
    let mut symbols: Vec<Token> = Vec::new();

    let chars: Vec<char> = text.chars().collect();
    let mut pointer = VecPointerRef::new(&chars);

    while pointer.has_next() {
        if let Some(s) = helpers::is_double_slash(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_slash(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_open_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_close_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_open_square_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_close_square_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_number(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_wildcard(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_double_dot(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_dot(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_assignment_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_at_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_add_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_minus_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_greater_than_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_less_than_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_double_colon(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_identifier(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_text(&mut pointer) {
            symbols.push(s);
        } else {
            if let Some(c) = pointer.current() {
                if !c.is_whitespace() {
                    // Unknown symbol, move on ¯\_(ツ)_/¯
                    eprintln!("Unknown XPath symbol {}", c);
                }
            }
            pointer.next();
        }
    }
    Ok(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_works1() {
        // arrange
        let text = "//bookstore/book[1]/page[last()-1]";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::DoubleSlash,
            Token::Identifier(String::from("bookstore")),
            Token::Slash,
            Token::Identifier(String::from("book")),
            Token::OpenSquareBracket,
            Token::Number(1.0),
            Token::CloseSquareBracket,
            Token::Slash,
            Token::Identifier(String::from("page")),
            Token::OpenSquareBracket,
            Token::Identifier(String::from("last")),
            Token::OpenBracket,
            Token::CloseBracket,
            Token::MinusSign,
            Token::Number(1.0),
            Token::CloseSquareBracket,
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_works2() {
        // arrange
        let text = "/bookstore/book[price>35]/price";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::Slash,
            Token::Identifier(String::from("bookstore")),
            Token::Slash,
            Token::Identifier(String::from("book")),
            Token::OpenSquareBracket,
            Token::Identifier(String::from("price")),
            Token::GreaterThanSign,
            Token::Number(35.0),
            Token::CloseSquareBracket,
            Token::Slash,
            Token::Identifier(String::from("price")),
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_works3() {
        // arrange
        let text = r###"//a[@hello="world"]"###;

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::DoubleSlash,
            Token::Identifier(String::from("a")),
            Token::OpenSquareBracket,
            Token::AtSign,
            Token::Identifier(String::from("hello")),
            Token::AssignmentSign,
            Token::Text(String::from("world")),
            Token::CloseSquareBracket,
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_works_alphanumeric_identifier() {
        // arrange
        let text = r###"//h1[@hello="world"]/h2"###;

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::DoubleSlash,
            Token::Identifier(String::from("h1")),
            Token::OpenSquareBracket,
            Token::AtSign,
            Token::Identifier(String::from("hello")),
            Token::AssignmentSign,
            Token::Text(String::from("world")),
            Token::CloseSquareBracket,
            Token::Slash,
            Token::Identifier(String::from("h2")),
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_works_double_colon() {
        // arrange
        let text = r###"//h1/parent::div"###;

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::DoubleSlash,
            Token::Identifier(String::from("h1")),
            Token::Slash,
            Token::Identifier(String::from("parent")),
            Token::DoubleColon,
            Token::Identifier(String::from("div")),
        ];

        assert_eq!(expected, result);
    }
}
