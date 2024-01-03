//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-named-function-ref

use std::fmt::Display;

use nom::{character::complete::char, error::context};

use crate::xpath::grammar::{
    recipes::Res,
    terminal_symbols::integer_literal,
    types::{eq_name, EQName},
    whitespace_recipes::ws,
};

pub fn named_function_ref(input: &str) -> Res<&str, NamedFunctionRef> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NamedFunctionRef

    context(
        "named_function_ref",
        ws((eq_name, char('#'), integer_literal)),
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

#[derive(PartialEq, Debug, Clone)]
pub struct NamedFunctionRef {
    pub name: EQName,
    pub number: u32,
}

impl Display for NamedFunctionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.name, self.number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_function_ref_should_parse() {
        // arrange
        let input = "fn:abs#1";

        // act
        let (next_input, res) = named_function_ref(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "fn:abs#1");
    }

    #[test]
    fn named_function_ref_should_parse_whitespace() {
        // arrange
        let input = "fn:abs # 1";

        // act
        let (next_input, res) = named_function_ref(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "fn:abs#1");
    }
}
