use std::fmt::Display;

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
    ExpressionApplyError, XpathExpressionContext,
};

use super::{
    axes::reverse_axis::ReverseAxis,
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
    ) -> Result<Vec<&'tree XpathItemTreeNode>, ExpressionApplyError> {
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

/// Evaluate a reverse axis with fused node-test filtering.
fn eval_reverse_axis<'tree>(
    context: &XpathExpressionContext<'tree>,
    axis: ReverseAxis,
    node_test: &NodeTest,
) -> Result<Vec<&'tree XpathItemTreeNode>, ExpressionApplyError> {
    let bi_axis = BiDirectionalAxis::ReverseAxis(axis);
    match axis {
        ReverseAxis::Parent => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if let Some(parent) = &node.parent(context.item_tree) {
                    if node_test.matches_node(bi_axis, *parent, context.item_tree)? {
                        nodes.push(*parent);
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
        ReverseAxis::Ancestor => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if let Some(node_id) = node.node_id() {
                    let mut current =
                        context.item_tree.arena.get(node_id).and_then(|n| n.parent());
                    while let Some(ancestor_id) = current {
                        let ancestor = context.item_tree.get(ancestor_id);
                        if node_test.matches_node(bi_axis, ancestor, context.item_tree)? {
                            nodes.push(ancestor);
                        }
                        current = context
                            .item_tree
                            .arena
                            .get(ancestor_id)
                            .and_then(|n| n.parent());
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
        ReverseAxis::AncestorOrSelf => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if node_test.matches_node(bi_axis, *node, context.item_tree)? {
                    nodes.push(*node);
                }
                if let Some(node_id) = node.node_id() {
                    let mut current =
                        context.item_tree.arena.get(node_id).and_then(|n| n.parent());
                    while let Some(ancestor_id) = current {
                        let ancestor = context.item_tree.get(ancestor_id);
                        if node_test.matches_node(bi_axis, ancestor, context.item_tree)? {
                            nodes.push(ancestor);
                        }
                        current = context
                            .item_tree
                            .arena
                            .get(ancestor_id)
                            .and_then(|n| n.parent());
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
        ReverseAxis::PrecedingSibling => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if let Some(node_id) = node.node_id() {
                    let mut prev = context
                        .item_tree
                        .arena
                        .get(node_id)
                        .and_then(|n| n.previous_sibling());
                    while let Some(sibling_id) = prev {
                        let sibling = context.item_tree.get(sibling_id);
                        if node_test.matches_node(bi_axis, sibling, context.item_tree)? {
                            nodes.push(sibling);
                        }
                        prev = context
                            .item_tree
                            .arena
                            .get(sibling_id)
                            .and_then(|n| n.previous_sibling());
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
        ReverseAxis::Preceding => {
            let mut nodes = Vec::new();
            if let XpathItem::Node(node) = &context.item {
                if let Some(node_id) = node.node_id() {
                    let mut current = Some(node_id);
                    while let Some(cur_id) = current {
                        let mut prev = context
                            .item_tree
                            .arena
                            .get(cur_id)
                            .and_then(|n| n.previous_sibling());
                        while let Some(sibling_id) = prev {
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
                            prev = context
                                .item_tree
                                .arena
                                .get(sibling_id)
                                .and_then(|n| n.previous_sibling());
                        }
                        current =
                            context.item_tree.arena.get(cur_id).and_then(|n| n.parent());
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
    }
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
