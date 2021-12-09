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
