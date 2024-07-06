//! <https://html.spec.whatwg.org/multipage/parsing.html>

use nom::error;
use thiserror::Error;
use tokenizer::{HtmlToken, TokenizerObserver};

use crate::{vecpointer::VecPointerRef, xpath::grammar::XpathItemTreeNode};

mod tokenizer;

enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(Debug)]
pub(crate) enum HtmlParseErrorType {
    AbruptClosingOfEmptyComment,
    AbruptDoctypePublicIdentifier,
    AbruptDoctypeSystemIdentifier,
    AbsenceOfDigitsInNumericCharacterReference,
    CdataInHtmlContent,
    CharacterReferenceOutsideUnicodeRange,
    ControlCharacterInInputStream,
    ControlCharacterReference,
    DuplicateAttribute,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    EofBeforeTagName,
    EofInCdata,
    EofInComment,
    EofInDoctype,
    EofInScriptHtmlCommentLikeText,
    EofInTag,
    IncorrectlyClosedComment,
    IncorrectlyOpenedComment,
    InvalidCharacterSequenceAfterDoctypeName,
    InvalidFirstCharacterOfTagName,
    MissingAttributeValue,
    MissingDoctypeName,
    MissingDoctypePublicIdentifier,
    MissingDoctypeSystemIdentifier,
    MissingEndTagName,
    MissingQuoteBeforeDoctypePublicIdentifier,
    MissingQuoteBeforeDoctypeSystemIdentifier,
    MissingSemicolonAfterCharacterReference,
    MissingWhitespaceAfterDoctypePublicKeyword,
    MissingWhitespaceAfterDoctypeSystemKeyword,
    MissingWhitespaceBeforeDoctypeName,
    MissingWhitespaceBetweenAttributes,
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
    NestedComment,
    NoncharacterCharacterReference,
    NoncharacterInInputStream,
    NonVoidHtmlElementStartTagWithTrailingSolidus,
    NullCharacterReference,
    SurrogateCharacterReference,
    SurrogateInInputStream,
    UnexpectedCharacterAfterDoctypeSystemIdentifier,
    UnexpectedCharacterInAttributeName,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedNullCharacter,
    UnexpectedQuestionMarkInsteadOfTagName,
    UnexpectedSolidusInTag,
    UnknownNamedCharacterReference,
}

#[derive(Debug, Error)]
#[error("parse error: {message}")]
pub struct HtmlParseError {
    pub message: String,
}

pub fn parse(text: &str) -> Result<(), HtmlParseError> {
    let mut open_elements: Vec<XpathItemTreeNode> = Vec::new();

    let chars: Vec<char> = text.chars().collect();
    let input_stream = VecPointerRef::new(&chars);
    let mut tokenizer = tokenizer::Tokenizer::new(input_stream);
    let mut tokenizer_error_handler = tokenizer::DefaultTokenizerErrorHandler;

    let mut parser = HtmlParser::new();
    let mut error_handler = DefaultParseErrorHandler;

    tokenizer.set_observer(Box::new(&parser));
    tokenizer.set_error_handler(Box::new(&tokenizer_error_handler));

    Ok(())
}

pub struct HtmlParser {
    insertion_mode: InsertionMode,
    open_elements: Vec<XpathItemTreeNode>,
}

impl HtmlParser {
    pub fn new() -> Self {
        HtmlParser {
            insertion_mode: InsertionMode::Initial,
            open_elements: Vec::new(),
        }
    }
}

impl TokenizerObserver for HtmlParser {
    fn token_emitted(&self, tokens: &[HtmlToken]) {
        for token in tokens {
            match token {
                _ => todo!(),
            }
        }
    }
}

pub trait ParseErrorHandler {
    fn error_emitted(&self, error: HtmlParseErrorType) -> Result<(), HtmlParseError>;
}

pub struct DefaultParseErrorHandler;

impl ParseErrorHandler for DefaultParseErrorHandler {
    fn error_emitted(&self, error: HtmlParseErrorType) -> Result<(), HtmlParseError> {
        Err(HtmlParseError {
            message: format!("{:?}", error),
        })
    }
}
