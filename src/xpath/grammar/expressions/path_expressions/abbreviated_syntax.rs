//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#abbrev>

use std::fmt::Display;

use nom::{character::complete::char, combinator::opt, error::context};

use crate::xpath::grammar::{recipes::Res, whitespace_recipes::ws};

use super::steps::node_tests::{node_test, NodeTest};

pub fn abbrev_forward_step(input: &str) -> Res<&str, AbbrevForwardStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AbbrevForwardStep
    context("abbrev_forward_step", ws((opt(char('@')), node_test)))(input).map(
        |(next_input, res)| {
            (
                next_input,
                AbbrevForwardStep {
                    has_at: res.0.is_some(),
                    node_test: res.1,
                },
            )
        },
    )
}

#[derive(PartialEq, Debug, Clone)]
pub struct AbbrevForwardStep {
    pub has_at: bool,
    pub node_test: NodeTest,
}

impl Display for AbbrevForwardStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_at {
            write!(f, "@")?;
        }
        write!(f, "{}", self.node_test)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbrev_forward_step_step_should_parse() {
        // arrange
        let input = "@chapter";

        // act
        let (next_input, res) = abbrev_forward_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn abbrev_forward_step_step_should_parse_with_whitespace() {
        // arrange
        let input = "@ chapter";

        // act
        let (next_input, res) = abbrev_forward_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "@chapter");
    }
}
