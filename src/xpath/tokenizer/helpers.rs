use crate::vecpointer::VecPointerRef;

use super::Token;

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleSlash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_double_slash(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some('/'), Some('/')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_add(2);
        return Some(Token::DoubleSlash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Slash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_slash(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('/') = pointer.current() {
        pointer.next();
        return Some(Token::Slash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_open_square_bracket(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('[') = pointer.current() {
        pointer.next();
        return Some(Token::OpenSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_close_square_bracket(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some(']') = pointer.current() {
        pointer.next();
        return Some(Token::CloseSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_open_bracket(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('(') = pointer.current() {
        pointer.next();
        return Some(Token::OpenBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_close_bracket(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some(')') = pointer.current() {
        pointer.next();
        return Some(Token::CloseBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Wildcard [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_wildcard(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('*') = pointer.current() {
        pointer.next();
        return Some(Token::Wildcard);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_double_dot(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some('.'), Some('.')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_add(2);
        return Some(Token::DoubleDot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_dot(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('.') = pointer.current() {
        pointer.next();
        return Some(Token::Dot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AssignmentSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_assignment_sign(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('=') = pointer.current() {
        pointer.next();
        return Some(Token::AssignmentSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AtSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_at_sign(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('@') = pointer.current() {
        pointer.next();
        return Some(Token::AtSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AddSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_add_sign(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('+') = pointer.current() {
        pointer.next();
        return Some(Token::AddSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a MinusSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_minus_sign(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('-') = pointer.current() {
        pointer.next();
        return Some(Token::MinusSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a GreaterThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_greater_than_sign(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('>') = pointer.current() {
        pointer.next();
        return Some(Token::GreaterThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a LessThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_less_than_sign(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some('<') = pointer.current() {
        pointer.next();
        return Some(Token::LessThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleColon [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_double_colon(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let (Some(':'), Some(':')) = (pointer.current(), pointer.peek()) {
        pointer.next_add(2);
        return Some(Token::DoubleColon);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Number [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_number(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some(c) = pointer.current() {
        if c.is_ascii_digit() {
            let mut num = c.to_string();
            while let Some(c) = pointer.next() {
                if c.is_ascii_digit() {
                    num.push(*c);
                } else {
                    break;
                }
            }

            // Check for decimal values
            if let Some('.') = pointer.current() {
                num.push('.');
                while let Some(c) = pointer.next() {
                    if c.is_ascii_digit() {
                        num.push(*c);
                    } else {
                        break;
                    }
                }
            }

            return Some(Token::Number(num.parse::<f32>().unwrap()));
        }
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to an Identifier [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_identifier(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some(c) = pointer.current() {
        // Identifier must start with a letter
        if c.is_alphabetic() {
            let mut id = c.to_string();

            while let Some(c) = pointer.next() {
                // Identifier can contain letters and numbers
                if c.is_alphanumeric() || c == &'-' {
                    id.push(*c);
                } else {
                    break;
                }
            }

            return Some(Token::Identifier(id));
        }
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Text [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_text(pointer: &mut VecPointerRef<char>) -> Option<Token> {
    if let Some(c) = pointer.current() {
        if c == &'"' || c == &'\'' {
            let delimiter = *c;
            let mut text = String::from("");

            while let Some(c) = pointer.next() {
                if c == &delimiter {
                    // Move to next character before exiting.
                    pointer.next();
                    return Some(Token::Text(text));
                } else {
                    text.push(*c);
                }
            }

            pointer.back_add(text.len() + 1);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_number_should_be_some_with_trailing_text() {
        let chars: Vec<char> = "1234abc".chars().collect();
        let symbol = is_number(&mut VecPointerRef::new(&chars)).unwrap();

        if let Token::Number(f) = symbol {
            assert_eq!(1234f32, f);
        } else {
            panic!("Expected number symbol")
        }
    }

    #[test]
    fn is_number_should_capture_decimal() {
        let chars: Vec<char> = "1234.5678".chars().collect();
        let symbol = is_number(&mut VecPointerRef::new(&chars)).unwrap();

        if let Token::Number(f) = symbol {
            assert_eq!(1234.5678f32, f);
        } else {
            panic!("Expected number symbol")
        }
    }

    #[test]
    fn is_number_should_be_none_with_leading_text() {
        let chars: Vec<char> = "abc1234".chars().collect();
        let symbol = is_number(&mut VecPointerRef::new(&chars));

        assert!(symbol.is_none());
    }

    #[test]
    fn is_text_should_capture_quoted_text() {
        let chars: Vec<char> = r###""world""###.chars().collect();
        let pointer = &mut VecPointerRef::new(&chars);
        let symbol = is_text(pointer);

        if let Some(Token::Text(text)) = symbol {
            assert_eq!("world", text);
            matches!(pointer.next(), None);
        } else {
            panic!("Expected text symbol")
        }
    }

    #[test]
    fn is_text_should_capture_single_quoted_text() {
        let chars: Vec<char> = r###"'world'"###.chars().collect();
        let pointer = &mut VecPointerRef::new(&chars);
        let symbol = is_text(pointer);

        if let Some(Token::Text(text)) = symbol {
            assert_eq!("world", text);
            matches!(pointer.next(), None);
        } else {
            panic!("Expected text symbol")
        }
    }

    #[test]
    fn is_text_should_not_capture_mismatched_quoted_text() {
        let chars: Vec<char> = r###""world'"###.chars().collect();
        let pointer = &mut VecPointerRef::new(&chars);
        let symbol = is_text(pointer);

        matches!(symbol, None);
        matches!(pointer.current(), Some('"')); // Assert cursor was not moved.
    }
}
