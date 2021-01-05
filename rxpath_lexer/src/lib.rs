#[macro_use]
extern crate lazy_static;

mod reader {
    pub struct TextPointer {
        text: Vec<char>,
        index: usize,
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

        pub fn get_index(&self) -> usize {
            self.index
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

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Symbol {
    StartTag(String),
    EndTag(String),
    TagClose,
    AssignmentSign,
    Text(String),
    Identifier(String),
    Comment(String),
}

pub fn lex(text: &str) -> Result<Vec<Symbol>, &'static str> {
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut pointer = TextPointer::new(text);

    loop {
        match pointer.current() {
            Some(c) if !c.is_whitespace() => {
                println!("Pointing at {} [i={}]", c, pointer.get_index());
                if let Some(s) = is_comment(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_start_tag(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_end_tag(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_tag_close(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_assignment_sign(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_text(&mut pointer) {
                    symbols.push(s);
                } else if let Some(s) = is_identifier(&mut pointer) {
                    symbols.push(s);
                } else {
                    // Unknown symbol, move on ¯\_(ツ)_/¯
                    eprintln!("Unknown symbol {}", c);
                    pointer.next();
                }
            },
            Some(_) => {
                pointer.next();
                continue
            },
            None => break,
        };
    }
    Ok(symbols)
}

fn is_start_tag(pointer: &mut TextPointer) -> Option<Symbol> {
    if let (Some(c1), Some(c2)) = (pointer.current(), pointer.peek()) {
        if c1 == '<' && c2 != '/' {
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
    
            return Some(Symbol::StartTag(name));
        }

        return None;
    }
    None
}

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

fn is_text(pointer: &mut TextPointer) -> Option<Symbol> {
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
    
            return Some(Symbol::Text(name));
        }
        return None;
    }
    None
}

lazy_static! {
    static ref INAVLID_ID_CHARS: Vec<char> = vec![' ', '<', '>', '/', '=', '"'];
}

fn is_identifier(pointer: &mut TextPointer) -> Option<Symbol> {
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
        assert_eq!(2, pointer.get_index());
    }

    #[test]
    fn is_start_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_start_tag(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn is_end_tag_works() {
        // arrange
        let mut pointer = TextPointer::new("</c>");

        // act
        let result = is_end_tag(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::EndTag(String::from("c")), result);
        assert_eq!(3, pointer.get_index());
    }

    #[test]
    fn is_end_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_end_tag(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn is_comment_works() {
        // arrange
        let mut pointer = TextPointer::new("<!--bean is-nice -->");

        // act
        let result = is_comment(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::Comment(String::from("bean is-nice ")), result);
        assert_eq!(20, pointer.get_index());
    }

    #[test]
    fn is_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_comment(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn is_end_comment_works() {
        // arrange
        let mut pointer = TextPointer::new("-->");

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(true, result);
        assert_eq!(3, pointer.get_index());
    }

    #[test]
    fn is_end_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(false, result);
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn is_tag_close_works() {
        // arrange
        let mut pointer = TextPointer::new(">");

        // act
        let result = is_tag_close(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::TagClose, result);
        assert_eq!(1, pointer.get_index());
    }

    #[test]
    fn is_tag_close_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_tag_close(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn is_assignment_sign_works() {
        // arrange
        let mut pointer = TextPointer::new("=");

        // act
        let result = is_assignment_sign(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::AssignmentSign, result);
        assert_eq!(1, pointer.get_index());
    }

    #[test]
    fn is_assignment_sign_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_assignment_sign(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn is_text_works() {
        // arrange
        let mut pointer = TextPointer::new("\"yo\"");

        // act
        let result = is_text(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::Text(String::from("yo")), result);
        assert_eq!(4, pointer.get_index());
    }

    #[test]
    fn is_text_does_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new("abcd");

        // act
        let result = is_text(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn is_identifier_works() {
        // arrange
        let mut pointer = TextPointer::new("yo");

        // act
        let result = is_identifier(&mut pointer).unwrap();

        // assert
        assert_eq!(Symbol::Identifier(String::from("yo")), result);
        assert_eq!(2, pointer.get_index());
    }

    #[test]
    fn is_identifier_not_move_pointer_if_not_found() {
        // arrange
        let mut pointer = TextPointer::new(" ");

        // act
        let result = is_identifier(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.get_index());
    }

    #[test]
    fn lex_works() {
        // arrange
        let text = "<start-tag id=\"bean\"><!--comment--><inner>\"<fake>\"</inner>hello</end-tag>";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Symbol::StartTag(String::from("start-tag")),
            Symbol::Identifier(String::from("id")),
            Symbol::AssignmentSign,
            Symbol::Text(String::from("bean")),
            Symbol::TagClose,
            Symbol::Comment(String::from("comment")),
            Symbol::StartTag(String::from("inner")),
            Symbol::TagClose,
            Symbol::Text(String::from("<fake>")),
            Symbol::EndTag(String::from("inner")),
            Symbol::TagClose,
            Symbol::Identifier(String::from("hello")),
            Symbol::EndTag(String::from("end-tag")),
            Symbol::TagClose];

        assert_eq!(expected, result);
    }
}