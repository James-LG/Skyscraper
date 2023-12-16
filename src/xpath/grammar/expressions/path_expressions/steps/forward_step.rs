use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::{Node, XpathItem},
        expressions::{
            path_expressions::{
                abbreviated_syntax::{abbrev_forward_step, AbbrevForwardStep},
                steps::{
                    axes::{forward_axis::forward_axis, reverse_axis::reverse_axis},
                    node_tests::node_test,
                },
            },
            postfix_expressions::{postfix_expr, predicate, PostfixExpr, Predicate},
        },
        recipes::{max, Res},
    },
    Expression, ExpressionApplyError, XPathExpressionContext, XPathResult, XpathItemTree,
    XpathItemTreeNode,
};

use super::{axes::forward_axis::ForwardAxis, node_tests::NodeTest};

pub fn forward_step(input: &str) -> Res<&str, ForwardStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ForwardStep

    fn full_forward_step(input: &str) -> Res<&str, ForwardStep> {
        tuple((forward_axis, node_test))(input)
            .map(|(next_input, res)| (next_input, ForwardStep::Full(res.0, res.1)))
    }

    fn abbrev_forward_step_map(input: &str) -> Res<&str, ForwardStep> {
        abbrev_forward_step(input)
            .map(|(next_input, res)| (next_input, ForwardStep::Abbreviated(res)))
    }

    context(
        "forward_step",
        alt((full_forward_step, abbrev_forward_step_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
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

impl ForwardStep {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<Vec<Node<'tree>>, ExpressionApplyError> {
        match self {
            ForwardStep::Full(axis, node_test) => eval_forward_axis(context, *axis, node_test),
            ForwardStep::Abbreviated(step) => {
                // Abbreviated forward step axis is attribute if it has @, otherwise it's child
                let axis = if step.has_at {
                    ForwardAxis::Attribute
                } else {
                    ForwardAxis::Child
                };

                eval_forward_axis(context, axis, &step.node_test)
            }
        }
    }
}

fn eval_forward_axis<'tree>(
    context: &XPathExpressionContext<'tree>,
    axis: ForwardAxis,
    node_test: &NodeTest,
) -> Result<Vec<Node<'tree>>, ExpressionApplyError> {
    match axis {
        ForwardAxis::Child => {
            let mut nodes: Vec<Node<'tree>> = Vec::new();

            for node in context.searchable_nodes.iter() {
                // Only elements have children
                if let Node::TreeNode(node) = node {
                    for child in node.children(context.item_tree) {
                        if node_test.matches(&child.data) {
                            nodes.push(Node::TreeNode(child));
                        }
                    }
                }
            }

            Ok(nodes)
        }
        ForwardAxis::Descendant => todo!(),
        ForwardAxis::Attribute => todo!(),
        ForwardAxis::SelfAxis => todo!(),
        ForwardAxis::DescendantOrSelf => todo!(),
        ForwardAxis::FollowingSibling => todo!(),
        ForwardAxis::Following => todo!(),
        ForwardAxis::Namespace => todo!(),
    }
}
