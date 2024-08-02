use std::collections::HashMap;

use nom::AsChar;
use once_cell::sync::Lazy;

use crate::{
    html::grammar::{chars, HtmlParseError},
    xpath::grammar::data_model::AttributeNode,
};

use super::{
    named_character_references::{NAMED_CHARACTER_REFS, NAMED_CHARACTER_REFS_MAX_LENGTH},
    Attribute, CommentToken, HtmlToken, TagToken, TagTokenType, Tokenizer, TokenizerError,
    TokenizerState,
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

    /// <https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state>
    pub(super) fn before_attribute_name_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(
                &chars::CHARACTER_TABULATION
                | &chars::LINE_FEED
                | &chars::FORM_FEED
                | &chars::SPACE,
            ) => {
                // ignore
            }
            Some(c) if ['/', '>'].contains(c) => {
                self.reconsume_in_state(TokenizerState::AfterAttributeName)?;
            }
            Some('=') => {
                self.handle_error(TokenizerError::UnexpectedEqualsSignBeforeAttributeName)?;
                let attribute = Attribute::new(String::from('='), String::new());
                self.create_new_attribute(attribute)?;
                self.state = TokenizerState::AttributeName;
            }
            None => {
                // TODO: Does reconsuming an EOF work?
                self.reconsume_in_state(TokenizerState::AfterAttributeName)?;
            }
            Some(c) => {
                let attribute = Attribute::new(String::from(*c), String::new());
                self.create_new_attribute(attribute)?;
                self.reconsume_in_state(TokenizerState::AttributeName)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state>
    pub(super) fn attribute_name_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(
                &chars::CHARACTER_TABULATION
                | &chars::LINE_FEED
                | &chars::FORM_FEED
                | &chars::SPACE
                | '/'
                | '>',
            ) => {
                self.reconsume_in_state(TokenizerState::AfterAttributeName)?;
            }
            None => {
                self.reconsume_in_state(TokenizerState::AfterAttributeName)?;
            }
            Some('=') => {
                self.state = TokenizerState::BeforeAttributeValue;
            }
            Some(c) if c.is_ascii_uppercase() => {
                let c = c.to_ascii_lowercase();
                self.push_char_to_attribute_name(c)?;
            }
            Some(&chars::NULL) => {
                self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                self.push_char_to_attribute_name(chars::FEED_REPLACEMENT_CHARACTER)?;
            }
            Some(c) if ['"', '\'', '<'].contains(c) => {
                let c = *c;
                self.handle_error(TokenizerError::UnexpectedCharacterInAttributeName)?;

                self.push_char_to_attribute_name(c)?;
            }
            Some(c) => {
                let c = *c;
                self.push_char_to_attribute_name(c)?;
            }
        }

        // TODO: check for duplicate attribtue names before emitting

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state>
    pub(super) fn after_attribute_name_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(
                &chars::CHARACTER_TABULATION
                | &chars::LINE_FEED
                | &chars::FORM_FEED
                | &chars::SPACE,
            ) => {
                // ignore
            }
            Some('/') => {
                self.state = TokenizerState::SelfClosingStartTag;
            }
            Some('=') => {
                self.state = TokenizerState::BeforeAttributeValue;
            }
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_tag_token();
            }
            None => {
                self.handle_error(TokenizerError::EofInTag)?;

                self.emit(HtmlToken::EndOfFile)?;
            }
            Some(_) => {
                let attribute = Attribute::new(String::new(), String::new());
                self.create_new_attribute(attribute)?;
                self.reconsume_in_state(TokenizerState::AttributeName)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state>
    pub(super) fn before_attribute_value_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(
                &chars::CHARACTER_TABULATION
                | &chars::LINE_FEED
                | &chars::FORM_FEED
                | &chars::SPACE,
            ) => {
                // ignore
            }
            Some('"') => {
                self.state = TokenizerState::AttributeValueDoubleQuoted;
            }
            Some('\'') => {
                self.state = TokenizerState::AttributeValueSingleQuoted;
            }
            Some('>') => {
                self.handle_error(TokenizerError::MissingAttributeValue)?;

                self.state = TokenizerState::Data;
                self.emit_current_tag_token();
            }
            Some(_) | None => {
                self.reconsume_in_state(TokenizerState::AttributeValueUnquoted)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state>
    pub(super) fn attribute_value_double_quoted_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some('"') => {
                self.state = TokenizerState::AfterAttributeValueQuoted;
            }
            Some('&') => {
                self.return_state = Some(TokenizerState::AttributeValueDoubleQuoted);
                self.state = TokenizerState::CharacterReference;
            }
            Some(&chars::NULL) => {
                self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                self.push_char_to_attribute_value(chars::FEED_REPLACEMENT_CHARACTER)?;
            }
            None => {
                self.handle_error(TokenizerError::EofInTag)?;

                self.emit(HtmlToken::EndOfFile)?;
            }
            Some(c) => {
                let c = *c;
                self.push_char_to_attribute_value(c)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state>
    pub(super) fn attribute_value_single_quoted_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some('\'') => {
                self.state = TokenizerState::AfterAttributeValueQuoted;
            }
            Some('&') => {
                self.return_state = Some(TokenizerState::AttributeValueSingleQuoted);
                self.state = TokenizerState::CharacterReference;
            }
            Some(&chars::NULL) => {
                self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                self.push_char_to_attribute_value(chars::FEED_REPLACEMENT_CHARACTER)?;
            }
            None => {
                self.handle_error(TokenizerError::EofInTag)?;

                self.emit(HtmlToken::EndOfFile)?;
            }
            Some(c) => {
                let c = *c;
                self.push_char_to_attribute_value(c)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state>
    pub(super) fn attribute_value_unquoted_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(
                &chars::CHARACTER_TABULATION
                | &chars::LINE_FEED
                | &chars::FORM_FEED
                | &chars::SPACE,
            ) => {
                self.state = TokenizerState::BeforeAttributeName;
            }
            Some('&') => {
                self.return_state = Some(TokenizerState::AttributeValueUnquoted);
                self.state = TokenizerState::CharacterReference;
            }
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_tag_token();
            }
            Some(&chars::NULL) => {
                self.handle_error(TokenizerError::UnexpectedNullCharacter)?;

                self.push_char_to_attribute_value(chars::FEED_REPLACEMENT_CHARACTER)?;
            }
            Some(c) if ['"', '\'', '<', '=', '`'].contains(c) => {
                let c = *c;
                self.handle_error(TokenizerError::UnexpectedCharacterInUnquotedAttributeValue)?;

                self.push_char_to_attribute_value(c)?;
            }
            None => {
                self.handle_error(TokenizerError::EofInTag)?;

                self.emit(HtmlToken::EndOfFile)?;
            }
            Some(c) => {
                let c = *c;
                self.push_char_to_attribute_value(c)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state>
    pub(super) fn after_attribute_value_quoted_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(
                &chars::CHARACTER_TABULATION
                | &chars::LINE_FEED
                | &chars::FORM_FEED
                | &chars::SPACE,
            ) => {
                self.state = TokenizerState::BeforeAttributeName;
            }
            Some('/') => {
                self.state = TokenizerState::SelfClosingStartTag;
            }
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_tag_token();
            }
            None => {
                self.handle_error(TokenizerError::EofInTag)?;

                self.emit(HtmlToken::EndOfFile)?;
            }
            Some(_) => {
                self.handle_error(TokenizerError::MissingWhitespaceBetweenAttributes)?;

                self.reconsume_in_state(TokenizerState::BeforeAttributeName)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state>
    pub(super) fn self_closing_start_tag_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some('>') => {
                *self.current_tag_token_mut()?.self_closing_mut() = true;
                self.state = TokenizerState::Data;
                self.emit_current_tag_token();
            }
            None => {
                self.handle_error(TokenizerError::EofInTag)?;

                self.emit(HtmlToken::EndOfFile)?;
            }
            Some(_) => {
                self.handle_error(TokenizerError::UnexpectedSolidusInTag)?;

                self.reconsume_in_state(TokenizerState::BeforeAttributeName)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state>
    pub(super) fn character_reference_state(&mut self) -> Result<(), HtmlParseError> {
        fn anything_else(tokenizer: &mut Tokenizer) -> Result<(), HtmlParseError> {
            tokenizer.flush_code_points_consumed_as_character_reference()?;
            tokenizer.reconsume_in_state(tokenizer.current_return_state()?)
        }

        // set the temporary buffer to the empty string
        self.temporary_buffer.clear();

        // append & to the temporary buffer
        self.temporary_buffer.push('&');

        // consume the next input character
        match self.input_stream.next() {
            Some(c) => match c {
                c if c.is_ascii_alphanumeric() => {
                    self.reconsume_in_state(TokenizerState::NamedCharacterReference)?;
                }
                '#' => {
                    self.temporary_buffer.push('#');
                    self.state = TokenizerState::NumericCharacterReference;
                }
                _ => {
                    anything_else(self)?;
                }
            },
            None => {
                anything_else(self)?;
            }
        };

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#named-character-reference-state>
    pub(super) fn named_character_reference_state(&mut self) -> Result<(), HtmlParseError> {
        fn historical_reasons(tokenizer: &mut Tokenizer) -> Result<(), HtmlParseError> {
            tokenizer.flush_code_points_consumed_as_character_reference()?;
            tokenizer.state = tokenizer.current_return_state()?;
            Ok(())
        }

        let mut chars: Vec<char> = Vec::new();

        if let Some(c) = self.input_stream.current() {
            chars.push(*c);
        }

        chars.extend(
            self.input_stream
                .peek_multiple(NAMED_CHARACTER_REFS_MAX_LENGTH)
                .into_iter()
                .map(|c| *c),
        );

        let key = chars.into_iter().collect::<String>();

        let char_ref = NAMED_CHARACTER_REFS
            .keys()
            .filter(|k| key.starts_with(**k))
            .max_by_key(|x| x.len())
            .map(|x| x.to_string());

        match char_ref {
            Some(char_ref) => {
                let length = char_ref.len();

                // consume the characters
                self.input_stream.next_add(length);

                // append the char_ref characters to the temporary buffer
                for code_point in char_ref.chars() {
                    self.temporary_buffer.push(code_point);
                }

                // if the character reference was consumed as part of an attribute,
                // and the last character matched is not a ";" character,
                // and the next input character is either a "=" character or an alphanumeric ASCII character,
                // then flush the code points consumed as a character reference,
                // and switch to the return state
                if self.charref_in_attribute() && char_ref.chars().last() != Some(';') {
                    if let Some(c) = self.input_stream.peek() {
                        match c {
                            '=' => {
                                historical_reasons(self)?;
                                return Ok(());
                            }
                            c if c.is_ascii_alphanumeric() => {
                                historical_reasons(self)?;
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }

                if char_ref.chars().last() != Some(';') {
                    self.handle_error(TokenizerError::MissingSemicolonAfterCharacterReference)?;
                }

                self.temporary_buffer.clear();
                let char_ref_characters = NAMED_CHARACTER_REFS.get(&char_ref.as_ref()).unwrap();

                // append the char_ref characters to the temporary buffer
                for code_point in char_ref_characters.chars() {
                    self.temporary_buffer.push(code_point);
                }

                self.flush_code_points_consumed_as_character_reference()?;
                self.state = self.current_return_state()?;
            }
            None => {
                self.flush_code_points_consumed_as_character_reference()?;
                self.state = TokenizerState::AmbiguousAmpersand;
            }
        }
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#ambiguous-ampersand-state>
    pub(super) fn ambiguous_ampersand_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) if c.is_alphanumeric() => {
                let c = *c;
                if self.charref_in_attribute() {
                    self.current_attribute_mut()?.value.push(c);
                } else {
                    self.emit(HtmlToken::Character(c))?;
                }
            }
            Some(';') => {
                self.handle_error(TokenizerError::UnknownNamedCharacterReference)?;
                self.reconsume_in_state(self.current_return_state()?)?;
            }
            _ => {
                self.reconsume_in_state(self.current_return_state()?)?;
            }
        };

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-state>
    pub(super) fn numeric_character_reference_state(&mut self) -> Result<(), HtmlParseError> {
        self.character_reference_code = 0;

        match self.input_stream.next() {
            Some(c) if [chars::LATIN_SMALL_LETTER_X, chars::LATIN_CAPITAL_LETTER_X].contains(c) => {
                self.temporary_buffer.push(*c);
                self.state = TokenizerState::HexadecimalCharacterReferenceStart;
            }
            _ => {
                self.reconsume_in_state(TokenizerState::DecimalCharacterReferenceStart)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-start-state>
    pub(super) fn decimal_character_reference_start_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) if c.is_ascii_digit() => {
                self.reconsume_in_state(TokenizerState::DecimalCharacterReference)?;
            }
            _ => {
                self.handle_error(TokenizerError::AbsenceOfDigitsInNumericCharacterReference)?;
                self.flush_code_points_consumed_as_character_reference()?;
                self.reconsume_in_state(self.current_return_state()?)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-state>
    pub(super) fn decimal_character_reference_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) if c.is_ascii_digit() => {
                self.character_reference_code *= 10;
                self.character_reference_code += c
                    .to_digit(10)
                    .ok_or(HtmlParseError::new("decimal character not a digit"))?;
            }
            Some(';') => {
                self.state = TokenizerState::NumericCharacterReferenceEnd;
            }
            _ => {
                self.handle_error(TokenizerError::MissingSemicolonAfterCharacterReference)?;
                self.reconsume_in_state(TokenizerState::NumericCharacterReferenceEnd)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state>
    pub(super) fn numeric_character_reference_end_state(&mut self) -> Result<(), HtmlParseError> {
        if self.character_reference_code == 0x00 {
            self.handle_error(TokenizerError::NullCharacterReference)?;
            self.character_reference_code = 0xFFFD;
        } else if self.character_reference_code > 0x10FFFF {
            self.handle_error(TokenizerError::CharacterReferenceOutsideUnicodeRange)?;
            self.character_reference_code = 0xFFFD;
        } else if is_surrogate(self.character_reference_code) {
            self.handle_error(TokenizerError::SurrogateCharacterReference)?;
            self.character_reference_code = 0xFFFD;
        } else if is_noncharacter(self.character_reference_code) {
            self.handle_error(TokenizerError::NoncharacterCharacterReference)?;
        } else if self.character_reference_code == 0x0D
            || (is_control(self.character_reference_code)
                && !is_ascii_whitespace(self.character_reference_code))
        {
            self.handle_error(TokenizerError::ControlCharacterReference)?;
            if let Some(num) = NUMERIC_CHARACTER_REF_END_TABLE.get(&self.character_reference_code) {
                self.character_reference_code = *num;
            }
        }

        self.temporary_buffer.clear();
        self.temporary_buffer
            .push(std::char::from_u32(self.character_reference_code).unwrap());

        self.flush_code_points_consumed_as_character_reference()?;
        self.state = self.current_return_state()?;

        Ok(())
    }
}

/// <https://infra.spec.whatwg.org/#surrogate>
fn is_surrogate(code_point: u32) -> bool {
    is_leading_surrogate(code_point) || is_trailing_surrogate(code_point)
}

/// <https://infra.spec.whatwg.org/#leading-surrogate>
fn is_leading_surrogate(code_point: u32) -> bool {
    code_point >= 0xD800 && code_point <= 0xDBFF
}

/// <https://infra.spec.whatwg.org/#trailing-surrogate>
fn is_trailing_surrogate(code_point: u32) -> bool {
    code_point >= 0xDC00 && code_point <= 0xDFFF
}

/// <https://infra.spec.whatwg.org/#noncharacter>
fn is_noncharacter(code_point: u32) -> bool {
    code_point >= 0xFDD0 && code_point <= 0xFDEF
        || [
            0xFFFE, 0xFFFF, 0x1FFFE, 0x1FFFF, 0x2FFFE, 0x2FFFF, 0x3FFFE, 0x3FFFF, 0x4FFFE, 0x4FFFF,
            0x5FFFE, 0x5FFFF, 0x6FFFE, 0x6FFFF, 0x7FFFE, 0x7FFFF, 0x8FFFE, 0x8FFFF, 0x9FFFE,
            0x9FFFF, 0xAFFFE, 0xAFFFF, 0xBFFFE, 0xBFFFF, 0xCFFFE, 0xCFFFF, 0xDFFFE, 0xDFFFF,
            0xEFFFE, 0xEFFFF, 0xFFFFE, 0xFFFFF, 0x10FFFE, 0x10FFFF,
        ]
        .contains(&code_point)
}

/// <https://infra.spec.whatwg.org/#control>
fn is_control(code_point: u32) -> bool {
    is_c0_control(code_point) || (code_point >= 0x007F && code_point <= 0x009F)
}

/// <https://infra.spec.whatwg.org/#c0-control>
fn is_c0_control(code_point: u32) -> bool {
    code_point >= 0x0000 && code_point <= 0x001F
}

/// <https://infra.spec.whatwg.org/#ascii-whitespace>
fn is_ascii_whitespace(code_point: u32) -> bool {
    code_point == 0x0009
        || code_point == 0x000A
        || code_point == 0x000C
        || code_point == 0x000D
        || code_point == 0x0020
}

/// <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state>
static NUMERIC_CHARACTER_REF_END_TABLE: Lazy<HashMap<u32, u32>> = Lazy::new(|| {
    let mut table = HashMap::new();
    table.insert(0x80, 0x20AC);
    table.insert(0x82, 0x201A);
    table.insert(0x83, 0x0192);
    table.insert(0x84, 0x201E);
    table.insert(0x85, 0x2026);
    table.insert(0x86, 0x2020);
    table.insert(0x87, 0x2021);
    table.insert(0x88, 0x02C6);
    table.insert(0x89, 0x2030);
    table.insert(0x8A, 0x0160);
    table.insert(0x8B, 0x2039);
    table.insert(0x8C, 0x0152);
    table.insert(0x8E, 0x017D);
    table.insert(0x91, 0x2018);
    table.insert(0x92, 0x2019);
    table.insert(0x93, 0x201C);
    table.insert(0x94, 0x201D);
    table.insert(0x95, 0x2022);
    table.insert(0x96, 0x2013);
    table.insert(0x97, 0x2014);
    table.insert(0x98, 0x02DC);
    table.insert(0x99, 0x2122);
    table.insert(0x9A, 0x0161);
    table.insert(0x9B, 0x203A);
    table.insert(0x9C, 0x0153);
    table.insert(0x9E, 0x017E);
    table.insert(0x9F, 0x0178);
    table
});
