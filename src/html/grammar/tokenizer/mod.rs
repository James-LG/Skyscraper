//! <https://html.spec.whatwg.org/multipage/parsing.html#tokenization>

use std::collections::HashMap;

use crate::vecpointer::VecPointerRef;

use super::{HtmlParseError, HtmlParseErrorType, ParseErrorHandler};

mod state_impls;

pub enum HtmlToken {
    DocType,
    StartTag(TagToken),
    EndTag(TagToken),
    Comment,
    Character(char),
    EndOfFile,
}

pub struct TagToken {
    pub tag_name: String,
    pub self_closing: bool,
    pub attributes: HashMap<String, String>,
}

impl TagToken {
    pub fn new(tag_name: String) -> Self {
        TagToken {
            tag_name,
            self_closing: false,
            attributes: HashMap::new(),
        }
    }
}

enum TokenizerState {
    Data,
    RCDATA,
    RAWTEXT,
    ScriptData,
    PLAINTEXT,
    TagOpen,
    EndTagOpen,
    TagName,
    RCDATALessThanSign,
    RCDATAEndTagOpen,
    RCDATAEndTagName,
    RAWTEXTLessThanSign,
    RAWTEXTEndTagOpen,
    RAWTEXTEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscapeStartDash,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    DOCTYPE,
    BeforeDOCTYPEName,
    DOCTYPEName,
    AfterDOCTYPEName,
    AfterDOCTYPEPublicKeyword,
    BeforeDOCTYPEPublicIdentifier,
    DOCTYPEPublicIdentifierDoubleQuoted,
    DOCTYPEPublicIdentifierSingleQuoted,
    AfterDOCTYPEPublicIdentifier,
    BetweenDOCTYPEPublicAndSystemIdentifiers,
    AfterDOCTYPESystemKeyword,
    BeforeDOCTYPESystemIdentifier,
    DOCTYPESystemIdentifierDoubleQuoted,
    DOCTYPESystemIdentifierSingleQuoted,
    AfterDOCTYPESystemIdentifier,
    BogusDOCTYPE,
    CDATASection,
    CDATASectionBracket,
    CDATASectionEnd,
    CharacterReference,
    NamedCharacterReference,
    AmbiguousAmpersand,
    NumericCharacterReference,
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReferenceStart,
    HexadecimalCharacterReference,
    DecimalCharacterReference,
    NumericCharacterReferenceEnd,
}

pub(crate) enum TokenizerError {
    UnexpectedNullCharacter,
}

pub(crate) trait TokenizerErrorHandler {
    fn error_emitted(
        &self,
        error: TokenizerError,
        tokenizer: &mut Tokenizer,
    ) -> Result<(), HtmlParseError>;
}

pub(crate) struct DefaultTokenizerErrorHandler;

impl TokenizerErrorHandler for DefaultTokenizerErrorHandler {
    fn error_emitted(
        &self,
        error: TokenizerError,
        tokenizer: &mut Tokenizer,
    ) -> Result<(), HtmlParseError> {
        match error {
            TokenizerError::UnexpectedNullCharacter => {
                // In general, NULL code points are ignored.
                Ok(())
            }
        }
    }
}

pub(crate) trait TokenizerObserver {
    fn token_emitted(&self, tokens: &[HtmlToken]);
}

