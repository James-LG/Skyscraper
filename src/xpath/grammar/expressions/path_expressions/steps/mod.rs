//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-steps

use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, multi::many0, sequence::tuple};

use crate::xpath::grammar::{
    expressions::{
        path_expressions::{
            abbreviated_syntax::abbrev_forward_step,
            steps::{
                axes::{forward_axis, reverse_axis},
                node_tests::node_test,
            },
        },
        postfix_expressions::{postfix_expr, predicate, PostfixExpr, Predicate},
    },
    recipes::{max, Res},
};

use self::{
    axes::{ForwardAxis, ReverseAxis},
    node_tests::NodeTest,
};

use super::abbreviated_syntax::AbbrevForwardStep;

pub mod axes;
pub mod node_tests;

pub fn step_expr(input: &str) -> Res<&str, StepExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-StepExpr

    fn postfix_expr_map(input: &str) -> Res<&str, StepExpr> {
        postfix_expr(input).map(|(next_input, res)| (next_input, StepExpr::PostfixExpr(res)))
    }

    fn axis_step_map(input: &str) -> Res<&str, StepExpr> {
        axis_step(input).map(|(next_input, res)| (next_input, StepExpr::AxisStep(res)))
    }

    max((postfix_expr_map, axis_step_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum StepExpr {
    PostfixExpr(PostfixExpr),
    AxisStep(AxisStep),
}

impl Display for StepExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepExpr::PostfixExpr(x) => write!(f, "{}", x),
            StepExpr::AxisStep(x) => write!(f, "{}", x),
        }
    }
}

fn axis_step(input: &str) -> Res<&str, AxisStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AxisStep

    fn reverse_step_map(input: &str) -> Res<&str, AxisStepType> {
        reverse_step(input).map(|(next_input, res)| (next_input, AxisStepType::ReverseStep(res)))
    }

    fn forward_step_map(input: &str) -> Res<&str, AxisStepType> {
        forward_step(input).map(|(next_input, res)| (next_input, AxisStepType::ForwardStep(res)))
    }

    tuple((alt((reverse_step_map, forward_step_map)), predicate_list))(input).map(
        |(next_input, res)| {
            (
                next_input,
                AxisStep {
                    step_type: res.0,
                    predicates: res.1,
                },
            )
        },
    )
}

#[derive(PartialEq, Debug)]
pub struct AxisStep {
    pub step_type: AxisStepType,
    pub predicates: Vec<Predicate>,
}

impl Display for AxisStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.step_type)?;
        for x in &self.predicates {
            write!(f, "{}", x)?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub enum AxisStepType {
    ReverseStep(ReverseStep),
    ForwardStep(ForwardStep),
}

impl Display for AxisStepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AxisStepType::ReverseStep(x) => write!(f, "{}", x),
            AxisStepType::ForwardStep(x) => write!(f, "{}", x),
        }
    }
}

fn forward_step(input: &str) -> Res<&str, ForwardStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ForwardStep

    fn full_forward_step(input: &str) -> Res<&str, ForwardStep> {
        tuple((forward_axis, node_test))(input)
            .map(|(next_input, res)| (next_input, ForwardStep::Full(res.0, res.1)))
    }

    fn abbrev_forward_step_map(input: &str) -> Res<&str, ForwardStep> {
        abbrev_forward_step(input)
            .map(|(next_input, res)| (next_input, ForwardStep::Abbreviated(res)))
    }

    alt((full_forward_step, abbrev_forward_step_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum ForwardStep {
    Full(ForwardAxis, NodeTest),
    Abbreviated(AbbrevForwardStep),
}

impl Display for ForwardStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForwardStep::Full(x, y) => write!(f, "{}{}", x, y),
            ForwardStep::Abbreviated(x) => write!(f, "{}", x),
        }
    }
}

fn reverse_step(input: &str) -> Res<&str, ReverseStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ReverseStep
    fn full_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        tuple((reverse_axis, node_test))(input)
            .map(|(next_input, res)| (next_input, ReverseStep::Full(res.0, res.1)))
    }

    fn abbrev_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AbbrevReverseStep
        tag("..")(input).map(|(next_input, _res)| (next_input, ReverseStep::Abbreviated))
    }

    alt((full_reverse_step, abbrev_reverse_step))(input)
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

fn predicate_list(input: &str) -> Res<&str, Vec<Predicate>> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PredicateList

    many0(predicate)(input)
}
