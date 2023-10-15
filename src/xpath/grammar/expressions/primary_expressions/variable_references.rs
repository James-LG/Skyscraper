//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-variables

use crate::xpath::grammar::{
    recipes::Res,
    types::{eq_name, EQName},
};

pub fn var_ref(input: &str) -> Res<&str, VarRef> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-VarRef
    var_name(input).map(|(next_input, res)| (next_input, VarRef(res)))
}

pub struct VarRef(VarName);

pub fn var_name(input: &str) -> Res<&str, VarName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-VarName
    eq_name(input).map(|(next_input, res)| (next_input, VarName(res)))
}

pub struct VarName(EQName);
