//! Methods for handling malformed HTML input.
//!
//! Methods are used by [Parser](crate::html::parse::Parser).
//!
//! See [ParseOptionsBuilder](crate::html::parse::parse_options::ParseOptionsBuilder)
//! for details on how to select these handlers.

use log::log;

#[cfg(test)]
use mockall::automock;

use super::{ParseError, ParserState};

/// Context given to a [MismatchedTagHandler].
/// Includes the entire mutable [ParserState], plus some additional context.
pub struct MismatchedTagHandlerContext<'a> {
    /// The name of the open tag.
    pub open_tag_name: &'a str,

    /// The name of the close tag that did not match the open tag name.
    pub close_tag_name: &'a str,

    /// The mutable state of the parser when the tag mismatch was encountered.
    pub parser_state: &'a mut ParserState,
}

/// Trait representing a method for handling a closing tag that does not have the same name as the corresponding opening tag.
/// This happens when the HTML input is malformed.
///
/// Examples:
///
/// 1. End tag is incorrect:
///
///     ```html
///     <html>
///     </body> <!-- This should have been closing 'html'. -->
///     ```
///
/// 2. End tag is missing entirely:
///
///     ```html
///     <html>
///       <div>
///     </html> <!-- The closing 'div' is missing, which makes it seem like this 'html' closing tag is mismatched with the opening 'div'. -->
///     ```
#[cfg_attr(test, automock)]
pub trait MismatchedTagHandler {
    /// Performs the handling by (optionally) changing the [ParserState](crate::html::parse::ParserState).
    ///
    /// If the result is Ok, the parser will continue. If the result is Err, it will return the error immediately.
    fn invoke<'a>(&self, context: MismatchedTagHandlerContext<'a>) -> Result<(), ParseError>;
}

/// Returns an error that an end tag did not match an opening tag.
/// This cuts the parsing short and causes the parser to immediately return the error.
pub struct ErrorMismatchedTagHandler {}

