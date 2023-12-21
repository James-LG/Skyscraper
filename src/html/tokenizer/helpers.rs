use once_cell::sync::Lazy;

use crate::vecpointer::VecPointerRef;

use super::Token;

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a StartTag [Token](Token).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// StartTag is defined as `<{{String}}`
///
/// Has additional checks to make sure it is not an end tag.
pub fn is_start_tag(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some('<'), Some(c2)) = (pointer.current(), pointer.peek()) {
        let c2 = *c2;
        if c2 != '/' && !c2.is_whitespace() {
            let mut name: Vec<char> = Vec::new();
            loop {
                match pointer.next() {
                    Some(' ') | Some('>') | Some('/') => break,
                    Some(c) if c.is_whitespace() => break,
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to an EndTag [Token](Token).
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
                Some(c) if c.is_whitespace() => break,
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a Comment [Token](Token).
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to the end of a Comment [Token](Token).
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a TagClose [Token](Token).
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a TagCloseAndEnd [Token](Token).
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a AssignmentSign [Token](Token).
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a Literal [Token](Token).
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

/// List of characters that end an Identifier [Token](Token).
static INAVLID_ID_CHARS: Lazy<Vec<char>> = Lazy::new(|| vec!['<', '>', '/', '=', '"']);

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a Identifier [Token](Token).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// Identifier is defined as any text inside a tag definition.
pub fn is_identifier(pointer: &mut VecPointerRef<char>, has_open_tag: bool) -> Option<Token> {
    fn valid_char(c: &char) -> bool {
        !c.is_whitespace() && !INAVLID_ID_CHARS.contains(c)
    }

    if !has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        if valid_char(c) {
            let mut text: Vec<char> = vec![*c];
            loop {
                match pointer.next() {
                    Some(c) if !valid_char(c) => break,
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

/// Checks if the [VecPointerRef](VecPointerRef) is currently pointing to a Text [Token](Token).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
///
/// Text is defined as any text outside a tag definition.
pub fn is_text(
    pointer: &mut VecPointerRef<char>,
    has_open_tag: bool,
    in_script_tag: bool,
) -> Option<Token> {
    if has_open_tag {
        return None;
    }

    if let Some(c) = pointer.current() {
        let c = *c;
        let start_index = pointer.index;
        // If character is not '<', or if it is, make sure its not a start or end tag.
        if c != '<' || (is_end_tag(pointer).is_none() && is_start_tag(pointer).is_none()) {
            let mut buffer: Vec<char> = vec![c];
            loop {
                match pointer.next() {
                    Some('<') => {
                        let pointer_index = pointer.index;

                        // In a script tag the *only* thing that can end a text is an end script tag.
                        if in_script_tag {
                            if let Some(end_tag) = is_end_tag(pointer) {
                                match end_tag {
                                    Token::EndTag(end_tag) => {
                                        if end_tag == "script" {
                                            // We can finally close the text
                                            pointer.index = pointer_index;
                                            break;
                                        }
                                    }
                                    token => panic!(
                                        "is_end_tag returned {:?} instead of Token::EndTag",
                                        token
                                    ),
                                }
                            }
                        } else {
                            // The current tag can end or a new tag can be started mid-text.
                            if is_end_tag(pointer).is_some() || is_start_tag(pointer).is_some() {
                                // Start or end tag was matched meaning we've moved the pointer up;
                                // reset it now so it can be matched in the main tokenizer loop.
                                pointer.index = pointer_index;
                                break;
                            }
                        }

                        // If the loop hasn't been broken at this point, add the '<' and move on.
                        pointer.index = pointer_index;
                        buffer.push('<');
                    }
                    Some('\n') => {
                        // Text is allowed to start with a new line, but not allowed to contain one mid-sequence.
                        break;
                    }
                    Some(c) => {
                        buffer.push(*c);
                    }
                    None => break,
                };
            }

            let text: String = buffer.into_iter().collect();
            return Some(Token::Text(text));
        } else {
            // Start or end tag was matched meaning we've moved the pointer up;
            // reset it now so it can be matched in the main tokenizer loop.
            pointer.index = start_index;
            return None;
        }
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
        assert_eq!(result, Token::StartTag(String::from("a")));
        assert_eq!(pointer.index, 2);
    }

    #[test]
    fn is_start_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_start_tag(&mut pointer);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_end_tag_works() {
        // arrange
        let chars: Vec<char> = "</c>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_tag(&mut pointer).unwrap();

        // assert
        assert_eq!(result, Token::EndTag(String::from("c")));
        assert_eq!(pointer.index, 3);
    }

    #[test]
    fn is_end_tag_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_tag(&mut pointer);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_comment_works() {
        // arrange
        let chars: Vec<char> = "<!--bean is-nice -->".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_comment(&mut pointer).unwrap();

        // assert
        assert_eq!(result, Token::Comment(String::from("bean is-nice ")));
        assert_eq!(pointer.index, 20);
    }

    #[test]
    fn is_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_comment(&mut pointer);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_end_comment_works() {
        // arrange
        let chars: Vec<char> = "-->".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(result, true);
        assert_eq!(pointer.index, 3);
    }

    #[test]
    fn is_end_comment_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_end_comment(&mut pointer);

        // assert
        assert_eq!(result, false);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_tag_close_works() {
        // arrange
        let chars: Vec<char> = ">".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close(&mut pointer).unwrap();

        // assert
        assert_eq!(result, Token::TagClose);
        assert_eq!(pointer.index, 1);
    }

    #[test]
    fn is_tag_close_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close(&mut pointer);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_tag_close_and_end_works() {
        // arrange
        let chars: Vec<char> = "/>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close_and_end(&mut pointer).unwrap();

        // assert
        assert_eq!(result, Token::TagCloseAndEnd);
        assert_eq!(pointer.index, 2);
    }

    #[test]
    fn is_tag_close_and_end_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_tag_close_and_end(&mut pointer);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_assignment_sign_works() {
        // arrange
        let chars: Vec<char> = "=".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_assignment_sign(&mut pointer).unwrap();

        // assert
        assert_eq!(result, Token::AssignmentSign);
        assert_eq!(pointer.index, 1);
    }

    #[test]
    fn is_assignment_sign_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_assignment_sign(&mut pointer);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_literal_works_double_quote() {
        // arrange
        let chars: Vec<char> = r###""yo""###.chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_literal(&mut pointer, true).unwrap();

        // assert
        assert_eq!(result, Token::Literal(String::from("yo")));
        assert_eq!(pointer.index, 4);
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
            result,
            Token::Literal(String::from(r#"the cow says "moo"."#))
        );
        assert_eq!(pointer.index, 23);
    }

    #[test]
    fn is_literal_works_single_quote() {
        // arrange
        let chars: Vec<char> = r###"'yo'"###.chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_literal(&mut pointer, true).unwrap();

        // assert
        assert_eq!(result, Token::Literal(String::from("yo")));
        assert_eq!(pointer.index, 4);
    }

    #[test]
    fn is_literal_does_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = "abcd".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_literal(&mut pointer, true);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_identifier_works() {
        // arrange
        let chars: Vec<char> = "foo bar".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_identifier(&mut pointer, true).unwrap();

        // assert
        assert_eq!(result, Token::Identifier(String::from("foo")));
        assert_eq!(pointer.index, 3);
    }

    #[test]
    fn is_identifier_not_move_pointer_if_not_found() {
        // arrange
        let chars: Vec<char> = " ".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_identifier(&mut pointer, true);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_identifier_should_not_match_newline() {
        // arrange
        let chars: Vec<char> = "\n".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_identifier(&mut pointer, true);

        // assert
        assert_eq!(result, None);
    }

    #[test]
    fn is_text_works() {
        // arrange
        let chars: Vec<char> = "foo bar".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, false).unwrap();

        // assert
        assert_eq!(result, Token::Text(String::from("foo bar")));
        assert_eq!(pointer.index, 7);
    }

    #[test]
    fn is_text_not_move_pointer_if_end_tag() {
        // arrange
        let chars: Vec<char> = "</foo>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, false);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_text_not_move_pointer_if_start_tag() {
        // arrange
        let chars: Vec<char> = "<foo>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, false);

        // assert
        assert_eq!(result, None);
        assert_eq!(pointer.index, 0);
    }

    #[test]
    fn is_text_should_not_end_on_floating_triangle_bracket() {
        // arrange
        let chars: Vec<char> = "foo > bar < baz".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, false).unwrap();

        // assert
        assert_eq!(result, Token::Text(String::from("foo > bar < baz")));
        assert_eq!(pointer.index, 15);
    }

    #[test]
    fn is_text_should_end_on_tag_end() {
        // arrange
        let chars: Vec<char> = "foo > bar </baz>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, false).unwrap();

        // assert
        assert_eq!(result, Token::Text(String::from("foo > bar ")));
        assert_eq!(pointer.index, 10);
    }

    #[test]
    fn is_text_should_allow_tag_like_strings_in_script_tags() {
        // arrange
        let chars: Vec<char> = "foo<bar></baz>".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, true).unwrap();

        // assert
        assert_eq!(result, Token::Text(String::from("foo<bar></baz>")));
        assert_eq!(pointer.index, 14);
    }

    #[test]
    fn is_text_should_terminate_on_newline() {
        // arrange
        let chars: Vec<char> = "foo\nbar".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, true).unwrap();

        // assert
        assert_eq!(result, Token::Text(String::from("foo")));
        assert_eq!(pointer.index, 3);
    }

    #[test]
    fn is_text_should_capture_starting_new_line_and_whitespace() {
        // arrange
        let chars: Vec<char> = "\n\t\t".chars().collect();
        let mut pointer = VecPointerRef::new(&chars);

        // act
        let result = is_text(&mut pointer, false, true).unwrap();

        // assert
        assert_eq!(result, Token::Text(String::from("\n\t\t")));
        assert_eq!(pointer.index, 3);
    }
}
