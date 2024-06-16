use std::fmt::Display;

use indexmap::IndexSet;
use nom::{branch::alt, error::context};

use crate::xpath::{
    grammar::{
        data_model::XpathItem,
        expressions::path_expressions::{
            abbreviated_syntax::{abbrev_forward_step, AbbrevForwardStep},
            steps::{axes::forward_axis::forward_axis, node_tests::node_test},
        },
        recipes::Res,
        whitespace_recipes::ws,
        XpathItemTreeNodeData,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext, XpathItemTreeNode,
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
    ) -> Result<IndexSet<XpathItemTreeNode<'tree>>, ExpressionApplyError> {
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
) -> Result<IndexSet<XpathItemTreeNode<'tree>>, ExpressionApplyError> {
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

    let items: XpathItemSet<'tree> = axis_nodes.into_iter().map(XpathItem::Node).collect();
    let mut nodes = IndexSet::new();

    for (i, _item) in items.iter().enumerate() {
        let node_test_context =
            XpathExpressionContext::new(context.item_tree, &items, i + 1, context.is_root_level);

        if let Some(result) =
            node_test.eval(BiDirectionalAxis::ForwardAxis(axis), &node_test_context)?
        {
            nodes.insert(result);
        }
    }

    Ok(nodes)
}

/// Direct children of the context nodes.
fn eval_forward_axis_child<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<XpathItemTreeNode<'tree>>, ExpressionApplyError> {
    let mut nodes: IndexSet<XpathItemTreeNode<'tree>> = IndexSet::new();

    // Only tree nodes have children
    if let XpathItem::Node(node) = &context.item {
        for child in node.children(context.item_tree) {
            nodes.insert(child);
        }
    }

    Ok(nodes)
}

/// All descendants of the context nodes.
fn eval_forward_axis_descendant<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<XpathItemTreeNode<'tree>>, ExpressionApplyError> {
    let mut nodes: IndexSet<XpathItemTreeNode<'tree>> = IndexSet::new();

    // Only tree nodes have children.
    if let XpathItem::Node(node) = &context.item {
        for child in node.children(context.item_tree) {
            // Add the child.
            nodes.insert(child.clone());

            // Add the child's descendants.
            let child_eval_context = XpathExpressionContext::new_single(
                context.item_tree,
                child.into(),
                context.is_root_level,
            );
            let child_descendants = eval_forward_axis_descendant(&child_eval_context)?;
            nodes.extend(child_descendants);
        }
    }

    Ok(nodes)
}

/// All descendants of the context nodes including the context nodes.
fn eval_forward_axis_self_or_descendant<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<XpathItemTreeNode<'tree>>, ExpressionApplyError> {
    let mut nodes = IndexSet::new();

    if let XpathItem::Node(node) = &context.item {
        nodes.insert(node.clone());
    } else {
        return Err(ExpressionApplyError {
            msg: String::from("err:XPTY0020 context item for axis step is not a node"),
        });
    }

    nodes.extend(eval_forward_axis_descendant(context)?);

    Ok(nodes)
}

// All attributes of the context nodes.
fn eval_forward_axis_attribute<'tree>(
    context: &XpathExpressionContext<'tree>,
) -> Result<IndexSet<XpathItemTreeNode<'tree>>, ExpressionApplyError> {
    let mut attributes: IndexSet<XpathItemTreeNode<'tree>> = IndexSet::new();

    // Only elements have attributes.
    if let XpathItem::Node(XpathItemTreeNode {
        data: XpathItemTreeNodeData::ElementNode(element),
        ..
    }) = context.item
    {
        for attribute in element.attributes.iter() {
            let attribute = attribute.clone();

            attributes.insert(attribute);
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
