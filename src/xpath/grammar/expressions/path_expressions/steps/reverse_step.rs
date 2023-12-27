use std::fmt::Display;

use indexmap::IndexSet;
use nom::{branch::alt, bytes::complete::tag, error::context, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::{Node, XpathItem},
        expressions::path_expressions::steps::{
            axes::reverse_axis::reverse_axis, node_tests::node_test,
        },
        recipes::Res,
        types::KindTest,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::{
    axes::reverse_axis::ReverseAxis,
    node_tests::{BiDirectionalAxis, NodeTest},
};

pub fn reverse_step(input: &str) -> Res<&str, ReverseStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ReverseStep
    fn full_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        tuple((reverse_axis, node_test))(input)
            .map(|(next_input, res)| (next_input, ReverseStep::Full(res.0, res.1)))
    }

    fn abbrev_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AbbrevReverseStep
        tag("..")(input).map(|(next_input, _res)| (next_input, ReverseStep::Abbreviated))
    }

    context(
        "reverse_step",
        alt((full_reverse_step, abbrev_reverse_step)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ReverseStep {
    Full(ReverseAxis, NodeTest),
    Abbreviated,
}

impl Display for ReverseStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReverseStep::Full(x, y) => write!(f, "{}{}", x, y),
            ReverseStep::Abbreviated => write!(f, ".."),
        }
    }
}

impl ReverseStep {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<IndexSet<Node<'tree>>, ExpressionApplyError> {
        match self {
            ReverseStep::Full(axis, node_test) => eval_reverse_axis(context, *axis, node_test),
            ReverseStep::Abbreviated => {
                // `..` is short for `parent::node()`.
                eval_reverse_axis(
                    context,
                    ReverseAxis::Parent,
                    &NodeTest::KindTest(KindTest::AnyKindTest),
                )
            }
        }
    }
}

fn eval_reverse_axis<'tree>(
    context: &XpathExpressionContext<'tree>,
    axis: ReverseAxis,
    node_test: &NodeTest,
) -> Result<IndexSet<Node<'tree>>, ExpressionApplyError> {
    let axis_nodes: IndexSet<Node> = match axis {
        ReverseAxis::Parent => eval_reverse_axis_parent(context),
        ReverseAxis::Ancestor => todo!("eval_reverse_axis ReverseAxis::Ancestor"),
        ReverseAxis::PrecedingSibling => todo!("eval_reverse_axis ReverseAxis::PrecedingSibling"),
        ReverseAxis::Preceding => todo!("eval_reverse_axis ReverseAxis::Preceding"),
        ReverseAxis::AncestorOrSelf => todo!("eval_reverse_axis ReverseAxis::AncestorOrSelf"),
    }?;

    let items: XpathItemSet<'tree> = axis_nodes.into_iter().map(XpathItem::Node).collect();
    let mut nodes = IndexSet::new();

    for (i, _node) in items.iter().enumerate() {
        let node_test_context = XpathExpressionContext::new(context.item_tree, &items, i + 1);

        if let Some(result) =
            node_test.eval(BiDirectionalAxis::ReverseAxis(axis), &node_test_context)?
        {
            nodes.insert(result);
        }
    }

    Ok(nodes)
}

/// Direct parent of the context node.
fn eval_reverse_axis_parent<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<Node<'tree>>, ExpressionApplyError> {
    let mut nodes: IndexSet<Node<'tree>> = IndexSet::new();

    // Only tree items have parents
    // TODO: Technically an attribute's parent is an element, but there is no link to that ATM.
    if let XpathItem::Node(Node::TreeNode(node)) = &context.item {
        if let Some(parent) = &node.parent(context.item_tree) {
            nodes.insert(Node::TreeNode(parent.clone()));
        }
    }

    Ok(nodes)
}
