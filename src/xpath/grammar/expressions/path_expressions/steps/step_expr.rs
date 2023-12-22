use std::fmt::Display;

use nom::error::context;

use crate::xpath::{
    grammar::{
        expressions::{
            path_expressions::steps::axis_step::axis_step,
            postfix_expressions::{postfix_expr, PostfixExpr},
        },
        recipes::{max, Res},
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XPathExpressionContext,
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
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        match self {
            StepExpr::PostfixExpr(expr) => expr.eval(context),
            StepExpr::AxisStep(step) => step.eval(context),
        }
    }
}
