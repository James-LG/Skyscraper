//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#node-tests

use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, character::complete::char, sequence::tuple};

use crate::xpath::grammar::{
    recipes::Res,
    terminal_symbols::braced_uri_literal,
    types::{eq_name, kind_test, EQName, KindTest},
    xml_names::nc_name,
};

pub fn node_test(input: &str) -> Res<&str, NodeTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NodeTest

    fn kind_test_map(input: &str) -> Res<&str, NodeTest> {
        kind_test(input).map(|(next_input, res)| (next_input, NodeTest::KindTest(res)))
    }

    fn name_test_map(input: &str) -> Res<&str, NodeTest> {
        name_test(input).map(|(next_input, res)| (next_input, NodeTest::NameTest(res)))
    }

    alt((kind_test_map, name_test_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum NodeTest {
    KindTest(KindTest),
    NameTest(NameTest),
}

impl Display for NodeTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeTest::KindTest(x) => write!(f, "{}", x),
            NodeTest::NameTest(x) => write!(f, "{}", x),
        }
    }
}

fn name_test(input: &str) -> Res<&str, NameTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NameTest

    fn eq_name_map(input: &str) -> Res<&str, NameTest> {
        eq_name(input).map(|(next_input, res)| (next_input, NameTest::Name(res)))
    }

    fn wildcard_map(input: &str) -> Res<&str, NameTest> {
        wildcard(input).map(|(next_input, res)| (next_input, NameTest::Wildcard(res)))
    }

    alt((eq_name_map, wildcard_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum NameTest {
    Name(EQName),
    Wildcard(Wildcard),
}

impl Display for NameTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameTest::Name(x) => write!(f, "{}", x),
            NameTest::Wildcard(x) => write!(f, "{}", x),
        }
    }
}

fn wildcard(input: &str) -> Res<&str, Wildcard> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-Wildcard

    fn prefixed_name_map(input: &str) -> Res<&str, Wildcard> {
        tuple((tag("*:"), nc_name))(input)
            .map(|(next_input, res)| (next_input, Wildcard::PrefixedName(res.1.to_string())))
    }

    fn suffixed_name_map(input: &str) -> Res<&str, Wildcard> {
        tuple((nc_name, tag(":*")))(input)
            .map(|(next_input, res)| (next_input, Wildcard::SuffixedName(res.0.to_string())))
    }

    fn braced_uri_map(input: &str) -> Res<&str, Wildcard> {
        tuple((braced_uri_literal, char('*')))(input)
            .map(|(next_input, res)| (next_input, Wildcard::BracedUri(res.0.to_string())))
    }

    fn simple_map(input: &str) -> Res<&str, Wildcard> {
        char('*')(input).map(|(next_input, _res)| (next_input, Wildcard::Simple))
    }

    alt((
        prefixed_name_map,
        suffixed_name_map,
        braced_uri_map,
        simple_map,
    ))(input)
}

#[derive(PartialEq, Debug)]
pub enum Wildcard {
    Simple,
    PrefixedName(String),
    SuffixedName(String),
    BracedUri(String),
}

impl Display for Wildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wildcard::Simple => write!(f, "*"),
            Wildcard::PrefixedName(x) => write!(f, "*:{}", x),
            Wildcard::SuffixedName(x) => write!(f, "{}:*", x),
            Wildcard::BracedUri(x) => write!(f, "{}*", x),
        }
    }
}
