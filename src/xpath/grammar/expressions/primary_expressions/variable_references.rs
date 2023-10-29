//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-variables

use std::fmt::Display;

use crate::xpath::grammar::{
    recipes::Res,
    types::{eq_name, EQName},
};

pub fn var_ref(input: &str) -> Res<&str, VarRef> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-VarRef
    var_name(input).map(|(next_input, res)| (next_input, VarRef(res)))
}

#[derive(PartialEq, Debug)]
pub struct VarRef(VarName);

impl Display for VarRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn var_name(input: &str) -> Res<&str, VarName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-VarName
    eq_name(input).map(|(next_input, res)| (next_input, VarName(res)))
}

#[derive(PartialEq, Debug)]
pub struct VarName(EQName);

impl Display for VarName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