pub struct Tokenizer<'a> {
    state: TokenizerState,
    return_state: Option<TokenizerState>,
    temporary_buffer: Vec<char>,
    input_stream: VecPointerRef<'a, char>,
    observer: Option<Box<&'a dyn TokenizerObserver>>,
    error_handler: Option<Box<&'a dyn TokenizerErrorHandler>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input_stream: VecPointerRef<'a, char>) -> Self {
        Tokenizer {
            state: TokenizerState::Data,
            return_state: None,
            temporary_buffer: Vec::new(),
            input_stream,
            observer: None,
            error_handler: None,
        }
    }

    pub fn set_observer(&mut self, observer: Box<&'a dyn TokenizerObserver>) {
        self.observer = Some(observer);
    }

    pub fn set_error_handler(&mut self, error_handler: Box<&'a dyn TokenizerErrorHandler>) {
        self.error_handler = Some(error_handler);
    }

    pub fn emit(&self, tokens: Vec<HtmlToken>) {
        if let Some(observer) = &self.observer {
            observer.token_emitted(&tokens);
        }
    }

    pub fn handle_error(&mut self, error: TokenizerError) -> Result<(), HtmlParseError> {
        if let Some(error_handler) = &self.error_handler {
            error_handler.error_emitted(error, self)?;
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), HtmlParseError> {
        match self.state {
            TokenizerState::Data => self.data_state(),
            TokenizerState::RCDATA => todo!(),
            TokenizerState::RAWTEXT => todo!(),
            TokenizerState::ScriptData => todo!(),
            TokenizerState::PLAINTEXT => todo!(),
            TokenizerState::TagOpen => todo!(),
            TokenizerState::EndTagOpen => todo!(),
            TokenizerState::TagName => todo!(),
            TokenizerState::RCDATALessThanSign => todo!(),
            TokenizerState::RCDATAEndTagOpen => todo!(),
            TokenizerState::RCDATAEndTagName => todo!(),
            TokenizerState::RAWTEXTLessThanSign => todo!(),
            TokenizerState::RAWTEXTEndTagOpen => todo!(),
            TokenizerState::RAWTEXTEndTagName => todo!(),
            TokenizerState::ScriptDataLessThanSign => todo!(),
            TokenizerState::ScriptDataEndTagOpen => todo!(),
            TokenizerState::ScriptDataEndTagName => todo!(),
            TokenizerState::ScriptDataEscapeStart => todo!(),
            TokenizerState::ScriptDataEscapeStartDash => todo!(),
            TokenizerState::ScriptDataEscaped => todo!(),
            TokenizerState::ScriptDataEscapedDash => todo!(),
            TokenizerState::ScriptDataEscapedDashDash => todo!(),
            TokenizerState::ScriptDataEscapedLessThanSign => todo!(),
            TokenizerState::ScriptDataEscapedEndTagOpen => todo!(),
            TokenizerState::ScriptDataEscapedEndTagName => todo!(),
            TokenizerState::ScriptDataDoubleEscapeStart => todo!(),
            TokenizerState::ScriptDataDoubleEscaped => todo!(),
            TokenizerState::ScriptDataDoubleEscapedDash => todo!(),
            TokenizerState::ScriptDataDoubleEscapedDashDash => todo!(),
            TokenizerState::ScriptDataDoubleEscapedLessThanSign => todo!(),
            TokenizerState::ScriptDataDoubleEscapeEnd => todo!(),
            TokenizerState::BeforeAttributeName => todo!(),
            TokenizerState::AttributeName => todo!(),
            TokenizerState::AfterAttributeName => todo!(),
            TokenizerState::BeforeAttributeValue => todo!(),
            TokenizerState::AttributeValueDoubleQuoted => todo!(),
            TokenizerState::AttributeValueSingleQuoted => todo!(),
            TokenizerState::AttributeValueUnquoted => todo!(),
            TokenizerState::AfterAttributeValueQuoted => todo!(),
            TokenizerState::SelfClosingStartTag => todo!(),
            TokenizerState::BogusComment => todo!(),
            TokenizerState::MarkupDeclarationOpen => todo!(),
            TokenizerState::CommentStart => todo!(),
            TokenizerState::CommentStartDash => todo!(),
            TokenizerState::Comment => todo!(),
            TokenizerState::CommentLessThanSign => todo!(),
            TokenizerState::CommentLessThanSignBang => todo!(),
            TokenizerState::CommentLessThanSignBangDash => todo!(),
            TokenizerState::CommentLessThanSignBangDashDash => todo!(),
            TokenizerState::CommentEndDash => todo!(),
            TokenizerState::CommentEnd => todo!(),
            TokenizerState::CommentEndBang => todo!(),
            TokenizerState::DOCTYPE => todo!(),
            TokenizerState::BeforeDOCTYPEName => todo!(),
            TokenizerState::DOCTYPEName => todo!(),
            TokenizerState::AfterDOCTYPEName => todo!(),
            TokenizerState::AfterDOCTYPEPublicKeyword => todo!(),
            TokenizerState::BeforeDOCTYPEPublicIdentifier => todo!(),
            TokenizerState::DOCTYPEPublicIdentifierDoubleQuoted => todo!(),
            TokenizerState::DOCTYPEPublicIdentifierSingleQuoted => todo!(),
            TokenizerState::AfterDOCTYPEPublicIdentifier => todo!(),
            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiers => todo!(),
            TokenizerState::AfterDOCTYPESystemKeyword => todo!(),
            TokenizerState::BeforeDOCTYPESystemIdentifier => todo!(),
            TokenizerState::DOCTYPESystemIdentifierDoubleQuoted => todo!(),
            TokenizerState::DOCTYPESystemIdentifierSingleQuoted => todo!(),
            TokenizerState::AfterDOCTYPESystemIdentifier => todo!(),
            TokenizerState::BogusDOCTYPE => todo!(),
            TokenizerState::CDATASection => todo!(),
            TokenizerState::CDATASectionBracket => todo!(),
            TokenizerState::CDATASectionEnd => todo!(),
            TokenizerState::CharacterReference => todo!(),
            TokenizerState::NamedCharacterReference => todo!(),
            TokenizerState::AmbiguousAmpersand => todo!(),
            TokenizerState::NumericCharacterReference => todo!(),
            TokenizerState::HexadecimalCharacterReferenceStart => todo!(),
            TokenizerState::DecimalCharacterReferenceStart => todo!(),
            TokenizerState::HexadecimalCharacterReference => todo!(),
            TokenizerState::DecimalCharacterReference => todo!(),
            TokenizerState::NumericCharacterReferenceEnd => todo!(),
        }
    }
}
