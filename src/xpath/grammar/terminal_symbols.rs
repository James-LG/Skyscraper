//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#terminal-symbols

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit0, digit1},
    combinator::{map_res, recognize},
    multi::many0,
    sequence::tuple,
};

use super::{
    recipes::{not_brace, not_quote, not_single_quote, Res},
    xml_names::nc_name,
};

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

pub fn string_literal(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-StringLiteral

    alt((
        tuple((
            char('"'),
            recognize(alt((escape_quot, recognize(many0(not_quote()))))),
            char('"'),
        )),
        tuple((
            char('\''),
            recognize(alt((escape_apos, recognize(many0(not_single_quote()))))),
            char('\''),
        )),
    ))(input)
    .map(|(next_input, res)| (next_input, res.1))
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

#[derive(PartialEq, Debug)]
pub struct UriQualifiedName {
    pub uri: String,
    pub name: String,
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

    tag("\"\"")(input)
}

fn escape_apos(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-EscapeApos

    tag("''")(input)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

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
            prop_assert_eq!(s, res.1);
        }

        #[test]
        fn string_literal_should_work_for_all_single_quoted_strings(s in "[^\']+") {
            let quoted_str = format!("'{}'", s);
            let res = string_literal(&quoted_str).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(s, res.1);
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
