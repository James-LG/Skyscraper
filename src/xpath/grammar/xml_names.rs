use std::fmt::Display;

use nom::{
    branch::alt,
    character::complete::char,
    combinator::recognize,
    multi::many0,
    sequence::{pair, separated_pair},
};

use super::recipes::{alphabetic, numeric, Res};

pub fn nc_name(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/REC-xml-names/#NT-NCName
    fn name_start_char_no_colon(input: &str) -> Res<&str, char> {
        alt((alphabetic(), char('_')))(input)
    }

    fn name_char_no_colon(input: &str) -> Res<&str, char> {
        alt((name_start_char_no_colon, char('-'), char('.'), numeric()))(input)
    }

    fn name_no_colon(input: &str) -> Res<&str, &str> {
        recognize(pair(name_start_char_no_colon, many0(name_char_no_colon)))(input)
    }

    name_no_colon(input)
}

fn local_part(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/REC-xml-names/#NT-LocalPart
    nc_name(input)
}

fn unprefixed_name(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/REC-xml-names/#NT-UnprefixedName
    local_part(input)
}

fn prefix(input: &str) -> Res<&str, &str> {
    // https://www.w3.org/TR/REC-xml-names/#NT-Prefix
    nc_name(input)
}

fn prefixed_name(input: &str) -> Res<&str, PrefixedName> {
    // https://www.w3.org/TR/REC-xml-names/#NT-PrefixedName
    separated_pair(prefix, char(':'), local_part)(input).map(|(next_input, res)| {
        (
            next_input,
            PrefixedName {
                prefix: res.0.to_string(),
                local_part: res.1.to_string(),
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct PrefixedName {
    pub prefix: String,
    pub local_part: String,
}

impl Display for PrefixedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.local_part)
    }
}

pub fn qname(input: &str) -> Res<&str, QName> {
    // https://www.w3.org/TR/REC-xml-names/#NT-QName
    fn prefixedname_map(input: &str) -> Res<&str, QName> {
        prefixed_name(input).map(|(next_input, res)| (next_input, QName::PrefixedName(res)))
    }

    fn unprefixed_name_map(input: &str) -> Res<&str, QName> {
        unprefixed_name(input)
            .map(|(next_input, res)| (next_input, QName::UnprefixedName(res.to_string())))
    }

    alt((prefixedname_map, unprefixed_name_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum QName {
    PrefixedName(PrefixedName),
    UnprefixedName(String),
}

impl Display for QName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QName::PrefixedName(x) => write!(f, "{}", x),
            QName::UnprefixedName(x) => write!(f, "{}", x),
        }
    }
}

#[cfg(test)]
pub mod arb_strats {
    use proptest::prelude::*;

    prop_compose! {
        pub fn valid_nc_name()(s in "[A-Z_a-z]+") -> String {
            s
        }
    }

    prop_compose! {
        pub fn invalid_nc_name()(s in "[^A-Z_a-z]*") -> String {
            s
        }
    }
}

#[cfg(test)]
mod test {
    use super::arb_strats::*;
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn ncname_should_not_match_colon() {
        // arrange
        let input = "hello:world";

        // act
        let res = nc_name(input);

        // assert
        assert_eq!(res, Ok((":world", "hello")))
    }

    #[test]
    fn qname_should_match_prefixed_name() {
        // arrange
        let input = "hello:world";

        // act
        let res = qname(input);

        // assert
        assert_eq!(
            res,
            Ok((
                "",
                QName::PrefixedName(PrefixedName {
                    prefix: String::from("hello"),
                    local_part: String::from("world")
                })
            ))
        )
    }

    proptest! {
        #[test]
        fn nc_name_should_work_for_all_valid_names(s in valid_nc_name()) {
            let res = nc_name(&s).unwrap();

            prop_assert_eq!("", res.0, "next input not empty");
            prop_assert_eq!(&s, res.1);
        }

        #[test]
        fn nc_name_should_fail_for_all_invalid_names(s in invalid_nc_name()) {
            let res = nc_name(&s);

            prop_assert!(res.is_err());
        }
    }
}
