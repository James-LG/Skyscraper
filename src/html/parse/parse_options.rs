//! Create options for [Parser](super::Parser).

use super::malformed_html_handlers::{ErrorMismatchedTagHandler, MismatchedTagHandler};

/// Options for [Parser](super::Parser).
pub struct ParseOptions {
    /// Defines the method for handling an end tag that doesn't match the currently opened tag.
    pub mismatched_tag_handler: Box<dyn MismatchedTagHandler>,
}

impl ParseOptions {
    /// Create a new [ParseOptions] with default values.
    pub fn new() -> Self {
        Self {
            mismatched_tag_handler: Box::new(ErrorMismatchedTagHandler::new()),
        }
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        ParseOptions::new()
    }
}

/// Builds [ParseOptions] for the [Parser](crate::html::parse::Parser).
///
/// See [ParseOptions] for the default values used if not set by the builder.
///
/// Example usage:
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use skyscraper::html::parse::{Parser, ParseOptionsBuilder, malformed_html_handlers::VoidMismatchedTagHandler};
///
/// let options = ParseOptionsBuilder::new()
///     .with_mismatched_tag_handler(Box::new(VoidMismatchedTagHandler::new(None)))
///     .build();
///
/// let parser = Parser::new(options);
/// # Ok(())
/// # }
/// ```
pub struct ParseOptionsBuilder {
    reducers: Vec<Box<dyn FnOnce(ParseOptions) -> ParseOptions>>,
}

impl ParseOptionsBuilder {
    /// Creates a new [ParseOptionsBuilder].
    pub fn new() -> Self {
        Self {
            reducers: Vec::new(),
        }
    }

    /// Set the type of [MismatchedTagHandler] the parser should use.
    pub fn with_mismatched_tag_handler(mut self, handler: Box<dyn MismatchedTagHandler>) -> Self {
        let reducer = |options| ParseOptions {
            mismatched_tag_handler: handler,
            ..options
        };
        self.reducers.push(Box::new(reducer));
        self
    }

    /// Build the [ParseOptions].
    pub fn build(self) -> ParseOptions {
        self.reducers
            .into_iter()
            .fold(ParseOptions::new(), |options, f| f(options))
    }
}

impl Default for ParseOptionsBuilder {
    fn default() -> Self {
        ParseOptionsBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::html::parse::{
        malformed_html_handlers::{MismatchedTagHandlerContext, MockMismatchedTagHandler},
        ParserState,
    };

    use super::*;

    #[test]
    fn with_mismatched_tag_handler_should_set_handler() {
        // arrange
        let builder = ParseOptionsBuilder::new();
        let mut handler = MockMismatchedTagHandler::new();
        let mut context = ParserState {
            ..Default::default()
        };
        let context = MismatchedTagHandlerContext {
            open_tag_name: "hi",
            close_tag_name: "bye",
            parser_state: &mut context,
        };

        handler.expect_invoke().times(1).returning(|_| Ok(()));

        // act
        let options = builder
            .with_mismatched_tag_handler(Box::new(handler))
            .build();

        // assert
        assert!(matches!(
            options.mismatched_tag_handler.invoke(context),
            Ok(())
        ));
    }
}
