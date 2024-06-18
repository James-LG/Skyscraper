//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-variables>

use std::fmt::Display;

use nom::{character::complete::char, error::context};

use crate::xpath::grammar::{
    recipes::Res,
    types::{eq_name, EQName},
    whitespace_recipes::ws,
};

pub fn var_ref(input: &str) -> Res<&str, VarRef> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-VarRef
    context("var_ref", ws((char('$'), var_name)))(input)
        .map(|(next_input, res)| (next_input, VarRef(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct VarRef(VarName);

impl Display for VarRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

pub fn var_name(input: &str) -> Res<&str, VarName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-VarName
    context("var_name", eq_name)(input).map(|(next_input, res)| (next_input, VarName(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct VarName(EQName);

impl Display for VarName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn var_ref_should_parse() {
        // arrange
        let input = "$products";

        // act
        let (next_input, res) = var_ref(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "$products");
    }

    #[test]
    fn var_ref_should_parse_whitespace() {
        // arrange
        let input = "$ products";

        // act
        let (next_input, res) = var_ref(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "$products");
    }
}
