mod symbols;
mod helpers;

use crate::vecpointer::VecPointer;
pub use symbols::Symbol;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexError {
}

pub fn lex(text: &str) -> Result<Vec<Symbol>, LexError> {
    let mut symbols: Vec<Symbol> = Vec::new();

    let chars = text.chars().collect();
    let mut pointer = VecPointer::new(chars);

    while let Some(c) = pointer.current() {
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
        } else if let Some(s) = helpers::is_identifier(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_text(&mut pointer) {
            symbols.push(s);
        } else {
            if !c.is_whitespace(){
                // Unknown symbol, move on ¯\_(ツ)_/¯
                eprintln!("Unknown XPath symbol {}", c);
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
            Symbol::DoubleSlash,
            Symbol::Identifier(String::from("bookstore")),
            Symbol::Slash,
            Symbol::Identifier(String::from("book")),
            Symbol::OpenSquareBracket,
            Symbol::Number(1.0),
            Symbol::CloseSquareBracket,
            Symbol::Slash,
            Symbol::Identifier(String::from("page")),
            Symbol::OpenSquareBracket,
            Symbol::Identifier(String::from("last")),
            Symbol::OpenBracket,
            Symbol::CloseBracket,
            Symbol::MinusSign,
            Symbol::Number(1.0),
            Symbol::CloseSquareBracket,
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
            Symbol::Slash,
            Symbol::Identifier(String::from("bookstore")),
            Symbol::Slash,
            Symbol::Identifier(String::from("book")),
            Symbol::OpenSquareBracket,
            Symbol::Identifier(String::from("price")),
            Symbol::GreaterThanSign,
            Symbol::Number(35.0),
            Symbol::CloseSquareBracket,
            Symbol::Slash,
            Symbol::Identifier(String::from("price"))
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
            Symbol::DoubleSlash,
            Symbol::Identifier(String::from("a")),
            Symbol::OpenSquareBracket,
            Symbol::AtSign,
            Symbol::Identifier(String::from("hello")),
            Symbol::AssignmentSign,
            Symbol::Text(String::from("world")),
            Symbol::CloseSquareBracket,
        ];

        assert_eq!(expected, result);
    }
}