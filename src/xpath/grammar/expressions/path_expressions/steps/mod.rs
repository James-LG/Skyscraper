//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-steps

use nom::{error::context, multi::many0};

use crate::xpath::grammar::{
    expressions::postfix_expressions::{predicate, Predicate},
    recipes::Res,
};

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
