mod reader {
    pub struct TextPointer {
        text: Vec<char>,
        pub index: usize,
    }
    
    impl TextPointer {
        pub fn new(text: &str) -> TextPointer {
            let text: Vec<char> = text.chars().collect();
            TextPointer {
                text,
                index: 0,
            }
        }

        pub fn current(&self) -> Option<char> {
            self.get_char(self.index)
        }

        pub fn next(&mut self) -> Option<char> {
            self.index += 1;
            self.current()
        }

        pub fn next_add(&mut self, i: usize) -> Option<char> {
            self.index += i;
            self.current()
        }

        pub fn peek(&self) -> Option<char> {
            self.get_char(self.index + 1)
        }

        pub fn peek_add(&self, i: usize) -> Option<char> {
            self.get_char(self.index + i)
        }

        fn get_char(&self, index: usize) -> Option<char> {
            if index >= self.text.len() {
                return None;
            }
            Some(self.text[index])
        }
    }
}

use reader::TextPointer;

/// Enum representing all possible symbols output by the lexer.
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Symbol {
    /// The start of a new tag. Example: `<{{string}}`.
    StartTag(String),

    /// The start of an end tag. Example: `</{{string}}`.
    EndTag(String),

    /// End *and* close a tag. Example: `/>`.
    TagCloseAndEnd,

    /// End a tag. Example: `>`.
    TagClose,

    /// Assignment sign. Example: `=`.
    AssignmentSign,

    /// A quoted string literal. Contained string does not include quotes. Example: `"{{string}}"`.
    Literal(String),

    /// Text contained in tags.
    Text(String),

    /// An identifier written in a tag declaration.
    Identifier(String),

    /// Xml comments. Example: `<!--{{string}}-->`.
    Comment(String),
}

