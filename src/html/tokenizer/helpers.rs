use crate::vecpointer::VecPointerRef;

use super::Token;

/// Checks if the [TextPointer](TextPointer) is currently pointing to a StartTag [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// StartTag is defined as `<{{String}}`
///
/// Has additional checks to make sure it is not an end tag.
pub fn is_start_tag(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some('<'), Some(c2)) = (pointer.current(), pointer.peek()) {
        if *c2 != '/' {
            let mut name: Vec<char> = Vec::new();
            loop {
                match pointer.next() {
                    Some(' ') | Some('>') | Some('/') => break,
                    Some(c) => {
                        name.push(*c);
                    }
                    None => break,
                };
            }
            let name: String = name.into_iter().collect();

            return Some(Token::StartTag(name));
        }

        return None;
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to an EndTag [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// EndTag is defined as `</{{String}}`
pub fn is_end_tag(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some('<'), Some('/')) = (pointer.current(), pointer.peek()) {
        pointer.next(); // peeked before, move up now

        let mut name: Vec<char> = Vec::new();
        loop {
            match pointer.next() {
                Some(' ') | Some('>') => break,
                Some(c) => {
                    name.push(*c);
                }
                None => break,
            };
        }
        let name: String = name.into_iter().collect();

        return Some(Token::EndTag(name));
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Comment [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// Comment is defined as `<!--{{String}}-->`
pub fn is_comment(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some('<'), Some('!'), Some('-'), Some('-')) = (
        pointer.current(),
        pointer.peek(),
        pointer.peek_add(2),
        pointer.peek_add(3),
    ) {
        pointer.next_add(3); // peeked before, move up now

        let mut text: Vec<char> = Vec::new();
        while let Some(c) = pointer.next() {
            let c = *c;
            if is_end_comment(pointer) {
                let name: String = text.into_iter().collect();
                return Some(Token::Comment(name));
            }
            text.push(c);
        }
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to the end of a Comment [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// This is a helper method not used directly in the lexer.
///
/// The end of a comment is defined as `-->`
pub fn is_end_comment(pointer: &mut VecPointerRef<char>) -> bool {
    if let (Some('-'), Some('-'), Some('>')) =
        (pointer.current(), pointer.peek(), pointer.peek_add(2))
    {
        pointer.next_add(3); // peeked before, move up now; 2+1 to end after comment

        return true;
    }
    false
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a TagClose [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// TagClose is defined as `>`
pub fn is_tag_close(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('>') = pointer.current() {
        pointer.next(); // move up for later
        return Some(Token::TagClose);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a TagCloseAndEnd [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// TagCloseAndEnd is defined as `/>`
pub fn is_tag_close_and_end(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some('/'), Some('>')) = (pointer.current(), pointer.peek()) {
        pointer.next_add(2); // move up for later
        return Some(Token::TagCloseAndEnd);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AssignmentSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// AssignmentSign is defined as `=`
pub fn is_assignment_sign(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('=') = pointer.current() {
        pointer.next(); // move up for later
        return Some(Token::AssignmentSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Literal [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// Literal is defined as `"{{String}}"` inside a tag definition.
pub fn is_literal(pointer: &mut VecPointerRef<char>, has_open_tag: bool) -> Option<Token> {
    if !has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        let c = *c;
        if c == '"' || c == '\'' {
            let start_quote = c;
            let mut text: Vec<char> = Vec::new();
            let mut escape = false;
            loop {
                match pointer.next() {
                    Some('\\') => escape = true,
                    Some(c) => {
                        // If this quote matches the starting quote, break the loop
                        if !escape && (*c == '"' || *c == '\'') && start_quote == *c {
                            break;
                        }
                        // Otherwise push the different quote to the text
                        else {
                            text.push(*c);
                        }
                        escape = false;
                    }
                    None => break,
                };
            }

            let name: String = text.into_iter().collect();

            pointer.next(); // skip over closing `"`

            return Some(Token::Literal(name));
        }
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
pub fn is_identifier(pointer: &mut VecPointerRef<char>, has_open_tag: bool) -> Option<Token> {
    if !has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        if !INAVLID_ID_CHARS.contains(c) {
            let mut text: Vec<char> = vec![*c];
            loop {
                match pointer.next() {
                    Some(c) if INAVLID_ID_CHARS.contains(c) => break,
                    Some(c) => {
                        text.push(*c);
                    }
                    None => break,
                };
            }
            let name: String = text.into_iter().collect();

            return Some(Token::Identifier(name));
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
pub fn is_text(pointer: &mut VecPointerRef<char>, has_open_tag: bool) -> Option<Token> {
    if has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        if !INAVLID_TEXT_CHARS.contains(c) {
            let start_index = pointer.index;
            let mut has_non_whitespace = !c.is_whitespace();

            let mut buffer: Vec<char> = vec![*c];
            loop {
                match pointer.next() {
                    Some(c) if INAVLID_TEXT_CHARS.contains(c) => break,
                    Some(c) => {
                        if !c.is_whitespace() {
                            has_non_whitespace = true;
                        }

                        buffer.push(*c);
                    }
                    None => break,
                };
            }

            if has_non_whitespace {
                let text: String = buffer.into_iter().collect();
                return Some(Token::Text(text));
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
        let chars: Vec<char> = "<a>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_start_tag(&mut pointer).unwrap();

        // assert
        assert_eq!(Token::StartTag(String::from("a")), result);
        assert_eq!(2, pointer.index);
    }

    #[test]
    fn is_start_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_start_tag(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_end_tag_works() {
        // arrange
        let chars: Vec<char> = "</c>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_tag(&mut pointer).unwrap();

        // assert
        assert_eq!(Token::EndTag(String::from("c")), result);
        assert_eq!(3, pointer.index);
    }

    #[test]
    fn is_end_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_tag(&mut pointer);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_comment_works() {
        // arrange
        let chars: Vec<char> = "<!--bean is-nice -->".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_comment(&mut pointer).unwrap();

        // assert
        assert_eq!(Token::Comment(String::from("bean is-nice ")), result);
        assert_eq!(20, pointer.index);
    }

    #[test]
    fn is_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_comment(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_end_comment_works() {
        // arrange
        let chars: Vec<char> = "-->".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(true, result);
        assert_eq!(3, pointer.index);
    }

    #[test]
    fn is_end_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(false, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_tag_close_works() {
        // arrange
        let chars: Vec<char> = ">".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close(&mut pointer).unwrap();

        // assert
        assert_eq!(Token::TagClose, result);
        assert_eq!(1, pointer.index);
    }

    #[test]
    fn is_tag_close_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_tag_close_and_end_works() {
        // arrange
        let chars: Vec<char> = "/>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close_and_end(&mut pointer).unwrap();

        // assert
        assert_eq!(Token::TagCloseAndEnd, result);
        assert_eq!(2, pointer.index);
    }

    #[test]
    fn is_tag_close_and_end_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close_and_end(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_assignment_sign_works() {
        // arrange
        let chars: Vec<char> = "=".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_assignment_sign(&mut pointer).unwrap();

        // assert
        assert_eq!(Token::AssignmentSign, result);
        assert_eq!(1, pointer.index);
    }

    #[test]
    fn is_assignment_sign_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_assignment_sign(&mut pointer);

        // assert
        assert_eq!(None, result);
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_literal_works_double_quote() {
        // arrange
        let chars: Vec<char> = r###""yo""###.chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_literal(&mut pointer, true).unwrap();

        // assert
        assert_eq!(Token::Literal(String::from("yo")), result);
        assert_eq!(4, pointer.index);
    }

    #[test]
    fn is_literal_works_escaped_quote() {
        // arrange
        let chars: Vec<char> = r###""the cow says \"moo\".""###.chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_literal(&mut pointer, true).unwrap();

        // assert
        assert_eq!(
            Token::Literal(String::from(r#"the cow says "moo"."#)),
            result
        );
        assert_eq!(23, pointer.index);
    }

    #[test]
    fn is_literal_works_single_quote() {
        // arrange
        let chars: Vec<char> = r###"'yo'"###.chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_literal(&mut pointer, true).unwrap();

        // assert
        assert_eq!(Token::Literal(String::from("yo")), result);
        assert_eq!(4, pointer.index);
    }

    #[test]
    fn is_literal_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_literal(&mut pointer, true);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_identifier_works() {
        // arrange
        let chars: Vec<char> = "foo bar".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_identifier(&mut pointer, true).unwrap();

        // assert
        assert_eq!(Token::Identifier(String::from("foo")), result);
        assert_eq!(3, pointer.index);
    }

    #[test]
    fn is_identifier_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = " ".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_identifier(&mut pointer, true);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }

    #[test]
    fn is_text_works() {
        // arrange
        let chars: Vec<char> = "foo bar".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false).unwrap();

        // assert
        assert_eq!(Token::Text(String::from("foo bar")), result);
        assert_eq!(7, pointer.index);
    }

    #[test]
    fn is_text_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "<".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false);

        // assert
        assert!(matches!(result, None));
        assert_eq!(0, pointer.index);
    }
}
