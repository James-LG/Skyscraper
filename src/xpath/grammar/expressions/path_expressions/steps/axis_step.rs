use std::fmt::Display;

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
        XpathItemTreeNode,
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

        // If there are no predicates, return expression result.
        if self.predicates.is_empty() {
            return Ok(nodes.into_iter().map(XpathItem::Node).collect());
        }

        // For reverse axes, context positions are assigned in reverse document
        // order (XPath 3.1 §3.3.2.2), so position 1 is the node closest to
        // the context node.
        let is_reverse = matches!(self.step_type, AxisStepType::ReverseStep(_));
        let nodes = if is_reverse {
            let mut sorted = nodes;
            sorted.sort_by(|a, b| {
                let a_id = a.node_id();
                let b_id = b.node_id();
                match (a_id, b_id) {
                    (Some(a), Some(b)) => b.cmp(&a), // reverse document order
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            });
            sorted
        } else {
            nodes
        };

        // Filter using predicates. Work with Vec directly to avoid hashing
        // all nodes into an intermediate XpathItemSet.
        let size = nodes.len();
        let mut filtered_items = XpathItemSet::new();
        for (i, &node) in nodes.iter().enumerate() {
            let predicate_context = context.new_with_item_and_size(
                XpathItem::Node(node),
                i + 1,
                size,
                context.is_initial_step,
            );
            let mut is_match = true;
            for predicate in self.predicates.iter() {
                if !predicate.is_match(&predicate_context)? {
                    is_match = false;
                    break;
                }
            }
            if is_match {
                filtered_items.insert(XpathItem::Node(node));
            }
        }

        // Restore document order for the final result.
        if is_reverse {
            filtered_items.sort_by_document_order();
            filtered_items.dedup();
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
    ) -> Result<Vec<&'tree XpathItemTreeNode>, ExpressionApplyError> {
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
