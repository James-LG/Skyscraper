mod symbols;
mod helpers;

use crate::vecpointer::VecPointer;
pub use symbols::Symbol;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexError {
}

/// Tokenize a string of HTML into Symbols used for parsing later on.
pub fn lex(text: &str) -> Result<Vec<Symbol>, LexError> {
    let mut symbols: Vec<Symbol> = Vec::new();

    let chars = text.chars().collect();
    let mut pointer = VecPointer::new(chars);

    let mut has_open_tag = false;

    while let Some(c) = pointer.current() {
        if let Some(s) = helpers::is_comment(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = helpers::is_start_tag(&mut pointer) {
            has_open_tag = true;
            symbols.push(s);
        } else if let Some(s) = helpers::is_end_tag(&mut pointer) {
            has_open_tag = true;
            symbols.push(s);
        } else if let Some(s) = helpers::is_tag_close_and_end(&mut pointer) {
            has_open_tag = false;
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
        } else if let Some(s) = helpers::is_text(&mut pointer, has_open_tag) {
            symbols.push(s);
        } else {
            if !c.is_whitespace(){
                // Unknown symbol, move on ¯\_(ツ)_/¯
                eprintln!("Unknown HTML symbol {}", c);
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
            Symbol::StartTag(String::from("node")),
            Symbol::TagClose,
            Symbol::Text(String::from("1")),
            Symbol::EndTag(String::from("node")),
            Symbol::TagClose
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
            Symbol::StartTag(String::from("script")),
            Symbol::Identifier(String::from("defer")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("script")),
            Symbol::TagClose
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
            Symbol::StartTag(String::from("script")),
            Symbol::Identifier(String::from("json")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from(r#"{"hello":"world"}"#)),
            Symbol::TagClose,
            Symbol::EndTag(String::from("script")),
            Symbol::TagClose
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
            Symbol::StartTag(String::from("start-tag")),
            Symbol::Identifier(String::from("id")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("bean")),
            Symbol::TagClose,
            Symbol::Comment(String::from("comment")),
            Symbol::StartTag(String::from("inner")),
            Symbol::TagCloseAndEnd,
            Symbol::Text(String::from("hello")),
            Symbol::EndTag(String::from("end-tag")),
            Symbol::TagClose];

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
            Symbol::StartTag(String::from("!DOCTYPE")),
            Symbol::Identifier(String::from("html")),
            Symbol::TagClose,
            Symbol::Comment(String::from(" saved from url=(0026)https://www.rust-lang.org/ ")),
            Symbol::StartTag(String::from("html")),
            Symbol::Identifier(String::from("lang")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("en-US")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("head")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("title")),
            Symbol::TagClose,
            Symbol::Text(String::from("Rust Programming Language")),
            Symbol::EndTag(String::from("title")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("meta")),
            Symbol::Identifier(String::from("name")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("viewport")),
            Symbol::Identifier(String::from("content")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("width=device-width,initial-scale=1.0")),
            Symbol::TagClose,
            Symbol::Comment(String::from(" Twitter card ")),
            Symbol::StartTag(String::from("meta")),
            Symbol::Identifier(String::from("name")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("twitter:card")),
            Symbol::Identifier(String::from("content")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("summary")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("head")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("body")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("main")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("section")),
            Symbol::Identifier(String::from("id")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("language-values")),
            Symbol::Identifier(String::from("class")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("green")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("div")),
            Symbol::Identifier(String::from("class")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("w-100 mw-none ph3 mw8-m mw9-l center f3")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("header")),
            Symbol::Identifier(String::from("class")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("pb0")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("h2")),
            Symbol::TagClose,
            Symbol::Text(String::from(r#"
                                Why Rust?
                                "#)),
            Symbol::EndTag(String::from("h2")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("header")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("div")),
            Symbol::Identifier(String::from("class")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("flex-none flex-l")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("section")),
            Symbol::Identifier(String::from("class")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("w-100 pv2 pv0-l mt4")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("h3")),
            Symbol::Identifier(String::from("class")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("f2 f1-l")),
            Symbol::TagClose,
            Symbol::Text(String::from("Performance")),
            Symbol::EndTag(String::from("h3")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("p")),
            Symbol::Identifier(String::from("class")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("f3 lh-copy")),
            Symbol::TagClose,
            Symbol::Text(String::from(r#"
                                    Rust is blazingly fast and memory-efficient: with no runtime or
                                    garbage collector, it can power performance-critical services, run on
                                    embedded devices, and easily integrate with other languages.
                                    "#)),
            Symbol::EndTag(String::from("p")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("section")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("div")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("div")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("section")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("main")),
            Symbol::TagClose,
            Symbol::StartTag(String::from("script")),
            Symbol::Identifier(String::from("src")),
            Symbol::AssignmentSign,
            Symbol::Literal(String::from("./Rust Programming Language_files/languages.js.download")),
            Symbol::TagCloseAndEnd,
            Symbol::EndTag(String::from("body")),
            Symbol::TagClose,
            Symbol::EndTag(String::from("html")),
            Symbol::TagClose,
        ];
        
        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result) {
            assert_eq!(e, r);
        }
    }
}