impl ErrorMismatchedTagHandler {
    /// Creates a new [ErrorMismatchedTagHandler].
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ErrorMismatchedTagHandler {
    fn default() -> Self {
        ErrorMismatchedTagHandler::new()
    }
}

impl MismatchedTagHandler for ErrorMismatchedTagHandler {
    fn invoke(&self, context: MismatchedTagHandlerContext) -> Result<(), ParseError> {
        Err(ParseError::EndTagMismatch {
            end_name: context.close_tag_name.to_string(),
            open_name: context.open_tag_name.to_string(),
        })
    }
}

/// Does not error, and performs no special handling, effectively ignoring the mismatching tag.
/// Optionally logs a message that a tag mismatch has occurred.
///
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use skyscraper::html::parse::{Parser, ParseOptionsBuilder, malformed_html_handlers::VoidMismatchedTagHandler};
/// let input = r#"
///     <html>
///         <body>
///             hello
///             <div>
///                 there
///         </body>
///         friend
///     </span>"#;
///
/// let options = ParseOptionsBuilder::new()
///     .with_mismatched_tag_handler(Box::new(VoidMismatchedTagHandler::new(None)))
///     .build();
///
/// let html = Parser::new(options).parse(input)?;
///
/// let output = r#"<html>
///     <body>
///         hello
///         <div>
///             there
///         </div>
///         friend
///     </body>
/// </html>
/// "#;
/// assert_eq!(html.to_string(), output);
/// # Ok(())
/// # }
/// ```
pub struct VoidMismatchedTagHandler {
    log_level: Option<log::Level>,
}

impl VoidMismatchedTagHandler {
    /// Creates a new [VoidMismatchedTagHandler].
    ///
    /// Set `log_level` to `None` if no logs are desired.
    pub fn new(log_level: Option<log::Level>) -> Self {
        Self { log_level }
    }
}

impl MismatchedTagHandler for VoidMismatchedTagHandler {
    fn invoke(&self, context: MismatchedTagHandlerContext) -> Result<(), ParseError> {
        if let Some(log_level) = self.log_level {
            log!(
                log_level,
                "End tag of {} mismatches opening tag of {}",
                context.close_tag_name,
                context.open_tag_name
            );
        }

        Ok(())
    }
}

/// Attempts to close a missing tag by checking if the parent of the current tag matches the mismatching end tag.
/// If the parent does not match, it ignores the mismatch without performing any additional handling, much like [VoidMismatchedTagHandler].
///
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use skyscraper::html::parse::{Parser, ParseOptionsBuilder, malformed_html_handlers::CloseMismatchedTagHandler};
/// let input = r#"
///     <html>
///         <body>
///             hello
///             <div>
///                 there
///         </body>
///         friend
///     </span>"#;
///
/// let options = ParseOptionsBuilder::new()
///     .with_mismatched_tag_handler(Box::new(CloseMismatchedTagHandler::new(None)))
///     .build();
///
/// let html = Parser::new(options).parse(input)?;
///
/// let output = r#"<html>
///     <body>
///         hello
///         <div>
///             there
///         </div>
///     </body>
///     friend
/// </html>
/// "#;
/// assert_eq!(html.to_string(), output);
/// # Ok(())
/// # }
/// ```
pub struct CloseMismatchedTagHandler {
    log_level: Option<log::Level>,
}

impl CloseMismatchedTagHandler {
    /// Creates a new [CloseMismatchedTagHandler].
    ///
    /// Set `log_level` to `None` if no logs are desired.
    pub fn new(log_level: Option<log::Level>) -> Self {
        Self { log_level }
    }
}

impl MismatchedTagHandler for CloseMismatchedTagHandler {
    fn invoke(&self, context: MismatchedTagHandlerContext) -> Result<(), ParseError> {
        if let Some(log_level) = self.log_level {
            log!(
                log_level,
                "End tag of {} mismatches opening tag of {}",
                context.close_tag_name,
                context.open_tag_name
            );
        }

        let cur_key = context
            .parser_state
            .arena
            .get(context.parser_state.cur_key_o.unwrap())
            .unwrap();

        if let Some(parent_key) = cur_key.parent() {
            // do not move up to the root node, otherwise the parser will attempt to move up past it.
            if parent_key != context.parser_state.root_key_o.unwrap() {
                let parent = context
                    .parser_state
                    .arena
                    .get(parent_key)
                    .unwrap()
                    .get()
                    .unwrap_tag();

                // if the parent name matches the end tag of the mistmatch, assume the parent's end tag is missing and move up to close it.
                // otherwise, ignore the mismatch and hope for the best.
                if parent.name == context.close_tag_name {
                    if let Some(log_level) = self.log_level {
                        log!(
                            log_level,
                            "Parent tag matches end tag {}; assuming end tag is missing, closing current tag and parent tag",
                            parent.name,
                        );
                    }
                    context.parser_state.cur_key_o = Some(parent_key);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::html::parse::{parse_options::ParseOptionsBuilder, Parser};

    const HTML_MISMATCHED_END_TAG: &'static str = r#"
        <html>
            <body>
                foo
                <span>
                    bar
                </div>
                <b>
                    baz
                </b>
                bat
            </body>
            <footer>
                foot
            </footer>
        </html>
        "#;

    const HTML_MISSING_END_TAG: &'static str = r#"
        <html>
            <body>
                foo
                <span>
                    bar
                    <b>
                        baz
                    </b>
                    bat
            </body>
            <footer>
                foot
            </footer>
        </html>
        "#;

    #[test]
    fn error_handler_should_err_for_missing_end_tag() {
        // arrange
        let parse_options = ParseOptionsBuilder::new()
            .with_mismatched_tag_handler(Box::new(ErrorMismatchedTagHandler::new()))
            .build();
        let parser = Parser::new(parse_options);

        // act
        let result = parser.parse(HTML_MISSING_END_TAG);

        // assert
        assert!(matches!(result, Err(ParseError::EndTagMismatch { .. })));

        if let Err(ParseError::EndTagMismatch {
            end_name,
            open_name,
        }) = result
        {
            assert_eq!(open_name, String::from("span"));
            assert_eq!(end_name, String::from("body"));
        }
    }

    #[test]
    fn error_handler_should_err_for_mismatched_end_tag() {
        // arrange
        let parse_options = ParseOptionsBuilder::new()
            .with_mismatched_tag_handler(Box::new(ErrorMismatchedTagHandler::new()))
            .build();
        let parser = Parser::new(parse_options);

        // act
        let result = parser.parse(HTML_MISMATCHED_END_TAG);

        // assert
        assert!(matches!(result, Err(ParseError::EndTagMismatch { .. })));

        if let Err(ParseError::EndTagMismatch {
            end_name,
            open_name,
        }) = result
        {
            assert_eq!(open_name, String::from("span"));
            assert_eq!(end_name, String::from("div"));
        }
    }

    #[test]
    fn void_handler_should_ignore_mismatch_for_missing_end_tag() {
        // arrange
        let parse_options = ParseOptionsBuilder::new()
            .with_mismatched_tag_handler(Box::new(VoidMismatchedTagHandler::new(None)))
            .build();
        let parser = Parser::new(parse_options);

        // act
        let result = parser.parse(HTML_MISSING_END_TAG).unwrap();

        // assert
        let output = result.to_string();
        let expected_output = indoc!(
            r#"
            <html>
                <body>
                    foo
                    <span>
                        bar
                        <b>
                            baz
                        </b>
                        bat
                    </span>
                    <footer>
                        foot
                    </footer>
                </body>
            </html>
            "#
        );

        assert_eq!(output, expected_output);
    }

    #[test]
    fn void_handler_should_ignore_mismatch_for_mismatched_end_tag() {
        // arrange
        let parse_options = ParseOptionsBuilder::new()
            .with_mismatched_tag_handler(Box::new(VoidMismatchedTagHandler::new(None)))
            .build();
        let parser = Parser::new(parse_options);

        // act
        let result = parser.parse(HTML_MISMATCHED_END_TAG).unwrap();

        // assert
        let output = result.to_string();
        let expected_output = indoc!(
            r#"
            <html>
                <body>
                    foo
                    <span>
                        bar
                    </span>
                    <b>
                        baz
                    </b>
                    bat
                </body>
                <footer>
                    foot
                </footer>
            </html>
            "#
        );

        assert_eq!(output, expected_output);
    }

    #[test]
    fn close_handler_should_close_both_tags_for_missing_end_tag() {
        // arrange
        let parse_options = ParseOptionsBuilder::new()
            .with_mismatched_tag_handler(Box::new(CloseMismatchedTagHandler::new(None)))
            .build();
        let parser = Parser::new(parse_options);

        // act
        let result = parser.parse(HTML_MISSING_END_TAG).unwrap();

        // assert
        let output = result.to_string();
        let expected_output = indoc!(
            r#"
            <html>
                <body>
                    foo
                    <span>
                        bar
                        <b>
                            baz
                        </b>
                        bat
                    </span>
                </body>
                <footer>
                    foot
                </footer>
            </html>
            "#
        );

        assert_eq!(output, expected_output);
    }

    #[test]
    fn close_handler_should_ignore_mismatch_for_mismatched_end_tag() {
        // arrange
        let parse_options = ParseOptionsBuilder::new()
            .with_mismatched_tag_handler(Box::new(CloseMismatchedTagHandler::new(None)))
            .build();
        let parser = Parser::new(parse_options);

        // act
        let result = parser.parse(HTML_MISMATCHED_END_TAG).unwrap();

        // assert
        let output = result.to_string();
        let expected_output = indoc!(
            r#"
            <html>
                <body>
                    foo
                    <span>
                        bar
                    </span>
                    <b>
                        baz
                    </b>
                    bat
                </body>
                <footer>
                    foot
                </footer>
            </html>
            "#
        );

        assert_eq!(output, expected_output);
    }
}
