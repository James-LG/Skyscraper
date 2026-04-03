use std::fmt::Display;

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
    ) -> Result<Vec<&'tree XpathItemTreeNode>, ExpressionApplyError> {
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

/// Evaluate a forward axis with fused node-test filtering.
///
/// Instead of collecting all axis nodes into an intermediate set and then
/// filtering, each axis function applies the node test during traversal,
/// returning only matching nodes in a `Vec` (no hashing overhead).
fn eval_forward_axis<'tree>(
    context: &XpathExpressionContext<'tree>,
    axis: ForwardAxis,
    node_test: &NodeTest,
) -> Result<Vec<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let bi_axis = BiDirectionalAxis::ForwardAxis(axis);
    match axis {
        ForwardAxis::Child => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                for child in node.children(context.item_tree) {
                    if node_test.matches_node(bi_axis, child, context.item_tree)? {
                        nodes.push(child);
                    }
                }
            } else {
                return Err(ExpressionApplyError {
                    msg: String::from(
                        "err:XPTY0020 context item for axis step is not a node",
                    ),
                });
            }
            Ok(nodes)
        }
        ForwardAxis::Descendant => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                let node_id = node_or_root_id(node, context);
                for desc_id in node_id.descendants(&context.item_tree.arena).skip(1) {
                    let desc_node = context.item_tree.get(desc_id);
                    if node_test.matches_node(bi_axis, desc_node, context.item_tree)? {
                        nodes.push(desc_node);
                    }
                }
            } else {
                return Err(ExpressionApplyError {
                    msg: String::from(
                        "err:XPTY0020 context item for axis step is not a node",
                    ),
                });
            }
            Ok(nodes)
        }
        ForwardAxis::DescendantOrSelf => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                let node_id = node_or_root_id(node, context);
                for desc_id in node_id.descendants(&context.item_tree.arena) {
                    let desc_node = context.item_tree.get(desc_id);
                    if node_test.matches_node(bi_axis, desc_node, context.item_tree)? {
                        nodes.push(desc_node);
                    }
                }
            } else {
                return Err(ExpressionApplyError {
                    msg: String::from(
                        "err:XPTY0020 context item for axis step is not a node",
                    ),
                });
            }
            Ok(nodes)
        }
        ForwardAxis::SelfAxis => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if node_test.matches_node(bi_axis, node, context.item_tree)? {
                    nodes.push(*node);
                }
            } else {
                return Err(ExpressionApplyError {
                    msg: String::from(
                        "err:XPTY0020 context item for axis step is not a node",
                    ),
                });
            }
            Ok(nodes)
        }
        ForwardAxis::Attribute => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(XpathItemTreeNode::ElementNode(element)) = context.item {
                for child in element.children(context.item_tree) {
                    if let XpathItemTreeNode::AttributeNode(_) = &child {
                        if node_test.matches_node(bi_axis, child, context.item_tree)? {
                            nodes.push(child);
                        }
                    }
                }
            } else if !matches!(context.item, XpathItem::Node(_)) {
                return Err(ExpressionApplyError {
                    msg: String::from(
                        "err:XPTY0020 context item for axis step is not a node",
                    ),
                });
            }
            Ok(nodes)
        }
        ForwardAxis::FollowingSibling => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if let Some(node_id) = node.node_id() {
                    let mut next =
                        context.item_tree.arena.get(node_id).and_then(|n| n.next_sibling());
                    while let Some(sibling_id) = next {
                        let sibling = context.item_tree.get(sibling_id);
                        if node_test.matches_node(bi_axis, sibling, context.item_tree)? {
                            nodes.push(sibling);
                        }
                        next = context
                            .item_tree
                            .arena
                            .get(sibling_id)
                            .and_then(|n| n.next_sibling());
                    }
                }
            } else {
                return Err(ExpressionApplyError {
                    msg: String::from(
                        "err:XPTY0020 context item for axis step is not a node",
                    ),
                });
            }
            Ok(nodes)
        }
        ForwardAxis::Following => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if let Some(node_id) = node.node_id() {
                    let mut current = Some(node_id);
                    while let Some(cur_id) = current {
                        let mut next =
                            context.item_tree.arena.get(cur_id).and_then(|n| n.next_sibling());
                        while let Some(sibling_id) = next {
                            for desc_id in
                                sibling_id.descendants(&context.item_tree.arena)
                            {
                                let desc_node = context.item_tree.get(desc_id);
                                if node_test.matches_node(
                                    bi_axis, desc_node, context.item_tree,
                                )? {
                                    nodes.push(desc_node);
                                }
                            }
                            next = context
                                .item_tree
                                .arena
                                .get(sibling_id)
                                .and_then(|n| n.next_sibling());
                        }
                        current =
                            context.item_tree.arena.get(cur_id).and_then(|n| n.parent());
                    }
                }
                // Sort in document order (ascending by node_id).
                nodes.sort_by_key(|n| n.node_id());
            } else {
                return Err(ExpressionApplyError {
                    msg: String::from(
                        "err:XPTY0020 context item for axis step is not a node",
                    ),
                });
            }
            Ok(nodes)
        }
        ForwardAxis::Namespace => Err(ExpressionApplyError {
            msg: String::from("namespace:: axis is not supported for HTML documents"),
        }),
    }
}

/// Get the [`NodeId`] for a tree node, falling back to the tree root for `DocumentNode`
/// (which doesn't store its own id).
fn node_or_root_id(node: &XpathItemTreeNode, context: &XpathExpressionContext<'_>) -> NodeId {
    match node.node_id() {
        Some(id) => id,
        None => context.item_tree.root_node, // DocumentNode
    }
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
