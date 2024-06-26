//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-postfix-lookup>

use std::fmt::Display;

use nom::{character::complete::char, error::context, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::unary_lookup::{key_specifier, KeySpecifier};

pub fn lookup(input: &str) -> Res<&str, Lookup> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Lookup
    context("lookup", tuple((char('?'), key_specifier)))(input)
        .map(|(next_input, res)| (next_input, Lookup(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct Lookup(pub KeySpecifier);

impl Display for Lookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_should_parse() {
        // arrange
        let input = "?2";

        // act
        let (next_input, res) = lookup(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
