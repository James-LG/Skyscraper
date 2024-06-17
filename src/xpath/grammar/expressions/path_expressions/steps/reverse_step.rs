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
        XpathItemTreeNodeData,
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
    ) -> Result<IndexSet<&'tree XpathItemTreeNodeData>, ExpressionApplyError> {
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
) -> Result<IndexSet<&'tree XpathItemTreeNodeData>, ExpressionApplyError> {
    let axis_nodes: IndexSet<&'tree XpathItemTreeNodeData> = match axis {
        ReverseAxis::Parent => eval_reverse_axis_parent(context),
        ReverseAxis::Ancestor => todo!("eval_reverse_axis ReverseAxis::Ancestor"),
        ReverseAxis::PrecedingSibling => todo!("eval_reverse_axis ReverseAxis::PrecedingSibling"),
        ReverseAxis::Preceding => todo!("eval_reverse_axis ReverseAxis::Preceding"),
        ReverseAxis::AncestorOrSelf => todo!("eval_reverse_axis ReverseAxis::AncestorOrSelf"),
    }?;

    let items: XpathItemSet<'tree> = axis_nodes.into_iter().map(XpathItem::Node).collect();
    let mut nodes = IndexSet::new();

    for (i, _node) in items.iter().enumerate() {
        let node_test_context =
            XpathExpressionContext::new(context.item_tree, &items, i + 1, context.is_root_level);

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
) -> Result<IndexSet<&'tree XpathItemTreeNodeData>, ExpressionApplyError> {
    let mut nodes: IndexSet<&'tree XpathItemTreeNodeData> = IndexSet::new();

    // Only tree items have parents
    // TODO: Technically an attribute's parent is an element, but there is no link to that ATM.
    if let XpathItem::Node(node) = &context.item {
        if let Some(parent) = &node.parent(context.item_tree) {
            nodes.insert(parent.clone());
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
