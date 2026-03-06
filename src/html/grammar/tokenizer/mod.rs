//! <https://html.spec.whatwg.org/multipage/parsing.html#tokenization>

use std::collections::{hash_map::Entry, HashMap};

use indextree::NodeId;
use thiserror::Error;

use crate::{vecpointer::VecPointerRef, xpath::grammar::XpathItemTreeNode};

use super::{Acknowledgement, HtmlParseError, HtmlParseErrorType, ParseErrorHandler};

mod named_character_references;
mod state_impls;

#[derive(Debug, Clone)]
pub enum HtmlToken {
    DocType(DoctypeToken),
    TagToken(TagTokenType),
    Comment(CommentToken),
    Character(char),
    /// A batch of consecutive characters emitted together to reduce
    /// per-character dispatch overhead in the tree builder.
    Characters(String),
    EndOfFile,
}

#[derive(Debug, Clone)]
pub struct DoctypeToken {
    pub name: String,
    pub force_quirks: bool,
    pub public_identifier: Option<String>,
    pub system_identifier: Option<String>,
}

impl DoctypeToken {
    pub fn new(name: String) -> Self {
        DoctypeToken {
            name,
            force_quirks: false,
            public_identifier: None,
            system_identifier: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TagTokenType {
    StartTag(TagToken),
    EndTag(TagToken),
}

impl TagTokenType {
    pub fn tag_name(&self) -> &str {
        match self {
            TagTokenType::StartTag(tag) => &tag.tag_name,
            TagTokenType::EndTag(tag) => &tag.tag_name,
        }
    }

    pub fn tag_name_mut(&mut self) -> &mut String {
        match self {
            TagTokenType::StartTag(tag) => &mut tag.tag_name,
            TagTokenType::EndTag(tag) => &mut tag.tag_name,
        }
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        match self {
            TagTokenType::StartTag(tag) => &tag.attributes,
            TagTokenType::EndTag(tag) => &tag.attributes,
        }
    }

    pub fn attributes_mut(&mut self) -> &mut Vec<Attribute> {
        match self {
            TagTokenType::StartTag(tag) => &mut tag.attributes,
            TagTokenType::EndTag(tag) => &mut tag.attributes,
        }
    }

    pub fn self_closing(&self) -> bool {
        match self {
            TagTokenType::StartTag(tag) => tag.self_closing,
            TagTokenType::EndTag(tag) => tag.self_closing,
        }
    }

    pub fn self_closing_mut(&mut self) -> &mut bool {
        match self {
            TagTokenType::StartTag(tag) => &mut tag.self_closing,
            TagTokenType::EndTag(tag) => &mut tag.self_closing,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TagToken {
    pub tag_name: String,
    pub self_closing: bool,
    pub attributes: Vec<Attribute>,
}

impl TagToken {
    pub fn new(tag_name: String) -> Self {
        TagToken {
            tag_name,
            self_closing: false,
            attributes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: String,
    /// Whitespace that preceded this attribute in the original source.
    /// Used for round-trip fidelity in Raw display mode.
    pub prefix: String,
    /// Original attribute name as written in source (before lowercasing).
    /// Used for round-trip fidelity in Raw display mode.
    pub original_name: Option<String>,
    /// The namespace URI of this attribute, if any.
    ///
    /// Set by `adjust_foreign_attributes` for attributes like `xlink:href`,
    /// `xml:lang`, and `xmlns` per WHATWG 13.2.6.3.
    pub namespace: Option<String>,
}

impl Attribute {
    pub fn new(name: String, value: String) -> Self {
        Attribute {
            name,
            value,
            prefix: String::new(),
            original_name: None,
            namespace: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommentToken {
    pub data: String,
}

impl CommentToken {
    pub fn new(data: String) -> Self {
        CommentToken { data }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum TokenizerState {
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
    #[error("missing semicolon after character reference")]
    MissingSemicolonAfterCharacterReference,
    #[error("unknown named character reference")]
    UnknownNamedCharacterReference,
    #[error("absence of digits in numeric character reference")]
    AbsenceOfDigitsInNumericCharacterReference,
    #[error("null character reference")]
    NullCharacterReference,
    #[error("character reference outside unicode range")]
    CharacterReferenceOutsideUnicodeRange,
    #[error("surrogate character reference")]
    SurrogateCharacterReference,
    #[error("noncharacter character reference")]
    NoncharacterCharacterReference,
    #[error("control character reference")]
    ControlCharacterReference,
    #[error("unexpected equals sign before attribute name")]
    UnexpectedEqualsSignBeforeAttributeName,
    #[error("unexpected character in attribute name")]
    UnexpectedCharacterInAttributeName,
    #[error("missing attribute value")]
    MissingAttributeValue,
    #[error("unexpected character in unquoted attribute value")]
    UnexpectedCharacterInUnquotedAttributeValue,
    #[error("missing whitespace between attributes")]
    MissingWhitespaceBetweenAttributes,
    #[error("unexpected solidus in tag")]
    UnexpectedSolidusInTag,
    #[error("CDATA in html content")]
    CdataInHtmlContent,
    #[error("incorrectly opened comment")]
    IncorrectlyOpenedComment,
    #[error("eof in doctype")]
    EofInDoctype,
    #[error("missing whitespace before doctype name")]
    MissingWhitespaceBeforeDoctypeName,
    #[error("missing doctype name")]
    MissingDoctypeName,
    #[error("invalid character sequence after doctype name")]
    InvalidCharacterSequenceAfterDoctypeName,
    #[error("abrupt closing of empty comment")]
    AbruptClosingOfEmptyComment,
    #[error("eof in comment")]
    EofInComment,
    #[error("nested comment")]
    NestedComment,
    #[error("incorrectly closed comment")]
    IncorrectlyClosedComment,
    #[error("eof in script html comment like text")]
    EofInScriptHtmlCommentLikeText,
    #[error("missing whitespace after doctype public keyword")]
    MissingWhitespaceAfterDoctypePublicKeyword,
    #[error("missing doctype public identifier")]
    MissingDoctypePublicIdentifier,
    #[error("missing quote before doctype public identifier")]
    MissingQuoteBeforeDoctypePublicIdentifier,
    #[error("missing whitespace between doctype public and system identifiers")]
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
    #[error("missing quote before doctype system identifier")]
    MissingQuoteBeforeDoctypeSystemIdentifier,
    #[error("missing whitespace after doctype system keyword")]
    MissingWhitespaceAfterDoctypeSystemKeyword,
    #[error("missing doctype system identifier")]
    MissingDoctypeSystemIdentifier,
    #[error("unexpected character after doctype system identifier")]
    UnexpectedCharacterAfterDoctypeSystemIdentifier,
    #[error("eof in cdata section")]
    EofInCdataSection,
    #[error("duplicate attribute")]
    DuplicateAttribute,
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
        let _ = error;
        Ok(())
    }
}

pub(crate) trait Parser {
    fn token_emitted(&mut self, token: HtmlToken) -> Result<Acknowledgement, HtmlParseError>;
    fn adjusted_current_node(&self) -> Option<&XpathItemTreeNode>;
}

pub struct Tokenizer<'a> {
    state: TokenizerState,
    return_state: Option<TokenizerState>,
    temporary_buffer: Vec<char>,
    input_stream: VecPointerRef<'a, char>,
    parser: Box<&'a mut dyn Parser>,
    error_handler: Option<Box<&'a dyn TokenizerErrorHandler>>,
    comment_token: Option<CommentToken>,
    doctype_token: Option<DoctypeToken>,
    tag_token: Option<TagTokenType>,
    character_reference_code: u32,
    last_emitted_start_tag_name: String,
    /// Buffer for accumulating whitespace between attributes for round-trip fidelity.
    attribute_prefix_buffer: String,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input_stream: VecPointerRef<'a, char>, parser: Box<&'a mut dyn Parser>) -> Self {
        Tokenizer {
            state: TokenizerState::Data,
            return_state: None,
            temporary_buffer: Vec::new(),
            input_stream,
            parser,
            error_handler: None,
            comment_token: None,
            tag_token: None,
            doctype_token: None,
            character_reference_code: 0,
            last_emitted_start_tag_name: String::new(),
            attribute_prefix_buffer: String::new(),
        }
    }

    pub fn set_error_handler(&mut self, error_handler: Box<&'a dyn TokenizerErrorHandler>) {
        self.error_handler = Some(error_handler);
    }

    pub fn set_state(&mut self, state: TokenizerState) {
        self.state = state;
    }

    pub fn emit(&mut self, token: HtmlToken) -> Result<(), HtmlParseError> {
        if let HtmlToken::TagToken(TagTokenType::StartTag(tag)) = &token {
            self.last_emitted_start_tag_name.clear();
            self.last_emitted_start_tag_name.push_str(&tag.tag_name);
        }

        let ack = self.parser.token_emitted(token)?;

        if let Some(state) = ack.tokenizer_state {
            self.state = state;
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-end-tag-token>
    pub fn is_appropriate_end_tag_token(&self, end_tag: &TagToken) -> bool {
        !self.last_emitted_start_tag_name.is_empty()
            && self.last_emitted_start_tag_name == end_tag.tag_name
    }

    pub fn is_current_end_tag_token_appropriate(&self) -> bool {
        if let Some(tag_token) = &self.tag_token {
            if let TagTokenType::EndTag(end_tag) = tag_token {
                return self.is_appropriate_end_tag_token(end_tag);
            }
        }

        false
    }

    pub fn handle_error(&mut self, error: TokenizerError) -> Result<(), HtmlParseError> {
        if let Some(error_handler) = &self.error_handler {
            error_handler.error_emitted(error, self)?;
        }

        Ok(())
    }

    pub fn current_attribute_mut(&mut self) -> Result<&mut Attribute, HtmlParseError> {
        let current_tag_token = self
            .tag_token
            .as_mut()
            .ok_or_else(|| HtmlParseError::new("no current tag found"))?;

        // Use last_mut() instead of find() because the current attribute being
        // built is always the last one pushed by create_new_attribute(). Using
        // find() by name would incorrectly match an earlier attribute when the
        // current attribute's partial name happens to equal an existing one
        // (e.g. "data-hydro-click" prefix of "data-hydro-click-hmac").
        let entry = current_tag_token
            .attributes_mut()
            .last_mut()
            .ok_or_else(|| {
                HtmlParseError::new("no attributes on current tag")
            })?;

        Ok(entry)
    }

    pub fn create_new_attribute(&mut self, mut attribute: Attribute) -> Result<(), HtmlParseError> {
        attribute.prefix = std::mem::take(&mut self.attribute_prefix_buffer);
        self.current_tag_token_mut()?
            .attributes_mut()
            .push(attribute);

        Ok(())
    }

    pub fn push_char_to_attribute_name(&mut self, c: char) -> Result<(), HtmlParseError> {
        let attr = self.current_attribute_mut()?;
        attr.name.push(c);
        // If we're already tracking original_name (due to earlier uppercase),
        // keep appending to it.
        if let Some(ref mut orig) = attr.original_name {
            orig.push(c);
        }

        Ok(())
    }

    /// Push a lowercased char to the attribute name while preserving
    /// the original character for round-trip fidelity.
    pub fn push_char_to_attribute_name_with_original(
        &mut self,
        c: char,
        original: char,
    ) -> Result<(), HtmlParseError> {
        let attr = self.current_attribute_mut()?;
        attr.name.push(c);
        // Start or continue tracking the original name since casing differs.
        let orig = attr.original_name.get_or_insert_with(|| attr.name[..attr.name.len() - 1].to_string());
        orig.push(original);

        Ok(())
    }

    pub fn push_char_to_attribute_value(&mut self, c: char) -> Result<(), HtmlParseError> {
        self.current_attribute_mut()?.value.push(c);

        Ok(())
    }

    pub fn current_return_state(&self) -> Result<TokenizerState, HtmlParseError> {
        self.return_state
            .ok_or_else(|| HtmlParseError::new("no return state found"))
    }

    pub fn reconsume(&mut self) {
        self.input_stream.prev();
    }

    pub fn reconsume_in_state(&mut self, state: TokenizerState) -> Result<(), HtmlParseError> {
        self.reconsume();
        self.state = state;
        self.step()
    }

    pub fn emit_current_tag_token(&mut self) -> Result<(), HtmlParseError> {
        if let Some(mut tag_token) = self.tag_token.take() {
            #[cfg(feature = "debug_prints")]
            println!("emitting tag token: {:?}", tag_token);

            // WHATWG 13.2.5.34: Before emitting a tag token, check for duplicate
            // attribute names. If a duplicate is found, it is a parse error and the
            // later attribute must be removed (keeping the first occurrence).
            let attributes = tag_token.attributes_mut();
            if attributes.len() > 1 {
                // O(n²) duplicate check without allocation — faster than HashSet
                // for typical attribute counts (< 20) and avoids per-attribute
                // String clones that the HashSet approach required.
                let mut had_duplicate = false;
                let mut i = 1;
                while i < attributes.len() {
                    let is_dup = (0..i).any(|j| attributes[j].name == attributes[i].name);
                    if is_dup {
                        had_duplicate = true;
                        attributes.remove(i);
                    } else {
                        i += 1;
                    }
                }
                if had_duplicate {
                    self.handle_error(TokenizerError::DuplicateAttribute)?;
                }
            }

            self.attribute_prefix_buffer.clear();
            self.emit(HtmlToken::TagToken(tag_token))?;
        }

        Ok(())
    }

    pub fn emit_current_comment_token(&mut self) -> Result<(), HtmlParseError> {
        if let Some(comment_token) = self.comment_token.take() {
            self.emit(HtmlToken::Comment(comment_token))?;
        }

        Ok(())
    }

    pub fn emit_current_doctype_token(&mut self) -> Result<(), HtmlParseError> {
        if let Some(doctype_token) = self.doctype_token.take() {
            self.emit(HtmlToken::DocType(doctype_token))?;
        }

        Ok(())
    }

    pub fn emit_current_token(&mut self) -> Result<(), HtmlParseError> {
        // this is an abritrary order, unclear if it is the correct behaviour
        if self.comment_token.is_some() {
            self.emit_current_comment_token()?;
        } else if self.doctype_token.is_some() {
            self.emit_current_doctype_token()?;
        } else if self.tag_token.is_some() {
            self.emit_current_tag_token()?;
        }

        Ok(())
    }

    pub fn current_tag_token_mut(&mut self) -> Result<&mut TagTokenType, HtmlParseError> {
        self.tag_token
            .as_mut()
            .ok_or_else(|| HtmlParseError::new("no current tag found"))
    }

    pub fn current_doctype_token_mut(&mut self) -> Result<&mut DoctypeToken, HtmlParseError> {
        self.doctype_token
            .as_mut()
            .ok_or_else(|| HtmlParseError::new("no current doctype found"))
    }

    pub fn current_comment_token_mut(&mut self) -> Result<&mut CommentToken, HtmlParseError> {
        self.comment_token
            .as_mut()
            .ok_or_else(|| HtmlParseError::new("no current comment found"))
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#charref-in-attribute>
    pub fn charref_in_attribute(&self) -> bool {
        match self.return_state {
            Some(TokenizerState::AttributeValueDoubleQuoted)
            | Some(TokenizerState::AttributeValueSingleQuoted)
            | Some(TokenizerState::AttributeValueUnquoted) => true,
            _ => false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#flush-code-points-consumed-as-a-character-reference>
    pub fn flush_code_points_consumed_as_character_reference(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        if self.charref_in_attribute() {
            // Split borrow: tag_token and temporary_buffer are disjoint fields.
            // Avoid intermediate Vec<char> allocation on this hot path.
            let attr = self
                .tag_token
                .as_mut()
                .ok_or_else(|| HtmlParseError::new("no current tag found"))?
                .attributes_mut()
                .last_mut()
                .ok_or_else(|| HtmlParseError::new("no attributes on current tag"))?;
            for &c in &self.temporary_buffer {
                attr.value.push(c);
            }
            self.temporary_buffer.clear();
        } else {
            let code_points: Vec<char> = self.temporary_buffer.drain(..).collect();
            if code_points.len() == 1 {
                self.emit(HtmlToken::Character(code_points[0]))?;
            } else {
                self.emit(HtmlToken::Characters(code_points.into_iter().collect()))?;
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), HtmlParseError> {
        match self.state {
            TokenizerState::Data => self.data_state(),
            TokenizerState::RCDATA => self.rcdata_state(),
            TokenizerState::RAWTEXT => self.rawtext_state(),
            TokenizerState::ScriptData => self.script_data_state(),
            TokenizerState::PLAINTEXT => self.plaintext_state(),
            TokenizerState::TagOpen => self.tag_open_state(),
            TokenizerState::EndTagOpen => self.end_tag_open_state(),
            TokenizerState::TagName => self.tag_name_state(),
            TokenizerState::RCDATALessThanSign => self.rcdata_less_than_sign_state(),
            TokenizerState::RCDATAEndTagOpen => self.rcdata_end_tag_open_state(),
            TokenizerState::RCDATAEndTagName => self.rcdata_end_tag_name_state(),
            TokenizerState::RAWTEXTLessThanSign => self.rawtext_less_than_sign_state(),
            TokenizerState::RAWTEXTEndTagOpen => self.rawtext_end_tag_open_state(),
            TokenizerState::RAWTEXTEndTagName => self.rawtext_end_tag_name_state(),
            TokenizerState::ScriptDataLessThanSign => self.script_data_less_than_sign_state(),
            TokenizerState::ScriptDataEndTagOpen => self.script_data_end_tag_open_state(),
            TokenizerState::ScriptDataEndTagName => self.script_data_end_tag_name_state(),
            TokenizerState::ScriptDataEscapeStart => self.script_data_escape_start_state(),
            TokenizerState::ScriptDataEscapeStartDash => self.script_data_escape_start_dash_state(),
            TokenizerState::ScriptDataEscaped => self.script_data_escaped_state(),
            TokenizerState::ScriptDataEscapedDash => self.script_data_escaped_dash_state(),
            TokenizerState::ScriptDataEscapedDashDash => self.script_data_escaped_dash_dash_state(),
            TokenizerState::ScriptDataEscapedLessThanSign => {
                self.script_data_escaped_less_than_sign_state()
            }
            TokenizerState::ScriptDataEscapedEndTagOpen => {
                self.script_data_escaped_end_tag_open_state()
            }
            TokenizerState::ScriptDataEscapedEndTagName => {
                self.script_data_escaped_end_tag_name_state()
            }
            TokenizerState::ScriptDataDoubleEscapeStart => {
                self.script_data_double_escape_start_state()
            }
            TokenizerState::ScriptDataDoubleEscaped => self.script_data_double_escaped_state(),
            TokenizerState::ScriptDataDoubleEscapedDash => {
                self.script_data_double_escaped_dash_state()
            }
            TokenizerState::ScriptDataDoubleEscapedDashDash => {
                self.script_data_double_escaped_dash_dash_state()
            }
            TokenizerState::ScriptDataDoubleEscapedLessThanSign => {
                self.script_data_double_escaped_less_than_sign_state()
            }
            TokenizerState::ScriptDataDoubleEscapeEnd => self.script_data_double_escape_end_state(),
            TokenizerState::BeforeAttributeName => self.before_attribute_name_state(),
            TokenizerState::AttributeName => self.attribute_name_state(),
            TokenizerState::AfterAttributeName => self.after_attribute_name_state(),
            TokenizerState::BeforeAttributeValue => self.before_attribute_value_state(),
            TokenizerState::AttributeValueDoubleQuoted => {
                self.attribute_value_double_quoted_state()
            }
            TokenizerState::AttributeValueSingleQuoted => {
                self.attribute_value_single_quoted_state()
            }
            TokenizerState::AttributeValueUnquoted => self.attribute_value_unquoted_state(),
            TokenizerState::AfterAttributeValueQuoted => self.after_attribute_value_quoted_state(),
            TokenizerState::SelfClosingStartTag => self.self_closing_start_tag_state(),
            TokenizerState::BogusComment => self.bogus_comment_state(),
            TokenizerState::MarkupDeclarationOpen => self.markup_declaration_open_state(),
            TokenizerState::CommentStart => self.comment_start_state(),
            TokenizerState::CommentStartDash => self.comment_start_dash_state(),
            TokenizerState::Comment => self.comment_state(),
            TokenizerState::CommentLessThanSign => self.comment_less_than_sign_state(),
            TokenizerState::CommentLessThanSignBang => self.comment_less_than_sign_bang_state(),
            TokenizerState::CommentLessThanSignBangDash => {
                self.comment_less_than_sign_bang_dash_state()
            }
            TokenizerState::CommentLessThanSignBangDashDash => {
                self.comment_less_than_sign_bang_dash_dash_state()
            }
            TokenizerState::CommentEndDash => self.comment_end_dash_state(),
            TokenizerState::CommentEnd => self.comment_end_state(),
            TokenizerState::CommentEndBang => self.comment_end_bang_state(),
            TokenizerState::DOCTYPE => self.doctype_state(),
            TokenizerState::BeforeDOCTYPEName => self.before_doctype_name(),
            TokenizerState::DOCTYPEName => self.doctype_name_state(),
            TokenizerState::AfterDOCTYPEName => self.after_doctype_name_state(),
            TokenizerState::AfterDOCTYPEPublicKeyword => self.after_doctype_public_keyword_state(),
            TokenizerState::BeforeDOCTYPEPublicIdentifier => {
                self.before_doctype_public_identifier_state()
            }
            TokenizerState::DOCTYPEPublicIdentifierDoubleQuoted => {
                self.doctype_public_identifier_double_quoted_state()
            }
            TokenizerState::DOCTYPEPublicIdentifierSingleQuoted => {
                self.doctype_public_identifier_single_quoted_state()
            }
            TokenizerState::AfterDOCTYPEPublicIdentifier => {
                self.after_doctype_public_identifier_state()
            }
            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiers => {
                self.between_doctype_public_and_system_identifiers_state()
            }
            TokenizerState::AfterDOCTYPESystemKeyword => self.after_doctype_system_keyword_state(),
            TokenizerState::BeforeDOCTYPESystemIdentifier => {
                self.before_doctype_system_identifier_state()
            }
            TokenizerState::DOCTYPESystemIdentifierDoubleQuoted => {
                self.doctype_system_identifier_double_quoted_state()
            }
            TokenizerState::DOCTYPESystemIdentifierSingleQuoted => {
                self.doctype_system_identifier_single_quoted_state()
            }
            TokenizerState::AfterDOCTYPESystemIdentifier => {
                self.after_doctype_system_identifier_state()
            }
            TokenizerState::BogusDOCTYPE => self.bogus_doctype_state(),
            TokenizerState::CDATASection => self.cdata_section_state(),
            TokenizerState::CDATASectionBracket => self.cdata_section_bracket_state(),
            TokenizerState::CDATASectionEnd => self.cdata_section_end_state(),
            TokenizerState::CharacterReference => self.character_reference_state(),
            TokenizerState::NamedCharacterReference => self.named_character_reference_state(),
            TokenizerState::AmbiguousAmpersand => self.ambiguous_ampersand_state(),
            TokenizerState::NumericCharacterReference => self.numeric_character_reference_state(),
            TokenizerState::HexadecimalCharacterReferenceStart => {
                self.hexadecimal_character_reference_start_state()
            }
            TokenizerState::DecimalCharacterReferenceStart => {
                self.decimal_character_reference_start_state()
            }
            TokenizerState::HexadecimalCharacterReference => {
                self.hexadecimal_character_reference_state()
            }
            TokenizerState::DecimalCharacterReference => self.decimal_character_reference_state(),
            TokenizerState::NumericCharacterReferenceEnd => {
                self.numeric_character_reference_end_state()
            }
        }
    }

    pub fn is_terminated(&self) -> bool {
        !self.input_stream.has_next()
    }
}
