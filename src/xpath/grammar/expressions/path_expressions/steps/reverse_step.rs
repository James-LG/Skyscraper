use std::fmt::Display;

use indexmap::IndexSet;
use nom::{branch::alt, bytes::complete::tag, error::context};

use crate::xpath::{
    grammar::{
        data_model::XpathItem,
        expressions::path_expressions::steps::{
            axes::reverse_axis::reverse_axis, node_tests::node_test,
        },
        recipes::Res,
        types::KindTest,
        whitespace_recipes::ws,
        XpathItemTreeNode,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::{
    axes::reverse_axis::ReverseAxis,
    collect_self_and_descendants,
    node_tests::{BiDirectionalAxis, NodeTest},
};

pub fn reverse_step(input: &str) -> Res<&str, ReverseStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ReverseStep
    fn full_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        ws((reverse_axis, node_test))(input)
            .map(|(next_input, res)| (next_input, ReverseStep::Full(res.0, res.1)))
    }

    fn abbrev_reverse_step(input: &str) -> Res<&str, ReverseStep> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AbbrevReverseStep
        ws((tag(".."),))(input).map(|(next_input, _res)| (next_input, ReverseStep::Abbreviated))
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
    ) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
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
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let axis_nodes: IndexSet<&'tree XpathItemTreeNode> = match axis {
        ReverseAxis::Parent => eval_reverse_axis_parent(context),
        ReverseAxis::Ancestor => eval_reverse_axis_ancestor(context),
        ReverseAxis::PrecedingSibling => eval_reverse_axis_preceding_sibling(context),
        ReverseAxis::Preceding => eval_reverse_axis_preceding(context),
        ReverseAxis::AncestorOrSelf => eval_reverse_axis_ancestor_or_self(context),
    }?;

    let items: XpathItemSet<'tree> = axis_nodes.into_iter().map(XpathItem::Node).collect();
    let mut nodes = IndexSet::new();

    for (i, _node) in items.iter().enumerate() {
        let node_test_context =
            context.new_with_variables(&items, i + 1, context.is_root_level);

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
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    // Only tree items have parents
    // TODO: Technically an attribute's parent is an element, but there is no link to that ATM.
    if let XpathItem::Node(node) = &context.item {
        if let Some(parent) = &node.parent(context.item_tree) {
            nodes.insert(*parent);
        }
    }

    Ok(nodes)
}

/// All ancestors of the context node (parent, grandparent, ...) up to the root, in reverse document order.
fn eval_reverse_axis_ancestor<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        if let Some(node_id) = node.node_id() {
            let mut current = context.item_tree.arena.get(node_id).and_then(|n| n.parent());
            while let Some(ancestor_id) = current {
                nodes.insert(context.item_tree.get(ancestor_id));
                current = context.item_tree.arena.get(ancestor_id).and_then(|n| n.parent());
            }
        }
    }

    Ok(nodes)
}

/// The context node and all its ancestors, in reverse document order.
fn eval_reverse_axis_ancestor_or_self<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        nodes.insert(*node);

        if let Some(node_id) = node.node_id() {
            let mut current = context.item_tree.arena.get(node_id).and_then(|n| n.parent());
            while let Some(ancestor_id) = current {
                nodes.insert(context.item_tree.get(ancestor_id));
                current = context.item_tree.arena.get(ancestor_id).and_then(|n| n.parent());
            }
        }
    } else {
        return Err(ExpressionApplyError {
            msg: String::from("err:XPTY0020 context item for axis step is not a node"),
        });
    }

    Ok(nodes)
}

/// All siblings of the context node that come before it in document order.
fn eval_reverse_axis_preceding_sibling<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        if let Some(node_id) = node.node_id() {
            let mut prev =
                context.item_tree.arena.get(node_id).and_then(|n| n.previous_sibling());
            while let Some(sibling_id) = prev {
                nodes.insert(context.item_tree.get(sibling_id));
                prev = context
                    .item_tree
                    .arena
                    .get(sibling_id)
                    .and_then(|n| n.previous_sibling());
            }
        }
    }

    Ok(nodes)
}

/// All nodes that come before the context node in document order, excluding ancestors.
fn eval_reverse_axis_preceding<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        if let Some(node_id) = node.node_id() {
            // Collect all preceding nodes: for each ancestor (including self),
            // take all preceding siblings and their descendants.
            let mut current = Some(node_id);
            while let Some(cur_id) = current {
                let mut prev = context
                    .item_tree
                    .arena
                    .get(cur_id)
                    .and_then(|n| n.previous_sibling());
                while let Some(sibling_id) = prev {
                    collect_self_and_descendants(context, sibling_id, &mut nodes);
                    prev = context
                        .item_tree
                        .arena
                        .get(sibling_id)
                        .and_then(|n| n.previous_sibling());
                }
                // Move up to parent.
                current = context.item_tree.arena.get(cur_id).and_then(|n| n.parent());
            }
        }
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_step_should_parse_abbrev() {
        // arrange
        let input = "..";

        // act
        let (next_input, res) = reverse_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn reverse_step_should_parse_full() {
        // arrange
        let input = "parent::*";

        // act
        let (next_input, res) = reverse_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn reverse_step_should_parse_full_whitespace() {
        // arrange
        let input = "parent:: *";

        // act
        let (next_input, res) = reverse_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "parent::*");
    }
}
