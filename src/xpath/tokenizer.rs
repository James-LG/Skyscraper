use crate::textpointer::TextPointer;

/// Enum representing all possible symbols output by the lexer.
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Symbol {
    /// `/`
    Slash,

    /// `//`
    DoubleSlash,

    /// `[`
    OpenSquareBracket,

    /// `]`
    CloseSquareBracket,

    /// `(`
    OpenBracket,

    /// `)`
    CloseBracket,

    /// `*`
    Wildcard,

    /// `.`
    Dot,

    /// `..`
    DoubleDot,

    /// `=`
    AssignmentSign,

    /// `@`
    AtSign,

    /// `-`
    MinusSign,
    
    /// `+`
    AddSign,

    /// `>`
    GreaterThanSign,

    /// `<`
    LessThanSign,

    /// Unquoted identifier. Example: div
    Identifier(String),

    /// Quoted string. Example: "red"
    Text(String),

    /// Literal number. Example: 1
    Number(f32),
}

pub fn lex(text: &str) -> Result<Vec<Symbol>, &'static str> {
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut pointer = TextPointer::new(text);

    while let Some(c) = pointer.current() {
        if let Some(s) = is_double_slash(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_slash(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_open_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_close_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_open_square_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_close_square_bracket(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_number(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_wildcard(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_double_dot(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_dot(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_assignment_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_at_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_add_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_minus_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_greater_than_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_less_than_sign(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_identifier(&mut pointer) {
            symbols.push(s);
        } else if let Some(s) = is_text(&mut pointer) {
            symbols.push(s);
        } else {
            if !c.is_whitespace(){
                // Unknown symbol, move on ¯\_(ツ)_/¯
                eprintln!("Unknown symbol {}", c);
            }
            pointer.next_char();
        }
    }
    Ok(symbols)
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleSlash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_double_slash(pointer: &mut TextPointer) -> Option<Symbol> {
    if let (Some('/'), Some('/')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_char_add(2);
        return Some(Symbol::DoubleSlash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Slash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_slash(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('/') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::Slash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_open_square_bracket(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('[') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::OpenSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_close_square_bracket(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some(']') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::CloseSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_open_bracket(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('(') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::OpenBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_close_bracket(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some(')') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::CloseBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Wildcard [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_wildcard(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('*') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::Wildcard);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_double_dot(pointer: &mut TextPointer) -> Option<Symbol> {
    if let (Some('.'), Some('.')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_char_add(2);
        return Some(Symbol::DoubleDot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_dot(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('.') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::Dot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AssignmentSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_assignment_sign(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('=') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::AssignmentSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AtSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_at_sign(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('@') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::AtSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AddSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_add_sign(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('+') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::AddSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a MinusSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_minus_sign(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('-') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::MinusSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a GreaterThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_greater_than_sign(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('>') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::GreaterThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a LessThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_less_than_sign(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some('<') = pointer.current() {
        pointer.next_char();
        return Some(Symbol::LessThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Number [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_number(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c.is_digit(10) {
            let mut num = c.to_string();
            while let Some(c) = pointer.next_char() {
                if c.is_digit(10) {
                    num.push(c);
                } else {
                    break;
                }
            }

            // Check for decimal values
            if let Some('.') = pointer.current() {
                num.push('.');
                while let Some(c) = pointer.next_char() {
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
fn is_identifier(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c.is_alphabetic() {
            let mut id = c.to_string();

            while let Some(c) = pointer.next_char() {
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
fn is_text(pointer: &mut TextPointer) -> Option<Symbol> {
    if let Some(c) = pointer.current() {
        if c == '"' {
            let mut text = c.to_string();

            while let Some(c) = pointer.next_char() {
                if c == '"' {
                    text.push(c);
                } else {
                    break;
                }
            }

            return Some(Symbol::Text(text));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_number_should_be_some_with_trailing_text() {
        let text = "1234abc";

        let symbol = is_number(&mut TextPointer::new(text)).unwrap();

        if let Symbol::Number(f) = symbol {
            assert_eq!(1234f32, f);
        } else {
            panic!("Expected number symbol")
        }
    }

    #[test]
    fn is_number_should_capture_decimal() {
        let text = "1234.5678";

        let symbol = is_number(&mut TextPointer::new(text)).unwrap();

        if let Symbol::Number(f) = symbol {
            assert_eq!(1234.5678f32, f);
        } else {
            panic!("Expected number symbol")
        }
    }

    #[test]
    fn is_number_should_be_none_with_leading_text() {
        let text = "abc1234";

        let symbol = is_number(&mut TextPointer::new(text));

        assert!(symbol.is_none());
    }

    #[test]
    fn lex_works() {
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
}