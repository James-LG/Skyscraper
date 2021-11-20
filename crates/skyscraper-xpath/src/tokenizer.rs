use skyscraper::vecpointer::VecPointer;

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

    let chars = text.chars().collect();
    let mut pointer = VecPointer::new(chars);

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
            pointer.next();
        }
    }
    Ok(symbols)
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleSlash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_double_slash(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let (Some('/'), Some('/')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_add(2);
        return Some(Symbol::DoubleSlash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Slash [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_slash(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('/') = pointer.current() {
        pointer.next();
        return Some(Symbol::Slash);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_open_square_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('[') = pointer.current() {
        pointer.next();
        return Some(Symbol::OpenSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseSquareBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_close_square_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some(']') = pointer.current() {
        pointer.next();
        return Some(Symbol::CloseSquareBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a OpenBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_open_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('(') = pointer.current() {
        pointer.next();
        return Some(Symbol::OpenBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a CloseBracket [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_close_bracket(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some(')') = pointer.current() {
        pointer.next();
        return Some(Symbol::CloseBracket);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Wildcard [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_wildcard(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('*') = pointer.current() {
        pointer.next();
        return Some(Symbol::Wildcard);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_double_dot(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let (Some('.'), Some('.')) = (pointer.current(), pointer.peek()) {
        // Peeked before, move up now.
        pointer.next_add(2);
        return Some(Symbol::DoubleDot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a DoubleDot [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_dot(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('.') = pointer.current() {
        pointer.next();
        return Some(Symbol::Dot);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AssignmentSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_assignment_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('=') = pointer.current() {
        pointer.next();
        return Some(Symbol::AssignmentSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AtSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_at_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('@') = pointer.current() {
        pointer.next();
        return Some(Symbol::AtSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a AddSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_add_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('+') = pointer.current() {
        pointer.next();
        return Some(Symbol::AddSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a MinusSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_minus_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('-') = pointer.current() {
        pointer.next();
        return Some(Symbol::MinusSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a GreaterThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_greater_than_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('>') = pointer.current() {
        pointer.next();
        return Some(Symbol::GreaterThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a LessThanSign [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_less_than_sign(pointer: &mut VecPointer<char>) -> Option<Symbol> {
    if let Some('<') = pointer.current() {
        pointer.next();
        return Some(Symbol::LessThanSign);
    }
    None
}

/// Checks if the [TextPointer](TextPointer) is currently pointing to a Number [Symbol](Symbol).
/// If true it will move the text pointer to the next symbol, otherwise it will not change the pointer.
fn is_number(pointer: &mut VecPointer<char>) -> Option<Symbol> {
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
fn is_identifier(pointer: &mut VecPointer<char>) -> Option<Symbol> {
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
fn is_text(pointer: &mut VecPointer<char>) -> Option<Symbol> {
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

    #[test]
    fn lex_works1() {
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

    #[test]
    fn lex_works2() {
        // arrange
        let text = "/bookstore/book[price>35]/price";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Symbol::Slash,
            Symbol::Identifier(String::from("bookstore")),
            Symbol::Slash,
            Symbol::Identifier(String::from("book")),
            Symbol::OpenSquareBracket,
            Symbol::Identifier(String::from("price")),
            Symbol::GreaterThanSign,
            Symbol::Number(35.0),
            Symbol::CloseSquareBracket,
            Symbol::Slash,
            Symbol::Identifier(String::from("price"))
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn lex_works3() {
        // arrange
        let text = r###"//a[@hello="world"]"###;

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Symbol::DoubleSlash,
            Symbol::Identifier(String::from("a")),
            Symbol::OpenSquareBracket,
            Symbol::AtSign,
            Symbol::Identifier(String::from("hello")),
            Symbol::AssignmentSign,
            Symbol::Text(String::from("world")),
            Symbol::CloseSquareBracket,
        ];

        assert_eq!(expected, result);
    }
}