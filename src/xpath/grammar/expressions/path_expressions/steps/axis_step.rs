use std::fmt::Display;

use indexmap::IndexSet;
use nom::{branch::alt, error::context, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::XpathItem,
        expressions::{
            path_expressions::steps::{
                forward_step::forward_step,
                predicate_list,
                reverse_step::{reverse_step, ReverseStep},
            },
            postfix_expressions::Predicate,
        },
        recipes::Res,
        XpathItemTreeNodeData,
    },
    ExpressionApplyError, XpathExpressionContext, XpathItemSet,
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

#[derive(PartialEq, Debug, Clone)]
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

impl AxisStep {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let nodes = self.step_type.eval(context)?;
        let items: XpathItemSet<'tree> = nodes.into_iter().map(XpathItem::Node).collect();

        // If there are no predicates, return expression result.
        if self.predicates.is_empty() {
            return Ok(items);
        }

        // Otherwise, filter using predicates.
        let mut filtered_items = XpathItemSet::new();
        for (i, item) in items.iter().enumerate() {
            // All predicates must match for a node to be selected.
            let mut is_match = true;

            let predicate_context = XpathExpressionContext::new(
                context.item_tree,
                &items,
                i + 1,
                context.is_root_level,
            );
            for predicate in self.predicates.iter() {
                if !predicate.is_match(&predicate_context)? {
                    is_match = false;
                }
            }

            if is_match {
                filtered_items.insert(item.clone());
            }
        }

        Ok(filtered_items)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum AxisStepType {
    ReverseStep(ReverseStep),
    ForwardStep(ForwardStep),
}

impl AxisStepType {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<IndexSet<&'tree XpathItemTreeNodeData>, ExpressionApplyError> {
        match self {
            AxisStepType::ReverseStep(step) => step.eval(context),
            AxisStepType::ForwardStep(step) => step.eval(context),
        }
    }
}

impl Display for AxisStepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AxisStepType::ReverseStep(x) => write!(f, "{}", x),
            AxisStepType::ForwardStep(x) => write!(f, "{}", x),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::xpath::grammar::{
        expressions::path_expressions::{
            abbreviated_syntax::AbbrevForwardStep, steps::node_tests::NodeTest,
        },
        types::KindTest,
    };

    use super::*;

    #[test]
    fn axis_step_should_parse() {
        // arrange
        let input = "child::chapter[2]";

        // act
        let (next_input, res) = axis_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn axis_step_should_parse_with_whitespace() {
        // arrange
        let input = "child::chapter [ 2 ]";

        // act
        let (next_input, res) = axis_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "child::chapter[2]");
    }

    /// `text()` could be matched by a function call or a node test. It should be a node test.
    #[test]
    fn axis_step_should_use_text_test_not_function_call() {
        // arrange
        let text = "text()";

        // act
        let xpath = axis_step(text).unwrap();

        // assert
        assert_eq!(
            xpath,
            (
                "",
                AxisStep {
                    step_type: AxisStepType::ForwardStep(ForwardStep::Abbreviated(
                        AbbrevForwardStep {
                            has_at: false,
                            node_test: NodeTest::KindTest(KindTest::TextTest)
                        }
                    )),
                    predicates: vec![]
                }
            )
        );
    }
}