pub fn lex(text: &str) -> Result<Vec<Symbol>, &'static str> {
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut pointer = TextPointer::new(text);

    let mut has_open_tag = false;

    loop {
        match pointer.current() {
            Some(c) => {
                if let Some(s) = is_comment(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_start_tag(&mut pointer) {
                    has_open_tag = true;
                    symbols.push(s);
                } else if let Some(s) = is_end_tag(&mut pointer) {
                    has_open_tag = true;
                    symbols.push(s);
                } else if let Some(s) = is_tag_close_and_end(&mut pointer) {
                    has_open_tag = false;
                    symbols.push(s);
                } else if let Some(s) = is_tag_close(&mut pointer) {
                    has_open_tag = false;
                    symbols.push(s);
                } else if let Some(s) = is_assignment_sign(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_literal(&mut pointer, has_open_tag) {
                    symbols.push(s);
                } else if let Some(s) = is_identifier(&mut pointer, has_open_tag) {
                    symbols.push(s);
                } else if let Some(s) = is_text(&mut pointer, has_open_tag) {
                    symbols.push(s);
                } else {
                    if !c.is_whitespace(){
                        // Unknown symbol, move on ¯\_(ツ)_/¯
                        eprintln!("Unknown symbol {}", c);
                    }
                    pointer.next();
                }
            },
            None => break,
        };
    }
    Ok(symbols)
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a StartTag [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// StartTag is defined as `<{{String}}`
/// 
/// Has additional checks to make sure it is not an end tag.
fn is_start_tag(pointer: &mut TextPointer) -> Option<Symbol> {
    if let (Some(c1), Some(c2)) = (pointer.current(), pointer.peek()) {
        if c1 == '<' && c2 != '/' {
            let mut name: Vec<char> = Vec::new();
            loop {
                match pointer.next() {
                    Some(' ') | Some('>') | Some('/') => break,
                    Some(c) => {
                        name.push(c);
                    },
                    None => break,
                };
            }
            let name: String = name.into_iter().collect();
    
            return Some(Symbol::StartTag(name));
        }

        return None;
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to an EndTag [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// EndTag is defined as `</{{String}}`
fn is_end_tag(pointer: &mut TextPointer) -> Option<Symbol> {
    if let (Some(c1), Some(c2)) = (pointer.current(), pointer.peek()) {
        if c1 == '<' && c2 == '/' {
            pointer.next(); // peeked before, move up now
            
            let mut name: Vec<char> = Vec::new();
            loop {
                match pointer.next() {
                    Some(' ') | Some('>') => break,
                    Some(c) => {
                        name.push(c);
                    },
                    None => break,
                };
            }
            let name: String = name.into_iter().collect();
    
            return Some(Symbol::EndTag(name));
        }
        return None;
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Comment [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// Comment is defined as `<!--{{String}}-->`
fn is_comment(pointer: &mut TextPointer) -> Option<Symbol> {
    if let (Some(c1), Some(c2), Some(c3), Some(c4)) = (pointer.current(), pointer.peek(), pointer.peek_add(2), pointer.peek_add(3)) {
        if c1 == '<' && c2 == '!' && c3 == '-' && c4 == '-' {
            pointer.next_add(3); // peeked before, move up now

            let mut text: Vec<char> = Vec::new();
            loop {
                match pointer.next() {
                    Some(c) => {
                        if is_end_comment(pointer) {
                            let name: String = text.into_iter().collect();
                            return Some(Symbol::Comment(name));
                        }
                        text.push(c);
                    },
                    None => break,
                };
            }
        }
        return None;
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to the end of a Comment [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// This is a helper method not used directly in the lexer.
/// 
/// The end of a comment is defined as `-->`
fn is_end_comment(pointer: &mut TextPointer) -> bool {
    if let (Some(c1), Some(c2), Some(c3)) = (pointer.current(), pointer.peek(), pointer.peek_add(2)) {
        if c1 == '-' && c2 == '-' && c3 == '>' {
            pointer.next_add(3); // peeked before, move up now; 2+1 to end after comment
    
            return true;
        }
        return false;
    }
    false
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a TagClose [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// TagClose is defined as `>`
fn is_tag_close(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c == '>' {
            pointer.next(); // move up for later
            return Some(Symbol::TagClose);
        }
        return None;
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a TagCloseAndEnd [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// TagCloseAndEnd is defined as `/>`
fn is_tag_close_and_end(pointer: &mut TextPointer) -> Option<Symbol> {
    if let (Some(c1), Some(c2)) = (pointer.current(), pointer.peek()) {
        if c1 == '/' && c2 == '>' {
            pointer.next_add(2); // move up for later
            return Some(Symbol::TagCloseAndEnd);
        }
        return None;
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AssignmentSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// AssignmentSign is defined as `=`
fn is_assignment_sign(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c == '=' {
            pointer.next(); // move up for later
            return Some(Symbol::AssignmentSign);
        }
        return None;
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Literal [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// Literal is defined as `"{{String}}"` inside a tag definition.
fn is_literal(pointer: &mut TextPointer, has_open_tag: bool) -> Option<Symbol> {
    if !has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        if c == '"' {            
            let mut text: Vec<char> = Vec::new();
            loop {
                match pointer.next() {
                    Some('"') => break,
                    Some(c) => {
                        text.push(c);
                    },
                    None => break,
                };
            }
            let name: String = text.into_iter().collect();

            pointer.next(); // skip over closing `"`
    
            return Some(Symbol::Literal(name));
        }
        return None;
    }
    None
}

lazy_static! {
    /// List of characters that end an Identifier [Symbol](Symbol).
    static ref INAVLID_ID_CHARS: Vec<char> = vec![' ', '<', '>', '/', '=', '"'];
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Identifier [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// Identifier is defined as any text inside a tag definition.
fn is_identifier(pointer: &mut TextPointer, has_open_tag: bool) -> Option<Symbol> {
    if !has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        if !INAVLID_ID_CHARS.contains(&c) {
            let mut text: Vec<char> = Vec::new();
            text.push(c);
            loop {
                match pointer.next() {
                    Some(c) if INAVLID_ID_CHARS.contains(&c) => break,
                    Some(c) => {
                        text.push(c);
                    },
                    None => break,
                };
            }
            let name: String = text.into_iter().collect();
    
            return Some(Symbol::Identifier(name));
        }
        return None;
    }
    None
}

lazy_static! {
    /// List of characters that end a Text [Symbol](Symbol).
    static ref INAVLID_TEXT_CHARS: Vec<char> = vec!['<', '>'];
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Text [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
/// 
/// Text is defined as any text outside a tag definition.
fn is_text(pointer: &mut TextPointer, has_open_tag: bool) -> Option<Symbol> {
    if has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        if !INAVLID_TEXT_CHARS.contains(&c) {
            let start_index = pointer.index;
            let mut has_non_whitespace = false;

            let mut text: Vec<char> = Vec::new();
            text.push(c);
            loop {
                match pointer.next() {
                    Some(c) if INAVLID_TEXT_CHARS.contains(&c) => break,
                    Some(c) => {
                        if !c.is_whitespace() {
                            has_non_whitespace = true;
                        }

                        text.push(c);
                    },
                    None => break,
                };
            }
            let name: String = text.into_iter().collect();
    
            if has_non_whitespace {
                return Some(Symbol::Text(name));
            } else {
                // roll back pointer
                pointer.index = start_index;
                return None;
            }
        }
        return None;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_start_tag_finds_and_moves_pointer() {
        // arrange
        let mut pointer = TextPointer::new("<a>");

        // act
        let result = is_start_tag(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::StartTag(String::from("a")), result);
        assert_eq!(2, pointer.index);
    }

    #[test]
    fn is_start_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_start_tag(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_end_tag_works() {
        // arrange
        let mut pointer = TextPointer::new("</c>");

        // act
        let result = is_end_tag(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::EndTag(String::from("c")), result);
        assert_eq!(3, pointer.index);
    }

    #[test]
    fn is_end_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_end_tag(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_comment_works() {
        // arrange
        let mut pointer = TextPointer::new("<!--bean is-nice -->");

        // act
        let result = is_comment(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::Comment(String::from("bean is-nice ")), result);
        assert_eq!(20, pointer.index);
    }

    #[test]
    fn is_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_comment(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_end_comment_works() {
        // arrange
        let mut pointer = TextPointer::new("-->");

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(true, result);
        assert_eq!(3, pointer.index);
    }

    #[test]
    fn is_end_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(false, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_tag_close_works() {
        // arrange
        let mut pointer = TextPointer::new(">");

        // act
        let result = is_tag_close(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::TagClose, result);
        assert_eq!(1, pointer.index);
    }

    #[test]
    fn is_tag_close_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_tag_close(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_tag_close_and_end_works() {
        // arrange
        let mut pointer = TextPointer::new("/>");

        // act
        let result = is_tag_close_and_end(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::TagCloseAndEnd, result);
        assert_eq!(2, pointer.index);
    }

    #[test]
    fn is_tag_close_and_end_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_tag_close_and_end(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_assignment_sign_works() {
        // arrange
        let mut pointer = TextPointer::new("=");

        // act
        let result = is_assignment_sign(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::AssignmentSign, result);
        assert_eq!(1, pointer.index);
    }

    #[test]
    fn is_assignment_sign_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_assignment_sign(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_literal_works() {
        // arrange
        let mut pointer = TextPointer::new("\"yo\"");

        // act
        let result = is_literal(&mut pointer, true).unwrap();

        // assert
        assert_eq!(Symbol::Literal(String::from("yo")), result);
        assert_eq!(4, pointer.index);
    }

    #[test]
    fn is_literal_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_literal(&mut pointer, true);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_identifier_works() {
        // arrange
        let mut pointer = TextPointer::new("foo bar");

        // act
        let result = is_identifier(&mut pointer, true).unwrap();

        // assert
        assert_eq!(Symbol::Identifier(String::from("foo")), result);
        assert_eq!(3, pointer.index);
    }

    #[test]
    fn is_identifier_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new(" ");

        // act
        let result = is_identifier(&mut pointer, true);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_text_works() {
        // arrange
        let mut pointer = TextPointer::new("foo bar");

        // act
        let result = is_text(&mut pointer, false).unwrap();

        // assert
        assert_eq!(Symbol::Text(String::from("foo bar")), result);
        assert_eq!(7, pointer.index);
    }

    #[test]
    fn is_text_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("<");

        // act
        let result = is_text(&mut pointer, false);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
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