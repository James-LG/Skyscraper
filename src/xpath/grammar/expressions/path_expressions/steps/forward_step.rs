use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::XpathItem,
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

#[derive(PartialEq, Debug)]
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

impl Expression for ForwardStep {
    fn eval<'tree>(
        &self,
        context: XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        let mut result = Vec::new();

        for node in searchable_nodes {
            result.extend(node.children(&item_tree));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        html,
        xpath::grammar::{
            data_model::{ElementNode, Node},
            XPath, XpathItemTreeNodeData,
        },
    };

    use super::*;

    #[test]
    fn apply_child_should_select_all_children() {
        // arrange
        let text = r###"
            <root>
                <child1/>
                <child2/>
                <child3/>
            </root>
        "###;

        let html_doc = html::parse(text).unwrap();

        let xpath_item_tree = XpathItemTree::from_html_document(&html_doc);
        let searchable_nodes = vec![xpath_item_tree.root()];

        let xpath_text = "child::*";
        let subject = forward_step(xpath_text).unwrap().1;

        // act
        let result = subject.eval(&xpath_item_tree, &searchable_nodes).unwrap();

        // assert
        let mut items = result.into_iter();

        let item = items.next().unwrap().data;
        assert_eq!(
            item,
            &XpathItemTreeNodeData::ElementNode(ElementNode {
                name: String::from("child1"),
                attributes: Vec::new()
            })
        );

        let item = items.next().unwrap().data;
        assert_eq!(
            item,
            &XpathItemTreeNodeData::ElementNode(ElementNode {
                name: String::from("child2"),
                attributes: Vec::new()
            })
        );

        let item = items.next().unwrap().data;
        assert_eq!(
            item,
            &XpathItemTreeNodeData::ElementNode(ElementNode {
                name: String::from("child3"),
                attributes: Vec::new()
            })
        );
    }
}
