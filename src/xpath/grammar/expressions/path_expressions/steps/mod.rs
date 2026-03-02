//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-steps>

use nom::{error::context, multi::many0};

use indexmap::IndexSet;

use crate::xpath::{
    grammar::{
        expressions::postfix_expressions::{predicate, Predicate},
        recipes::Res,
        XpathItemTreeNode,
    },
    XpathExpressionContext,
};

pub mod axes;
pub mod axis_step;
pub mod forward_step;
pub mod node_tests;
pub mod reverse_step;
pub mod step_expr;

/// Helper: insert a node and all its descendants into the set.
pub(super) fn collect_self_and_descendants<'tree>(
    context: &XpathExpressionContext<'tree>,
    node_id: indextree::NodeId,
    nodes: &mut IndexSet<&'tree XpathItemTreeNode>,
) {
    // Use indextree's built-in descendants iterator instead of manual recursion.
    for descendant_id in node_id.descendants(&context.item_tree.arena) {
        nodes.insert(context.item_tree.get(descendant_id));
    }
}

fn predicate_list(input: &str) -> Res<&str, Vec<Predicate>> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PredicateList

    context("predicate_list", many0(predicate))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_list_should_parse() {
        // arrange
        let input = "[1][2]";

        // act
        let (next_input, res) = predicate_list(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res[0].to_string(), "[1]");
        assert_eq!(res[1].to_string(), "[2]");
    }

    #[test]
    fn predicate_list_should_parse_whitespace() {
        // arrange
        let input = "[ 1 ] [ 2 ]";

        // act
        let (next_input, res) = predicate_list(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res[0].to_string(), "[1]");
        assert_eq!(res[1].to_string(), "[2]");
    }
}
