use std::fmt::Display;

use nom::error::context;

use crate::xpath::grammar::recipes::Res;

use super::{eq_name, AtomicOrUnionType, EQName};

pub fn element_name(input: &str) -> Res<&str, ElementName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ElementName

    context("element_name", eq_name)(input).map(|(next_input, res)| (next_input, ElementName(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct ElementName(pub EQName);

pub fn type_name(input: &str) -> Res<&str, TypeName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-TypeName

    context("type_name", eq_name)(input).map(|(next_input, res)| (next_input, TypeName(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct TypeName(pub EQName);

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn attribute_name(input: &str) -> Res<&str, AttributeName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AttributeName

    context("attribute_name", eq_name)(input)
        .map(|(next_input, res)| (next_input, AttributeName(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct AttributeName(pub EQName);

pub fn atomic_or_union_type(input: &str) -> Res<&str, AtomicOrUnionType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AtomicOrUnionType
    context("atomic_or_union_type", eq_name)(input)
        .map(|(next_input, res)| (next_input, AtomicOrUnionType(res)))
}
