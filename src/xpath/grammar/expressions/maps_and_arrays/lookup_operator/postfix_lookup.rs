//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-postfix-lookup

use nom::{character::complete::char, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::unary_lookup::{key_specifier, KeySpecifier};

pub fn lookup(input: &str) -> Res<&str, Lookup> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Lookup
    tuple((char('?'), key_specifier))(input).map(|(next_input, res)| (next_input, Lookup(res.1)))
}

pub struct Lookup(pub KeySpecifier);
