//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-steps

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

use self::{
    axes::{forward_axis::ForwardAxis, reverse_axis::ReverseAxis},
    node_tests::NodeTest,
};

use super::abbreviated_syntax::AbbrevForwardStep;

pub mod axes;
pub mod axis_step;
pub mod forward_step;
pub mod node_tests;
pub mod reverse_step;
pub mod step_expr;

fn predicate_list(input: &str) -> Res<&str, Vec<Predicate>> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PredicateList

    context("predicate_list", many0(predicate))(input)
}
