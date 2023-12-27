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
    ExpressionApplyError, XpathExpressionContext,
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

    context("step_expr", max((axis_step_map, postfix_expr_map)))(input)
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
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        match self {
            StepExpr::PostfixExpr(expr) => expr.eval(context),
            StepExpr::AxisStep(step) => step.eval(context),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::xpath::grammar::{
        expressions::path_expressions::{
            abbreviated_syntax::AbbrevForwardStep,
            steps::{axis_step::AxisStepType, forward_step::ForwardStep, node_tests::NodeTest},
        },
        types::KindTest,
    };

    use super::*;

    /// `text()` could be matched by a function call or a node test. It should be a node test.
    #[test]
    fn step_expr_should_use_text_test_not_function_call() {
        // arrange
        let text = "text()";

        // act
        let xpath = step_expr(text).unwrap();

        // assert
        assert_eq!(
            xpath,
            (
                "",
                StepExpr::AxisStep(AxisStep {
                    step_type: AxisStepType::ForwardStep(ForwardStep::Abbreviated(
                        AbbrevForwardStep {
                            has_at: false,
                            node_test: NodeTest::KindTest(KindTest::TextTest)
                        }
                    )),
                    predicates: vec![]
                })
            )
        );
    }
}
