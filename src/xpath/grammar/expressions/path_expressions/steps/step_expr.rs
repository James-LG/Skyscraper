use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::{Node, XpathItem},
        expressions::{
            path_expressions::{
                abbreviated_syntax::abbrev_forward_step,
                steps::{
                    axes::{forward_axis::forward_axis, reverse_axis},
                    axis_step::axis_step,
                    node_tests::node_test,
                },
            },
            postfix_expressions::{postfix_expr, predicate, PostfixExpr, Predicate},
        },
        recipes::{max, Res},
    },
    ExpressionApplyError, XPathExpressionContext, XPathResult,
};

use super::axis_step::AxisStep;

pub fn step_expr(input: &str) -> Res<&str, StepExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-StepExpr

    fn postfix_expr_map(input: &str) -> Res<&str, StepExpr> {
        postfix_expr(input).map(|(next_input, res)| (next_input, StepExpr::PostfixExpr(res)))
    }

    fn axis_step_map(input: &str) -> Res<&str, StepExpr> {
        axis_step(input).map(|(next_input, res)| (next_input, StepExpr::AxisStep(res)))
    }

    context("step_expr", max((postfix_expr_map, axis_step_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
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

impl StepExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<Vec<XpathItem<'tree>>, ExpressionApplyError> {
        match self {
            StepExpr::PostfixExpr(expr) => expr.eval(context),
            StepExpr::AxisStep(step) => {
                let nodes = step.eval(context)?;
                let items = nodes.into_iter().map(|x| XpathItem::Node(x)).collect();
                Ok(items)
            }
        }
    }
}
