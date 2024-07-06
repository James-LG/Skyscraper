use crate::html::grammar::{HtmlParseError, HtmlParseErrorType};

use super::{HtmlToken, Tokenizer, TokenizerState};

impl<'a> Tokenizer<'a> {
    pub(crate) fn data_state(&mut self) -> Result<(), HtmlParseError> {
        match self.input_stream.next() {
            Some(c) => match c {
                '&' => {
                    self.return_state = Some(TokenizerState::Data);
                    self.state = TokenizerState::CharacterReference;
                }
                '<' => {
                    self.state = TokenizerState::TagOpen;
                }
                '\0' => {
                    let current_input_character = *c;
                    self.handle_error(HtmlParseErrorType::UnexpectedNullCharacter)?;

                    self.emit(vec![HtmlToken::Character(current_input_character)]);
                }
                _ => {
                    let current_input_character = *c;
                    self.emit(vec![HtmlToken::Character(current_input_character)]);
                }
            },
            None => self.emit(vec![HtmlToken::EndOfFile]),
        };

        Ok(())
    }
}
