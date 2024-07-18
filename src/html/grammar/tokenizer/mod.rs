//! <https://html.spec.whatwg.org/multipage/parsing.html#tokenization>

use std::collections::HashMap;

use thiserror::Error;

use crate::vecpointer::VecPointerRef;

use super::{HtmlParseError, HtmlParseErrorType, ParseErrorHandler};

mod state_impls;

#[derive(Debug)]
pub enum HtmlToken {
    DocType,
    StartTag(TagToken),
    EndTag(TagToken),
    Comment(CommentToken),
    Character(char),
    EndOfFile,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct CommentToken {
    pub data: String,
}

impl CommentToken {
    pub fn new(data: String) -> Self {
        CommentToken { data }
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

#[derive(Debug, Error)]
pub(crate) enum TokenizerError {
    #[error("unexpected null character")]
    UnexpectedNullCharacter,
    #[error("unexpected question mark instead of tag name")]
    UnexpectedQuestionMarkInsteadOfTagName,
    #[error("invalid first character of tag name")]
    InvalidFirstCharacterOfTagName,
    #[error("eof before tag name")]
    EofBeforeTagName,
    #[error("eof in tag")]
    EofInTag,
    #[error("missing end tag name")]
    MissingEndTagName,
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
            _ => Err(HtmlParseError {
                message: format!("{:?}", error),
            }),
        }
    }
}

pub(crate) trait TokenizerObserver {
    fn token_emitted(&mut self, token: HtmlToken) -> Result<(), HtmlParseError>;
}

pub struct Tokenizer<'a> {
    state: TokenizerState,
    return_state: Option<TokenizerState>,
    temporary_buffer: Vec<char>,
    input_stream: VecPointerRef<'a, char>,
    observer: Option<Box<&'a mut dyn TokenizerObserver>>,
    error_handler: Option<Box<&'a dyn TokenizerErrorHandler>>,
    comment_token: Option<CommentToken>,
    tag_token: Option<TagToken>,
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
            comment_token: None,
            tag_token: None,
        }
    }

    pub fn set_observer(&mut self, observer: Box<&'a mut dyn TokenizerObserver>) {
        self.observer = Some(observer);
    }

    pub fn set_error_handler(&mut self, error_handler: Box<&'a dyn TokenizerErrorHandler>) {
        self.error_handler = Some(error_handler);
    }

    pub fn emit(&mut self, token: HtmlToken) -> Result<(), HtmlParseError> {
        println!("emitting token: {:?}", token);
        if let Some(observer) = &mut self.observer {
            observer.token_emitted(token)?;
        }

        Ok(())
    }

    pub fn handle_error(&mut self, error: TokenizerError) -> Result<(), HtmlParseError> {
        if let Some(error_handler) = &self.error_handler {
            error_handler.error_emitted(error, self)?;
        }

        Ok(())
    }

    pub fn reconsume(&mut self) {
        self.input_stream.prev();
    }

    pub fn reconsume_in_state(&mut self, state: TokenizerState) -> Result<(), HtmlParseError> {
        self.reconsume();
        self.state = state;
        self.step()
    }

    pub fn emit_current_tag_token(&mut self) {
        if let Some(tag_token) = self.tag_token.take() {
            self.emit(HtmlToken::StartTag(tag_token));
            self.tag_token = None;
        }
    }

    pub fn current_tag_token_mut(&mut self) -> Result<&mut TagToken, HtmlParseError> {
        self.tag_token
            .as_mut()
            .ok_or(HtmlParseError::new("no current tag found"))
    }

    pub fn step(&mut self) -> Result<(), HtmlParseError> {
        match self.state {
            TokenizerState::Data => self.data_state(),
            TokenizerState::RCDATA => self.rcdata_state(),
            TokenizerState::RAWTEXT => self.rawtext_state(),
            TokenizerState::ScriptData => todo!(),
            TokenizerState::PLAINTEXT => todo!(),
            TokenizerState::TagOpen => self.tag_open_state(),
            TokenizerState::EndTagOpen => self.end_tag_open_state(),
            TokenizerState::TagName => self.tag_name_state(),
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

    pub fn is_terminated(&self) -> bool {
        !self.input_stream.has_next()
    }
}
