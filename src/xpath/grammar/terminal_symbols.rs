//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#terminal-symbols

use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{char, digit0, digit1},
    combinator::{map_res, peek, recognize},
    error::{context, ParseError, VerboseError},
    multi::{fold_many0, many0, many1},
    sequence::{terminated, tuple},
    Err as NomErr, IResult, Offset, Parser,
};

use super::{
    recipes::{not_brace, not_quote, not_single_quote, Res},
    xml_names::nc_name,
};

pub fn sep<I: Clone, O, E: ParseError<I>, List: Sep<I, O, E>>(
    mut l: List,
) -> impl FnMut(I) -> IResult<I, O, E> {
    move |i: I| l.parse(i)
}

pub trait Sep<I, O, E> {
    /// Tests each parser with a symbol separator between them.
    fn parse(&mut self, input: I) -> IResult<I, O, E>;
}

fn sep_parse<'a, Output, A: Parser<&'a str, Output, VerboseError<&'a str>>>(
    input: &'a str,
    parser: &mut A,
) -> Result<(&'a str, Output), NomErr<VerboseError<&'a str>>> {
    let symbol_sep_res = symbol_separator(input);

    match symbol_sep_res {
        Ok((next_input, _)) => {
            // If there's a separator, the parser must capture something.
            match parser.parse(next_input) {
                Ok((parser_next_input, parser_res)) => {
                    let length = next_input.offset(parser_next_input);

                    // If the parser didn't capture anything, and there was a separator, uncapture the separator.
                    // Return the original input like nothing happened.
                    if length == 0 {
                        Ok((input, parser_res))
                    }
                    // If the parser did capture something, and there was a separator, we have a successful match.
                    else {
                        Ok((parser_next_input, parser_res))
                    }
                }
                Err(_) => Err(NomErr::Error(VerboseError::from_error_kind(
                    input,
                    nom::error::ErrorKind::Many0,
                ))),
            }
        }
        Err(e) => {
            // If there's no separator, this is only an error if the parser wanted to capture something.
            // i.e. If the parser is optional, it's not an error.
            match parser.parse(input) {
                Ok((parser_next_input, parser_res)) => {
                    let length = input.offset(parser_next_input);

                    // If the parser didn't capture anything, and there was no separator, this is not an error.
                    // Return the original input like nothing happened.
                    if length == 0 {
                        Ok((input, parser_res))
                    }
                    // If the parser captured something, but there was no separator, this is an error.
                    else {
                        Err(e)
                    }
                }
                Err(_) => {
                    // The separator is what really failed here, not the parser.
                    Err(e)
                }
            }
        }
    }
}

impl<
        'a,
        Output1,
        Output2,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
    > Sep<&'a str, (Output1, Output2), VerboseError<&'a str>> for (A, B)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2), VerboseError<&'a str>> {
        let (input, res1) = self.0.parse(input)?;
        let (input, res2) = sep_parse(input, &mut self.1)?;
        Ok((input, (res1, res2)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
    > Sep<&'a str, (Output1, Output2, Output3), VerboseError<&'a str>> for (A, B, C)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3), VerboseError<&'a str>> {
        let (input, res1) = self.0.parse(input)?;
        let (input, res2) = sep_parse(input, &mut self.1)?;
        let (input, res3) = sep_parse(input, &mut self.2)?;
        Ok((input, (res1, res2, res3)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        Output4,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
        D: Parser<&'a str, Output4, VerboseError<&'a str>>,
    > Sep<&'a str, (Output1, Output2, Output3, Output4), VerboseError<&'a str>> for (A, B, C, D)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3, Output4), VerboseError<&'a str>> {
        let (input, res1) = self.0.parse(input)?;
        let (input, res2) = sep_parse(input, &mut self.1)?;
        let (input, res3) = sep_parse(input, &mut self.2)?;
        let (input, res4) = sep_parse(input, &mut self.3)?;
        Ok((input, (res1, res2, res3, res4)))
    }
}

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
        alt((
            recognize(tuple((char('.'), digit1))),
            recognize(tuple((digit1, char('.'), digit0))),
        )),
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
    use nom::combinator::opt;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn sep_should_allow_whitespace_between() {
        // arrange
        let input = "hello world";

        // act
        let (next_input, res) = sep((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn sep_should_allow_multiple_whitespace_before_and_after() {
        // arrange
        let input = "hello  world";

        // act
        let (next_input, res) = sep((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn sep_should_match_when_opt_doesnt() {
        // arrange
        let input = "hello bob";

        // act
        let (next_input, res) = sep((tag("hello"), opt(tag("world"))))(input).unwrap();

        // assert
        assert_eq!(next_input, " bob");
        assert_eq!(res, ("hello", None))
    }

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
            let i_str = format!("{:?}", i);
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
