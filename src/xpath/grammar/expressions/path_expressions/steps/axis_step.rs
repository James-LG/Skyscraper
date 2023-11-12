use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::grammar::{
    expressions::{
        path_expressions::{
            abbreviated_syntax::abbrev_forward_step,
            steps::{
                axes::{forward_axis::forward_axis, reverse_axis},
                forward_step::forward_step,
                node_tests::node_test,
                predicate_list,
                reverse_step::{reverse_step, ReverseStep},
            },
        },
        postfix_expressions::{postfix_expr, predicate, PostfixExpr, Predicate},
    },
    recipes::{max, Res},
};

use super::forward_step::ForwardStep;

pub fn axis_step(input: &str) -> Res<&str, AxisStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AxisStep

    fn reverse_step_map(input: &str) -> Res<&str, AxisStepType> {
        reverse_step(input).map(|(next_input, res)| (next_input, AxisStepType::ReverseStep(res)))
    }

    fn forward_step_map(input: &str) -> Res<&str, AxisStepType> {
        forward_step(input).map(|(next_input, res)| (next_input, AxisStepType::ForwardStep(res)))
    }

    context(
        "axis_step",
        tuple((alt((reverse_step_map, forward_step_map)), predicate_list)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            AxisStep {
                step_type: res.0,
                predicates: res.1,
            },
        )
    })
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
