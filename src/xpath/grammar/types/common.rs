use crate::xpath::grammar::recipes::Res;

use super::{eq_name, AtomicOrUnionType, EQName};

pub fn element_name(input: &str) -> Res<&str, ElementName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ElementName

    eq_name(input).map(|(next_input, res)| (next_input, ElementName(res)))
}

#[derive(PartialEq, Debug)]
pub struct ElementName(pub EQName);

pub fn type_name(input: &str) -> Res<&str, TypeName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-TypeName

    eq_name(input).map(|(next_input, res)| (next_input, TypeName(res)))
}

#[derive(PartialEq, Debug)]
pub struct TypeName(pub EQName);

pub fn attribute_name(input: &str) -> Res<&str, AttributeName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AttributeName

    eq_name(input).map(|(next_input, res)| (next_input, AttributeName(res)))
}

#[derive(PartialEq, Debug)]
pub struct AttributeName(pub EQName);

pub fn atomic_or_union_type(input: &str) -> Res<&str, AtomicOrUnionType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AtomicOrUnionType
    eq_name(input).map(|(next_input, res)| (next_input, AtomicOrUnionType(res)))
}
