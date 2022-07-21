mod tokens;
mod helpers;

use crate::vecpointer::VecPointerRef;
pub use tokens::Token;
use thiserror::Error;

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
                },
                token => panic!("is_start_tag returned {:?} instead of Token::StartTag", token)
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
                if !c.is_whitespace() {
                    // Unknown symbol, move on ¯\_(ツ)_/¯
                    eprintln!("Unknown HTML symbol {}", c);
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

    #[test]
    fn lex_should_work_with_html() {
        // arrange
        let html = r###"<!DOCTYPE html>
        <!-- saved from url=(0026)https://www.rust-lang.org/ -->
        <html lang="en-US">
            <head>
                <title>Rust Programming Language</title>
                <meta name="viewport" content="width=device-width,initial-scale=1.0">
        
                <!-- Twitter card -->
                <meta name="twitter:card" content="summary">
            </head>
            <body>
                <main>
                    <section id="language-values" class="green">
                        <div class="w-100 mw-none ph3 mw8-m mw9-l center f3">
                            <header class="pb0">
                                <h2>
                                Why Rust?
                                </h2>
                            </header>
                            <div class="flex-none flex-l">
                                <section class="w-100 pv2 pv0-l mt4">
                                    <h3 class="f2 f1-l">Performance</h3>
                                    <p class="f3 lh-copy">
                                    Rust is blazingly fast and memory-efficient: with no runtime or
                                    garbage collector, it can power performance-critical services, run on
                                    embedded devices, and easily integrate with other languages.
                                    </p>
                                </section>
                            </div>
                        </div>
                    </section>
                </main>
                <script src="./Rust Programming Language_files/languages.js.download"/>
            </body>
        </html>"###;

        // act
        let result = lex(html).unwrap();

        // assert
        let expected = vec![
            Token::StartTag(String::from("!DOCTYPE")),
            Token::Identifier(String::from("html")),
            Token::TagClose,
            Token::Comment(String::from(
                " saved from url=(0026)https://www.rust-lang.org/ ",
            )),
            Token::StartTag(String::from("html")),
            Token::Identifier(String::from("lang")),
            Token::AssignmentSign,
            Token::Literal(String::from("en-US")),
            Token::TagClose,
            Token::StartTag(String::from("head")),
            Token::TagClose,
            Token::StartTag(String::from("title")),
            Token::TagClose,
            Token::Text(String::from("Rust Programming Language")),
            Token::EndTag(String::from("title")),
            Token::TagClose,
            Token::StartTag(String::from("meta")),
            Token::Identifier(String::from("name")),
            Token::AssignmentSign,
            Token::Literal(String::from("viewport")),
            Token::Identifier(String::from("content")),
            Token::AssignmentSign,
            Token::Literal(String::from("width=device-width,initial-scale=1.0")),
            Token::TagClose,
            Token::Comment(String::from(" Twitter card ")),
            Token::StartTag(String::from("meta")),
            Token::Identifier(String::from("name")),
            Token::AssignmentSign,
            Token::Literal(String::from("twitter:card")),
            Token::Identifier(String::from("content")),
            Token::AssignmentSign,
            Token::Literal(String::from("summary")),
            Token::TagClose,
            Token::EndTag(String::from("head")),
            Token::TagClose,
            Token::StartTag(String::from("body")),
            Token::TagClose,
            Token::StartTag(String::from("main")),
            Token::TagClose,
            Token::StartTag(String::from("section")),
            Token::Identifier(String::from("id")),
            Token::AssignmentSign,
            Token::Literal(String::from("language-values")),
            Token::Identifier(String::from("class")),
            Token::AssignmentSign,
            Token::Literal(String::from("green")),
            Token::TagClose,
            Token::StartTag(String::from("div")),
            Token::Identifier(String::from("class")),
            Token::AssignmentSign,
            Token::Literal(String::from("w-100 mw-none ph3 mw8-m mw9-l center f3")),
            Token::TagClose,
            Token::StartTag(String::from("header")),
            Token::Identifier(String::from("class")),
            Token::AssignmentSign,
            Token::Literal(String::from("pb0")),
            Token::TagClose,
            Token::StartTag(String::from("h2")),
            Token::TagClose,
            Token::Text(String::from(
                r#"
                                Why Rust?
                                "#,
            )),
            Token::EndTag(String::from("h2")),
            Token::TagClose,
            Token::EndTag(String::from("header")),
            Token::TagClose,
            Token::StartTag(String::from("div")),
            Token::Identifier(String::from("class")),
            Token::AssignmentSign,
            Token::Literal(String::from("flex-none flex-l")),
            Token::TagClose,
            Token::StartTag(String::from("section")),
            Token::Identifier(String::from("class")),
            Token::AssignmentSign,
            Token::Literal(String::from("w-100 pv2 pv0-l mt4")),
            Token::TagClose,
            Token::StartTag(String::from("h3")),
            Token::Identifier(String::from("class")),
            Token::AssignmentSign,
            Token::Literal(String::from("f2 f1-l")),
            Token::TagClose,
            Token::Text(String::from("Performance")),
            Token::EndTag(String::from("h3")),
            Token::TagClose,
            Token::StartTag(String::from("p")),
            Token::Identifier(String::from("class")),
            Token::AssignmentSign,
            Token::Literal(String::from("f3 lh-copy")),
            Token::TagClose,
            Token::Text(String::from(
                r#"
                                    Rust is blazingly fast and memory-efficient: with no runtime or
                                    garbage collector, it can power performance-critical services, run on
                                    embedded devices, and easily integrate with other languages.
                                    "#,
            )),
            Token::EndTag(String::from("p")),
            Token::TagClose,
            Token::EndTag(String::from("section")),
            Token::TagClose,
            Token::EndTag(String::from("div")),
            Token::TagClose,
            Token::EndTag(String::from("div")),
            Token::TagClose,
            Token::EndTag(String::from("section")),
            Token::TagClose,
            Token::EndTag(String::from("main")),
            Token::TagClose,
            Token::StartTag(String::from("script")),
            Token::Identifier(String::from("src")),
            Token::AssignmentSign,
            Token::Literal(String::from(
                "./Rust Programming Language_files/languages.js.download",
            )),
            Token::TagCloseAndEnd,
            Token::EndTag(String::from("body")),
            Token::TagClose,
            Token::EndTag(String::from("html")),
            Token::TagClose,
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result) {
            assert_eq!(e, r);
        }
    }
}
