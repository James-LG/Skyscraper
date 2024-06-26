//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#terminal-symbols>

use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{char, digit0, digit1},
    combinator::{map_res, opt, peek, recognize},
    error::context,
    multi::{fold_many0, many0, many1},
    sequence::{terminated, tuple},
};

use super::{
    recipes::{not_brace, not_quote, not_single_quote, Res},
    xml_names::nc_name,
};

pub fn symbol_separator(input: &str) -> Res<&str, ()> {
    //https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-terminal-delimitation

    fn comment_map(input: &str) -> Res<&str, char> {
        comment(input).map(|(next_input, _res)| (next_input, ' '))
    }
    many1(alt((
        char(' '),
        char('\t'),
        char('\r'),
        char('\n'),
        comment_map,
    )))(input)
    .map(|(next_input, _res)| (next_input, ()))
}

pub fn comment(input: &str) -> Res<&str, Comment> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Comment
    fn comment_contents_map(input: &str) -> Res<&str, CommentItem> {
        comment_contents(input)
            .map(|(next_input, res)| (next_input, CommentItem::CommentContents(res)))
    }

    fn comment_map(input: &str) -> Res<&str, CommentItem> {
        comment(input).map(|(next_input, res)| (next_input, CommentItem::Comment(Box::new(res))))
    }

    context(
        "comment",
        tuple((
            tag("(:"),
            many0(alt((comment_contents_map, comment_map))),
            tag(":)"),
        )),
    )(input)
    .map(|(next_input, res)| {
        let items = res.1;
        (next_input, Comment { items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct Comment {
    pub items: Vec<CommentItem>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CommentItem {
    Comment(Box<Comment>),
    CommentContents(CommentContents),
}

pub fn comment_contents(input: &str) -> Res<&str, CommentContents> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-CommentContents
    context(
        "comment_contents",
        terminated(
            take_till1(|c: char| c == '(' || c == ':'),
            peek(alt((tag("(:"), tag(":)")))),
        ),
    )(input)
    .map(|(next_input, res)| (next_input, CommentContents(res.to_string())))
}

#[derive(PartialEq, Debug, Clone)]
pub struct CommentContents(String);

pub fn integer_literal(input: &str) -> Res<&str, u32> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-IntegerLiteral
    map_res(digit1, str::parse)(input)
}

pub fn decimal_literal(input: &str) -> Res<&str, f32> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-DecimalLiteral

    map_res(
        alt((
            recognize(tuple((char('.'), digit1))),
            recognize(tuple((digit1, char('.'), digit0))),
        )),
        str::parse,
    )(input)
}

pub fn double_literal(input: &str) -> Res<&str, f64> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-DoubleLiteral

    map_res(
        recognize(tuple((
            alt((
                recognize(tuple((char('.'), digit1))),
                recognize(tuple((digit1, opt(tuple((char('.'), digit0)))))),
            )),
            alt((char('e'), char('E'))),
            opt(alt((char('+'), char('-')))),
            digit1,
        ))),
        str::parse,
    )(input)
}

pub fn string_literal(input: &str) -> Res<&str, StringLiteral> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-StringLiteral

    fn double_quoted_map(input: &str) -> Res<&str, StringLiteral> {
        context(
            "string_literal::double_quoted_map",
            tuple((
                char('"'),
                fold_many0(
                    alt((escape_quot, recognize(not_quote()))),
                    || String::from(""),
                    |acc, item| format!("{}{}", acc, item),
                ),
                char('"'),
            )),
        )(input)
        .map(|(next_input, res)| {
            (
                next_input,
                StringLiteral {
                    value: res.1.to_string(),
                    quotation_type: QuotationType::Double,
                },
            )
        })
    }

    fn single_quoted_map(input: &str) -> Res<&str, StringLiteral> {
        context(
            "string_literal::single_quoted_map",
            tuple((
                char('\''),
                fold_many0(
                    alt((escape_apos, recognize(not_single_quote()))),
                    || String::from(""),
                    |acc, item| format!("{}{}", acc, item),
                ),
                char('\''),
            )),
        )(input)
        .map(|(next_input, res)| {
            (
                next_input,
                StringLiteral {
                    value: res.1.to_string(),
                    quotation_type: QuotationType::Single,
                },
            )
        })
    }

    context(
        "string_literal",
        alt((single_quoted_map, double_quoted_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub struct StringLiteral {
    pub value: String,
    quotation_type: QuotationType,
}

impl Display for StringLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.quotation_type {
            QuotationType::Single => write!(f, "'{}'", self.value),
            QuotationType::Double => write!(f, "\"{}\"", self.value),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum QuotationType {
    Single,
    Double,
}

pub fn uri_qualified_name(input: &str) -> Res<&str, UriQualifiedName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-URIQualifiedName

    tuple((braced_uri_literal, nc_name))(input).map(|(next_input, res)| {
        (
            next_input,
            UriQualifiedName {
                uri: res.0.to_string(),
                name: res.1.to_string(),
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct UriQualifiedName {
    pub uri: String,
    pub name: String,
}

impl Display for UriQualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.uri, self.name)
    }
}

pub fn braced_uri_literal(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-BracedURILiteral

    tuple((
        char('Q'),
        char('{'),
        recognize(many0(not_brace())),
        char('}'),
    ))(input)
    .map(|(next_input, res)| (next_input, res.2))
}

fn escape_quot(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-EscapeQuot

    tag("\"\"")(input).map(|(next_input, _)| (next_input, "\""))
}

fn escape_apos(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-EscapeApos

    tag("''")(input).map(|(next_input, _)| (next_input, "'"))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn string_literal_should_allow_escaped_double_quotes() {
        // arrange
        let input = format!("\"{}\"", "He said, \"\"I don't like it.\"\"");

        // act
        let (next_input, res) = string_literal(&input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res,
            StringLiteral {
                value: String::from("He said, \"I don't like it.\""),
                quotation_type: QuotationType::Double
            }
        )
    }

    #[test]
    fn string_literal_should_allow_escaped_single_quotes() {
        // arrange
        let input = format!("'{}'", "He said, \"I don''t like it.\"");

        // act
        let (next_input, res) = string_literal(&input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res,
            StringLiteral {
                value: String::from("He said, \"I don't like it.\""),
                quotation_type: QuotationType::Single
            }
        )
    }

    #[test]
    fn comment_should_match_comment() {
        // arrange
        let input = "(: hello world :)";

        // act
        let (next_input, res) = comment(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res,
            Comment {
                items: vec![CommentItem::CommentContents(CommentContents(String::from(
                    " hello world "
                )))]
            }
        )
    }

    #[test]
    fn comment_should_allow_nested_comments() {
        // arrange
        let input = "(: hello (: world :) :)";

        // act
        let (next_input, res) = comment(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res,
            Comment {
                items: vec![
                    CommentItem::CommentContents(CommentContents(String::from(" hello "))),
                    CommentItem::Comment(Box::new(Comment {
                        items: vec![CommentItem::CommentContents(CommentContents(String::from(
                            " world "
                        )))]
                    })),
                    CommentItem::CommentContents(CommentContents(String::from(" ")))
                ]
            }
        )
    }

    #[test]
    fn double_should_match_scientific_notation_positive() {
        // arrange
        let input = "1.0e+2";

        // act
        let (next_input, res) = double_literal(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, 100.0);
    }

    #[test]
    fn double_should_match_scientific_notation_negative() {
        // arrange
        let input = "1.0e-2";

        // act
        let (next_input, res) = double_literal(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, 0.01);
    }

    proptest! {
        #[test]
        fn integer_literal_should_work_for_all_valid_u32(i in any::<u32>()) {
            let i_str = format!("{:?}", i);
            let res = integer_literal(&i_str).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(i, res.1);
        }

        #[test]
        fn decimal_literal_should_work_for_all_valid_f32(i in 0f32..10_000.0) {
            let i_str = format!("{:?}", i);
            let res = decimal_literal(&i_str).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(i, res.1);
        }

        #[test]
        fn double_literal_should_work_for_all_valid_f64(i in 0f64..10_000.0) {
            let i_str = format!("{:e}", i);
            let res = double_literal(&i_str).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(i, res.1);
        }

        #[test]
        fn string_literal_should_work_for_all_quoted_strings(s in "[^\"]+") {
            let quoted_str = format!("\"{}\"", s);
            let res = string_literal(&quoted_str).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(s, res.1.value);
        }

        #[test]
        fn string_literal_should_work_for_all_single_quoted_strings(s in "[^\']+") {
            let quoted_str = format!("'{}'", s);
            let res = string_literal(&quoted_str).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(s, res.1.value);
        }

        #[test]
        fn braced_uri_literal_should_work_for_all_strings(s in "[^{}]*") {
            let quoted_str = format!("Q{{{}}}", s);
            let res = braced_uri_literal(&quoted_str).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(s, res.1);
        }
    }
}
