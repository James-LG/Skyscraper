use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple, Err};

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
        NonTreeXpathNode, XpathItemTreeNodeData,
    },
    Expression, ExpressionApplyError, XPathExpressionContext, XPathResult, XpathItemTree,
    XpathItemTreeNode,
};

use super::{
    axes::forward_axis::ForwardAxis,
    node_tests::{BiDirectionalAxis, NodeTest},
};

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
    let axis_nodes = match axis {
        ForwardAxis::Child => eval_forward_axis_child(context),
        ForwardAxis::Descendant => eval_forward_axis_descendant(context),
        ForwardAxis::Attribute => eval_forward_axis_attribute(context),
        ForwardAxis::SelfAxis => todo!("eval_forward_axis ForwardAxis::SelfAxis"),
        ForwardAxis::DescendantOrSelf => eval_forward_axis_self_or_descendant(context),
        ForwardAxis::FollowingSibling => todo!("eval_forward_axis ForwardAxis::FollowingSibling"),
        ForwardAxis::Following => todo!("eval_forward_axis ForwardAxis::Following"),
        ForwardAxis::Namespace => todo!("eval_forward_axis ForwardAxis::Namespace"),
    }?;

    let items: Vec<XpathItem<'tree>> = axis_nodes.into_iter().map(XpathItem::Node).collect();
    let mut nodes = Vec::new();

    for (i, item) in items.iter().enumerate() {
        let node_test_context = XPathExpressionContext::new(context.item_tree, &items, i + 1);

        let result = node_test.eval(BiDirectionalAxis::ForwardAxis(axis), &node_test_context)?;
        nodes.extend(result);
    }

    Ok(nodes)
}

/// Direct children of the context nodes.
fn eval_forward_axis_child<'tree>(
    context: &XPathExpressionContext<'tree>,
) -> Result<Vec<Node<'tree>>, ExpressionApplyError> {
    let mut nodes: Vec<Node<'tree>> = Vec::new();

    // Only tree nodes have children
    if let XpathItem::Node(Node::TreeNode(node)) = &context.item {
        for child in node.children(context.item_tree) {
            let child = Node::TreeNode(child);
            nodes.push(child);
        }
    }

    Ok(nodes)
}

/// All descendants of the context nodes.
fn eval_forward_axis_descendant<'tree>(
    context: &XPathExpressionContext<'tree>,
) -> Result<Vec<Node<'tree>>, ExpressionApplyError> {
    let mut nodes: Vec<Node<'tree>> = Vec::new();

    // Only tree nodes have children.
    if let XpathItem::Node(Node::TreeNode(node)) = &context.item {
        for child in node.children(context.item_tree) {
            // Add the child.
            let child = Node::TreeNode(child);
            nodes.push(child.clone());

            // Add the child's descendants.
            let child_eval_context =
                XPathExpressionContext::new_single(context.item_tree, child.into());
            let child_descendants = eval_forward_axis_descendant(&child_eval_context)?;
            nodes.extend(child_descendants);
        }
    }

    Ok(nodes)
}

/// All descendants of the context nodes including the context nodes.
fn eval_forward_axis_self_or_descendant<'tree>(
    context: &XPathExpressionContext<'tree>,
) -> Result<Vec<Node<'tree>>, ExpressionApplyError> {
    let mut nodes = eval_forward_axis_descendant(context)?;

    if let XpathItem::Node(node) = &context.item {
        nodes.push(node.clone());
    } else {
        return Err(ExpressionApplyError {
            msg: String::from("err:XPTY0020 context item for axis step is not a node"),
        });
    }

    Ok(nodes)
}

// All attributes of the context nodes.
fn eval_forward_axis_attribute<'tree>(
    context: &XPathExpressionContext<'tree>,
) -> Result<Vec<Node<'tree>>, ExpressionApplyError> {
    let mut attributes = Vec::new();

    // Only elements have attributes.
    if let XpathItem::Node(Node::TreeNode(XpathItemTreeNode {
        data: XpathItemTreeNodeData::ElementNode(element),
        ..
    })) = context.item
    {
        for attribute in element.attributes.iter() {
            let attribute = Node::NonTreeNode(NonTreeXpathNode::AttributeNode(attribute.clone()));

            attributes.push(attribute);
        }
    }

    Ok(attributes)
}
