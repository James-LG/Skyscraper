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

    /// Unquoted identifier. Example: div
    Identifier(String),

    /// Quoted string. Example: "red"
    Text(String),

    /// Literal number. Example: 1
    Number(f32),

    /// `@`
    AtSign,

    /// `-`
    SubtractSign,
    
    /// `+`
    AddSign,

    /// `>`
    GreaterThanSign,

    /// `<`
    LessThanSign,
}

pub fn lex(text: &str) -> Result<Vec<Symbol>, &'static str> {
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut pointer = TextPointer::new(text);

    let mut has_open_tag = false;

    Err("boom")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_works() {
        // arrange
        let text = "/bookstore/book[1]/page[last()-1]";

        // act
        let result = lex(text).unwrap();

        // assert
        let expected = vec![
            Symbol::Slash,
            Symbol::Identifier(String::from("bookstore")),
            Symbol::Slash,
            Symbol::Identifier(String::from("book")),
            Symbol::OpenSquareBracket,
            Symbol::Number(1.0),
            Symbol::CloseSquareBracket,
            Symbol::Slash,
            Symbol::Identifier(String::from("last")),
            Symbol::OpenBracket,
            Symbol::CloseBracket,
            Symbol::SubtractSign,
            Symbol::Number(1.0),
            Symbol::CloseSquareBracket,
        ];

        assert_eq!(expected, result);
    }
}