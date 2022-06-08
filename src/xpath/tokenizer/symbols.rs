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

    /// `::`
    DoubleColon,

    /// Unquoted identifier. Example: div
    Identifier(String),

    /// Quoted string. Example: "red"
    Text(String),

    /// Literal number. Example: 1
    Number(f32),
}