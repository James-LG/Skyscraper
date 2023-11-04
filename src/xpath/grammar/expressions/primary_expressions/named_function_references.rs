//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-named-function-ref

use nom::{character::complete::char, error::context, sequence::tuple};

use crate::xpath::grammar::{
    recipes::Res,
    terminal_symbols::integer_literal,
    types::{eq_name, EQName},
};

pub fn named_function_ref(input: &str) -> Res<&str, NamedFunctionRef> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NamedFunctionRef

    context(
        "named_function_ref",
        tuple((eq_name, char('#'), integer_literal)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            NamedFunctionRef {
                name: res.0,
                number: res.2,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct NamedFunctionRef {
    pub name: EQName,
    pub number: u32,
}
