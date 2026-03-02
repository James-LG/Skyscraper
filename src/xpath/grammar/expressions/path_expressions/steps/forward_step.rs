use std::fmt::Display;

use indexmap::IndexSet;
use nom::{branch::alt, error::context};

use indextree::NodeId;

use crate::xpath::{
    grammar::{
        data_model::XpathItem,
        expressions::path_expressions::{
            abbreviated_syntax::{abbrev_forward_step, AbbrevForwardStep},
            steps::{axes::forward_axis::forward_axis, node_tests::node_test},
        },
        recipes::Res,
        whitespace_recipes::ws,
        XpathItemTreeNode,
    },
    ExpressionApplyError, XpathExpressionContext,
};

use super::{
    axes::forward_axis::ForwardAxis,
    collect_self_and_descendants,
    node_tests::{BiDirectionalAxis, NodeTest},
};

pub fn forward_step(input: &str) -> Res<&str, ForwardStep> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ForwardStep

    fn full_forward_step(input: &str) -> Res<&str, ForwardStep> {
        ws((forward_axis, node_test))(input)
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
        context: &XpathExpressionContext<'tree>,
    ) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
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
    context: &XpathExpressionContext<'tree>,
    axis: ForwardAxis,
    node_test: &NodeTest,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let axis_nodes = match axis {
        ForwardAxis::Child => eval_forward_axis_child(context),
        ForwardAxis::Descendant => eval_forward_axis_descendant(context),
        ForwardAxis::Attribute => eval_forward_axis_attribute(context),
        ForwardAxis::SelfAxis => eval_forward_axis_self(context),
        ForwardAxis::DescendantOrSelf => eval_forward_axis_self_or_descendant(context),
        ForwardAxis::FollowingSibling => eval_forward_axis_following_sibling(context),
        ForwardAxis::Following => eval_forward_axis_following(context),
        ForwardAxis::Namespace => {
            return Err(ExpressionApplyError {
                msg: String::from("namespace:: axis is not supported for HTML documents"),
            })
        }
    }?;

    // Filter axis nodes using the node test directly — no context creation needed.
    let bi_axis = BiDirectionalAxis::ForwardAxis(axis);
    let mut nodes = IndexSet::new();
    for node in axis_nodes {
        if node_test.matches_node(bi_axis, node, context.item_tree)? {
            nodes.insert(node);
        }
    }

    Ok(nodes)
}

/// Get the [`NodeId`] for a tree node, falling back to the tree root for `DocumentNode`
/// (which doesn't store its own id).
fn node_or_root_id(node: &XpathItemTreeNode, context: &XpathExpressionContext<'_>) -> NodeId {
    match node.node_id() {
        Some(id) => id,
        None => context.item_tree.root_node, // DocumentNode
    }
}

/// Direct children of the context node.
fn eval_forward_axis_child<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    // Only tree nodes have children
    if let XpathItem::Node(node) = &context.item {
        for child in node.children(context.item_tree) {
            nodes.insert(child);
        }
    }

    Ok(nodes)
}

/// All descendants of the context node.
fn eval_forward_axis_descendant<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    // Only tree nodes have children.
    if let XpathItem::Node(node) = &context.item {
        let node_id = node_or_root_id(node, context);

        // Use indextree's built-in descendants iterator instead of manual recursion.
        // skip(1) to exclude self — descendants() includes the node itself.
        for descendant_id in node_id.descendants(&context.item_tree.arena).skip(1) {
            nodes.insert(context.item_tree.get(descendant_id));
        }
    }

    Ok(nodes)
}

/// All descendants of the context node, including the context node itself.
fn eval_forward_axis_self_or_descendant<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        let node_id = node_or_root_id(node, context);

        // Use indextree's built-in descendants iterator — includes self.
        for descendant_id in node_id.descendants(&context.item_tree.arena) {
            nodes.insert(context.item_tree.get(descendant_id));
        }
    } else {
        return Err(ExpressionApplyError {
            msg: String::from("err:XPTY0020 context item for axis step is not a node"),
        });
    }

    Ok(nodes)
}

/// The context node itself.
fn eval_forward_axis_self<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        nodes.insert(*node);
    } else {
        return Err(ExpressionApplyError {
            msg: String::from("err:XPTY0020 context item for axis step is not a node"),
        });
    }

    Ok(nodes)
}

/// All siblings of the context node that come after it in document order.
fn eval_forward_axis_following_sibling<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        if let Some(node_id) = node.node_id() {
            let mut next = context.item_tree.arena.get(node_id).and_then(|n| n.next_sibling());
            while let Some(sibling_id) = next {
                nodes.insert(context.item_tree.get(sibling_id));
                next = context.item_tree.arena.get(sibling_id).and_then(|n| n.next_sibling());
            }
        }
    }

    Ok(nodes)
}

/// All nodes that come after the context node in document order, excluding descendants.
fn eval_forward_axis_following<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut nodes = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        if let Some(node_id) = node.node_id() {
            // Collect all following nodes: for each ancestor (including self),
            // take all following siblings and their descendants.
            let mut current = Some(node_id);
            while let Some(cur_id) = current {
                // Add all following siblings of current and their descendants.
                let mut next = context.item_tree.arena.get(cur_id).and_then(|n| n.next_sibling());
                while let Some(sibling_id) = next {
                    collect_self_and_descendants(context, sibling_id, &mut nodes);
                    next = context.item_tree.arena.get(sibling_id).and_then(|n| n.next_sibling());
                }
                // Move up to parent.
                current = context.item_tree.arena.get(cur_id).and_then(|n| n.parent());
            }
        }
    }

    Ok(nodes)
}

// All attributes of the context nodes.
fn eval_forward_axis_attribute<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let mut attributes: IndexSet<&'tree XpathItemTreeNode> = IndexSet::new();

    // Only elements have attributes.
    if let XpathItem::Node(XpathItemTreeNode::ElementNode(element)) = context.item {
        for child in element.children(context.item_tree) {
            if let XpathItemTreeNode::AttributeNode(_attribute) = &child {
                attributes.insert(child);
            }
        }
    }

    Ok(attributes)
}

#[cfg(test)]
mod tests {
    use crate::xpath::grammar::types::KindTest;

    use super::*;

    #[test]
    fn forward_step_should_parse_abbrev() {
        // arrange
        let input = "@class";

        // act
        let (next_input, res) = forward_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn forward_step_should_parse_full() {
        // arrange
        let input = "child::*";

        // act
        let (next_input, res) = forward_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn forward_step_should_parse_full_whitespace() {
        // arrange
        let input = "child:: *";

        // act
        let (next_input, res) = forward_step(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "child::*");
    }

    /// `text()` could be matched by a function call or a node test. It should be a node test.
    #[test]
    fn forward_step_should_use_text_test_not_function_call() {
        // arrange
        let text = "text()";

        // act
        let xpath = forward_step(text).unwrap();

        // assert
        assert_eq!(
            xpath,
            (
                "",
                ForwardStep::Abbreviated(AbbrevForwardStep {
                    has_at: false,
                    node_test: NodeTest::KindTest(KindTest::TextTest)
                })
            )
        );
    }
}
