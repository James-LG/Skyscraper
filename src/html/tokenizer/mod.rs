mod helpers;
mod tokens;

use crate::vecpointer::VecPointerRef;
use log::error;
use thiserror::Error;
pub use tokens::Token;

#[derive(Error, Debug)]
pub enum LexError {}

/// Tokenize a string of HTML into Symbols used for parsing later on.
pub fn lex(text: &str) -> Result<Vec<Token>, LexError> {
    let mut symbols: Vec<Token> = Vec::new();

    let chars: Vec<char> = text.chars().collect();
    let mut pointer = VecPointerRef::new(&chars);

    let mut has_open_tag = false;
    let mut in_script_tag = false;

    while pointer.has_next() {
        if let Some(s) = helpers::is_comment(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_start_tag(&mut pointer) {
            has_open_tag = true;

            // Check if this is a "script" tag so the flag can be set if required
            match &s {
                Token::StartTag(start_tag) => {
                    if start_tag == "script" {
                        in_script_tag = true;
                    }
                }
                token => panic!(
                    "is_start_tag returned {:?} instead of Token::StartTag",
                    token
                ),
            }

            symbols.push(s);
        } else if let Some(s) = helpers::is_end_tag(&mut pointer) {
            has_open_tag = true;
            in_script_tag = false;
            symbols.push(s);
        } else if let Some(s) = helpers::is_tag_close_and_end(&mut pointer) {
            has_open_tag = false;
            in_script_tag = false;
            symbols.push(s);
        } else if let Some(s) = helpers::is_tag_close(&mut pointer) {
            has_open_tag = false;
            symbols.push(s);
        } else if let Some(s) = helpers::is_assignment_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_literal(&mut pointer, has_open_tag) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_identifier(&mut pointer, has_open_tag) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_text(&mut pointer, has_open_tag, in_script_tag) {
            symbols.push(s);
        } else {
            if let Some(c) = pointer.current() {
                if *c != ' ' {
                    println!("Unknown symbol {}", c);
                }
                if !c.is_whitespace() {
                    // Unknown symbol, move on ¯\_(ツ)_/¯
                    error!("Unknown HTML symbol {}", c);
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
    fn lex_should_work_with_single_char_text() {
        // arrange
        let text = "<node>1</node>";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::StartTag(String::from("node")),
            Token::TagClose,
            Token::Text(String::from("1")),
            Token::EndTag(String::from("node")),
            Token::TagClose,
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_should_handle_attribute_without_value() {
        // arrange
        let text = "<script defer></script>";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::StartTag(String::from("script")),
            Token::Identifier(String::from("defer")),
            Token::TagClose,
            Token::EndTag(String::from("script")),
            Token::TagClose,
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_should_handle_encoded_json() {
        // arrange
        let text = r###"<script json='{"hello":"world"}'></script>"###;

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::StartTag(String::from("script")),
            Token::Identifier(String::from("json")),
            Token::AssignmentSign,
            Token::Literal(String::from(r#"{"hello":"world"}"#)),
            Token::TagClose,
            Token::EndTag(String::from("script")),
            Token::TagClose,
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_should_handle_end_tag_with_whitespace() {
        // arrange
        let text = r#"<node>1</node
            >"#;

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::StartTag(String::from("node")),
            Token::TagClose,
            Token::Text(String::from("1")),
            Token::EndTag(String::from("node")),
            Token::TagClose,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn lex_works() {
        // arrange
        let text = "<start-tag id=\"bean\"><!--comment--><inner/>hello</end-tag>";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Token::StartTag(String::from("start-tag")),
            Token::Identifier(String::from("id")),
            Token::AssignmentSign,
            Token::Literal(String::from("bean")),
            Token::TagClose,
            Token::Comment(String::from("comment")),
            Token::StartTag(String::from("inner")),
            Token::TagCloseAndEnd,
            Token::Text(String::from("hello")),
            Token::EndTag(String::from("end-tag")),
            Token::TagClose,
        ];

        assert_eq!(expected, result);
    }
}
