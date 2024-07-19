use crate::html::grammar::{chars, HtmlParseError};

use super::{
    CommentToken, HtmlToken, TagToken, TagTokenType, Tokenizer, TokenizerError, TokenizerState,
};

impl<'a> Tokenizer<'a> {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#data-state>
    pub(super) fn data_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) => match c {
                '&' => {
                    self.return_state = Some(TokenizerState::Data);
                    self.state = TokenizerState::CharacterReference;
                }
                '<' => {
                    self.state = TokenizerState::TagOpen;
                }
                &chars::NULL => {
                    let current_input_character = *c;
                    self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                    self.emit(HtmlToken::Character(current_input_character))?;
                }
                _ => {
                    let current_input_character = *c;
                    self.emit(HtmlToken::Character(current_input_character))?;
                }
            },
            None => self.emit(HtmlToken::EndOfFile)?,
        };

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state>
    pub(super) fn rcdata_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) => match c {
                '&' => {
                    self.return_state = Some(TokenizerState::RCDATA);
                    self.state = TokenizerState::CharacterReference;
                }
                '<' => {
                    self.state = TokenizerState::RCDATALessThanSign;
                }
                &chars::NULL => {
                    self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                    self.emit(HtmlToken::Character(chars::FEED_REPLACEMENT_CHARACTER))?;
                }
                _ => {
                    let current_input_character = *c;
                    self.emit(HtmlToken::Character(current_input_character))?;
                }
            },
            None => self.emit(HtmlToken::EndOfFile)?,
        };

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#rawtext-state>
    pub(super) fn rawtext_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) => match c {
                '<' => {
                    self.state = TokenizerState::RAWTEXTLessThanSign;
                }
                &chars::NULL => {
                    self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                    self.emit(HtmlToken::Character(chars::FEED_REPLACEMENT_CHARACTER))?;
                }
                _ => {
                    let current_input_character = *c;
                    self.emit(HtmlToken::Character(current_input_character))?;
                }
            },
            None => self.emit(HtmlToken::EndOfFile)?,
        };

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state>
    pub(super) fn tag_open_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) => match c {
                '!' => {
                    self.state = TokenizerState::MarkupDeclarationOpen;
                }
                '/' => {
                    self.state = TokenizerState::EndTagOpen;
                }
                '?' => {
                    self.handle_error(TokenizerError::UnexpectedQuestionMarkInsteadOfTagName)?;

                    self.comment_token = Some(CommentToken {
                        data: String::new(),
                    });
                }
                _ if c.is_ascii_alphabetic() => {
                    self.tag_token = Some(TagTokenType::StartTag(TagToken::new(String::new())));
                    self.reconsume_in_state(TokenizerState::TagName)?;
                }
                _ => {
                    self.handle_error(TokenizerError::InvalidFirstCharacterOfTagName)?;

                    self.emit(HtmlToken::Character('<'))?;
                    self.emit(HtmlToken::EndOfFile)?;
                }
            },
            None => {
                self.handle_error(TokenizerError::EofBeforeTagName)?;

                self.emit(HtmlToken::Character('<'))?;
                self.reconsume_in_state(TokenizerState::Data)?
            }
        };

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state>
    pub(super) fn end_tag_open_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) => match c {
                _ if c.is_ascii_alphabetic() => {
                    self.tag_token = Some(TagTokenType::EndTag(TagToken::new(String::new())));
                    self.reconsume_in_state(TokenizerState::TagName)?;
                }
                '>' => {
                    self.handle_error(TokenizerError::MissingEndTagName)?;

                    self.state = TokenizerState::Data;
                }
                _ => {
                    self.handle_error(TokenizerError::InvalidFirstCharacterOfTagName)?;

                    self.comment_token = Some(CommentToken::new(String::new()));
                    self.reconsume_in_state(TokenizerState::BogusComment)?;
                }
            },
            None => {
                self.handle_error(TokenizerError::EofBeforeTagName)?;

                self.emit(HtmlToken::Character('<'))?;
                self.emit(HtmlToken::Character('/'))?;
                self.reconsume_in_state(TokenizerState::Data)?;
            }
        };

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state>
    pub(super) fn tag_name_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) => match c {
                &chars::CHARACTER_TABULATION
                | &chars::LINE_FEED
                | &chars::FORM_FEED
                | &chars::SPACE => {
                    self.state = TokenizerState::BeforeAttributeName;
                }
                &'/' => {
                    self.state = TokenizerState::SelfClosingStartTag;
                }
                &'>' => {
                    self.state = TokenizerState::Data;
                    self.emit_current_tag_token();
                }
                &chars::NULL => {
                    self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                    self.current_tag_token_mut()?
                        .tag_name_mut()
                        .push(chars::FEED_REPLACEMENT_CHARACTER);
                }
                _ if c.is_ascii_uppercase() => {
                    let c = c.to_ascii_lowercase();
                    self.current_tag_token_mut()?.tag_name_mut().push(c);
                }
                _ => {
                    let c = *c;
                    self.current_tag_token_mut()?.tag_name_mut().push(c);
                }
            },
            None => {
                self.handle_error(TokenizerError::EofInTag)?;

                self.emit(HtmlToken::EndOfFile)?;
            }
        };

        Ok(())
    }
}
