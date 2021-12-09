use crate::vecpointer::VecPointer;

use super::Symbol;

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleSlash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_double_slash(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let (Some('/'), Some('/')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_add(2);
        return Some(Symbol::DoubleSlash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Slash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_slash(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('/') = pointer.current() {
        pointer.next();
        return Some(Symbol::Slash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_open_square_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('[') = pointer.current() {
        pointer.next();
        return Some(Symbol::OpenSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_close_square_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some(']') = pointer.current() {
        pointer.next();
        return Some(Symbol::CloseSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_open_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('(') = pointer.current() {
        pointer.next();
        return Some(Symbol::OpenBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_close_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some(')') = pointer.current() {
        pointer.next();
        return Some(Symbol::CloseBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Wildcard [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_wildcard(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('*') = pointer.current() {
        pointer.next();
        return Some(Symbol::Wildcard);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_double_dot(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let (Some('.'), Some('.')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_add(2);
        return Some(Symbol::DoubleDot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_dot(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('.') = pointer.current() {
        pointer.next();
        return Some(Symbol::Dot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AssignmentSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_assignment_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('=') = pointer.current() {
        pointer.next();
        return Some(Symbol::AssignmentSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AtSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_at_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('@') = pointer.current() {
        pointer.next();
        return Some(Symbol::AtSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AddSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_add_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('+') = pointer.current() {
        pointer.next();
        return Some(Symbol::AddSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a MinusSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_minus_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('-') = pointer.current() {
        pointer.next();
        return Some(Symbol::MinusSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a GreaterThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_greater_than_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('>') = pointer.current() {
        pointer.next();
        return Some(Symbol::GreaterThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a LessThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_less_than_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('<') = pointer.current() {
        pointer.next();
        return Some(Symbol::LessThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Number [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_number(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c.is_digit(10) {
            let mut num = c.to_string();
            while let Some(c) = pointer.next() {
                if c.is_digit(10) {
                    num.push(c);
                } else {
                    break;
                }
            }

            // Check for decimal values
            if let Some('.') = pointer.current() {
                num.push('.');
                while let Some(c) = pointer.next() {
                    if c.is_digit(10) {
                        num.push(c);
                    } else {
                        break;
                    }
                }
            }

            return Some(Symbol::Number(num.parse::<f32>().unwrap()));
        }
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to an Identifier [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_identifier(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c.is_alphabetic() {
            let mut id = c.to_string();

            while let Some(c) = pointer.next() {
                if c.is_alphabetic() {
                    id.push(c);
                } else {
                    break;
                }
            }

            return Some(Symbol::Identifier(id));
        }
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Text [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
pub fn is_text(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c == '"' || c == '\'' {
            let delimiter = c;
            let mut text = String::from("");

            while let Some(c) = pointer.next() {
                if c == delimiter {
                    // Move to next character before exiting.
                    pointer.next();
                    return Some(Symbol::Text(text));
                } else {
                    text.push(c);
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
        let chars = "1234abc".chars().collect();
        let symbol = is_number(&mut VecPointer::new(chars)).unwrap();

        if let Symbol::Number(f) = symbol {
            assert_eq!(1234f32, f);
        } else {
            panic!("Expected number symbol")
        }
    }

    #[test]
    fn is_number_should_capture_decimal() {
        let chars = "1234.5678".chars().collect();
        let symbol = is_number(&mut VecPointer::new(chars)).unwrap();

        if let Symbol::Number(f) = symbol {
            assert_eq!(1234.5678f32, f);
        } else {
            panic!("Expected number symbol")
        }
    }

    #[test]
    fn is_number_should_be_none_with_leading_text() {
        let chars = "abc1234".chars().collect();
        let symbol = is_number(&mut VecPointer::new(chars));

        assert!(symbol.is_none());
    }

    #[test]
    fn is_text_should_capture_quoted_text() {
        let chars = r###""world""###.chars().collect();
        let pointer = &mut VecPointer::new(chars);
        let symbol = is_text(pointer);

        if let Some(Symbol::Text(text)) = symbol {
            assert_eq!("world", text);
            matches!(pointer.next(), None);
        } else {
            panic!("Expected text symbol")
        }
    }

    #[test]
    fn is_text_should_capture_single_quoted_text() {
        let chars = r###"'world'"###.chars().collect();
        let pointer = &mut VecPointer::new(chars);
        let symbol = is_text(pointer);

        if let Some(Symbol::Text(text)) = symbol {
            assert_eq!("world", text);
            matches!(pointer.next(), None);
        } else {
            panic!("Expected text symbol")
        }
    }

    #[test]
    fn is_text_should_not_capture_mismatched_quoted_text() {
        let chars = r###""world'"###.chars().collect();
        let pointer = &mut VecPointer::new(chars);
        let symbol = is_text(pointer);

        matches!(symbol, None);
        matches!(pointer.current(), Some('"')); // Assert cursor was not moved.
    }
}