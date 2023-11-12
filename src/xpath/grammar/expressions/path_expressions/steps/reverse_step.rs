use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::grammar::{
    expressions::{
        path_expressions::{
            abbreviated_syntax::abbrev_forward_step,
            steps::{
                axes::{forward_axis::forward_axis, reverse_axis::reverse_axis},
                node_tests::node_test,
            },
        },
        postfix_expressions::{postfix_expr, predicate, PostfixExpr, Predicate},
    },
    recipes::{max, Res},
};

use super::{axes::reverse_axis::ReverseAxis, node_tests::NodeTest};

pub fn reverse_step(input: &str) -> Res<&str, ReverseStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ReverseStep
    fn full_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        tuple((reverse_axis, node_test))(input)
            .map(|(next_input, res)| (next_input, ReverseStep::Full(res.0, res.1)))
    }

    fn abbrev_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AbbrevReverseStep
        tag("..")(input).map(|(next_input, _res)| (next_input, ReverseStep::Abbreviated))
    }

    context(
        "reverse_step",
        alt((full_reverse_step, abbrev_reverse_step)),
    )(input)
}

#[derive(PartialEq, Debug)]
pub enum ReverseStep {
    Full(ReverseAxis, NodeTest),
    Abbreviated,
}

impl Display for ReverseStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReverseStep::Full(x, y) => write!(f, "{} {}", x, y),
            ReverseStep::Abbreviated => write!(f, ".."),
        }
    }
}
