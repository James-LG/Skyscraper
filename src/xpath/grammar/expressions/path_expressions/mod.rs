//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions>

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    multi::many0, sequence::tuple,
};

use once_cell::sync::Lazy;

use crate::xpath::grammar::data_model::XpathItem;
use crate::xpath::grammar::types::KindTest;
use crate::xpath::grammar::whitespace_recipes::ws;
use crate::xpath::grammar::XpathItemTreeNode;
use crate::xpath::xpath_item_set::XpathItemSet;
use crate::xpath::{
    grammar::{expressions::path_expressions::steps::step_expr::step_expr, recipes::Res},
    ExpressionApplyError, XpathExpressionContext,
};

use self::steps::{
    axes::forward_axis::ForwardAxis,
    axis_step::{AxisStep, AxisStepType},
    forward_step::ForwardStep,
    node_tests::{BiDirectionalAxis, NodeTest},
    step_expr::StepExpr,
};

pub mod abbreviated_syntax;
pub mod steps;

/// Lazily-parsed `(fn:root(self::node()) treat as document-node())` step expression.
static ROOT_STEP: Lazy<StepExpr> = Lazy::new(|| {
    step_expr("(fn:root(self::node()) treat as document-node())")
        .expect("ROOT_STEP parse failed")
        .1
});

/// Lazily-parsed `descendant-or-self::node()` step expression.
static DESC_OR_SELF_STEP: Lazy<StepExpr> = Lazy::new(|| {
    step_expr("descendant-or-self::node()")
        .expect("DESC_OR_SELF_STEP parse failed")
        .1
});

/// Lazily-parsed `.` (context item) step expression.
static DOT_STEP: Lazy<StepExpr> = Lazy::new(|| {
    step_expr(".")
        .expect("DOT_STEP parse failed")
        .1
});

pub fn path_expr(input: &str) -> Res<&str, PathExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-PathExpr

    fn leading_slash(input: &str) -> Res<&str, PathExpr> {
        ws((char('/'), opt(relative_path_expr)))(input)
            .map(|(next_input, res)| (next_input, PathExpr::LeadingSlash(res.1)))
    }

    fn leading_double_slash(input: &str) -> Res<&str, PathExpr> {
        ws((tag("//"), relative_path_expr))(input)
            .map(|(next_input, res)| (next_input, PathExpr::LeadingDoubleSlash(res.1)))
    }

    fn plain(input: &str) -> Res<&str, PathExpr> {
        relative_path_expr(input).map(|(next_input, res)| (next_input, PathExpr::Plain(res)))
    }

    context(
        "path_expr",
        alt((leading_double_slash, leading_slash, plain)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum PathExpr {
    LeadingSlash(Option<RelativePathExpr>),
    LeadingDoubleSlash(RelativePathExpr),
    Plain(RelativePathExpr),
}

impl Display for PathExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathExpr::LeadingSlash(x) => {
                write!(f, "/")?;
                if let Some(x) = x {
                    write!(f, "{}", x)?;
                }

                Ok(())
            }
            PathExpr::LeadingDoubleSlash(x) => write!(f, "//{}", x),
            PathExpr::Plain(x) => write!(f, "{}", x),
        }
    }
}

impl PathExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Leading slashes mean different things than slashes in the middle of a path expression
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
        match self {
            PathExpr::LeadingSlash(expr) => {
                let expanded_expr = if context.is_initial_step {
                    initial_slash_expansion(expr)
                } else {
                    relative_slash_expansion(expr)
                };

                expanded_expr.eval(context)
            }
            PathExpr::LeadingDoubleSlash(expr) => {
                let expanded_expr = if context.is_initial_step {
                    initial_double_slash_expansion(expr)
                } else {
                    relative_double_slash_expansion(expr)
                };

                expanded_expr.eval(context)
            }
            PathExpr::Plain(expr) => expr.eval(context),
        }
    }
}

fn initial_slash_expansion(unexpanded_expr: &Option<RelativePathExpr>) -> RelativePathExpr {
    // A leading slash is expanded to `(fn:root(self::node()) treat as document-node())/<unexpanded_expr>`
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
    let first_step = ROOT_STEP.clone();

    let items = match unexpanded_expr {
        Some(x) => {
            let mut items = vec![StepPair(PathSeparator::Slash, x.expr.clone())];
            items.extend(x.items.iter().cloned());
            items
        }
        None => vec![],
    };

    RelativePathExpr {
        expr: first_step,
        items,
    }
}

fn relative_slash_expansion(unexpanded_expr: &Option<RelativePathExpr>) -> RelativePathExpr {
    // A leading slash in a relative expression is expanded to `./<unexpanded_expr>`
    let first_step = DOT_STEP.clone();

    let items = match unexpanded_expr {
        Some(x) => {
            let mut items = vec![StepPair(PathSeparator::Slash, x.expr.clone())];
            items.extend(x.items.iter().cloned());
            items
        }
        None => vec![],
    };

    RelativePathExpr {
        expr: first_step,
        items,
    }
}

/// Try to extract a child-axis node test from a step expression, for `//X` fusion.
/// Returns `Some(node_test)` if the step is a predicate-free child axis step.
fn try_extract_child_node_test(step: &StepExpr) -> Option<&NodeTest> {
    if let StepExpr::AxisStep(axis_step) = step {
        if !axis_step.predicates.is_empty() {
            return None;
        }
        match &axis_step.step_type {
            AxisStepType::ForwardStep(ForwardStep::Full(ForwardAxis::Child, node_test)) => {
                Some(node_test)
            }
            AxisStepType::ForwardStep(ForwardStep::Abbreviated(abbrev)) if !abbrev.has_at => {
                Some(&abbrev.node_test)
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Build a `descendant::node_test` step expression from a node test.
fn make_descendant_step(node_test: &NodeTest) -> StepExpr {
    StepExpr::AxisStep(AxisStep {
        step_type: AxisStepType::ForwardStep(ForwardStep::Full(
            ForwardAxis::Descendant,
            node_test.clone(),
        )),
        predicates: vec![],
    })
}

/// Check if a step expression is `descendant-or-self::node()` (no predicates).
fn is_desc_or_self_any_kind(step: &StepExpr) -> bool {
    if let StepExpr::AxisStep(axis_step) = step {
        axis_step.predicates.is_empty()
            && matches!(
                &axis_step.step_type,
                AxisStepType::ForwardStep(ForwardStep::Full(
                    ForwardAxis::DescendantOrSelf,
                    NodeTest::KindTest(KindTest::AnyKindTest)
                ))
            )
    } else {
        false
    }
}

/// Try to extract the node test and predicates from a child-axis step with predicates.
/// Returns `Some((node_test, axis_step))` for `child::X[pred]` or abbreviated `X[pred]`.
fn try_extract_child_with_predicates(step: &StepExpr) -> Option<(&NodeTest, &AxisStep)> {
    if let StepExpr::AxisStep(axis_step) = step {
        if axis_step.predicates.is_empty() {
            return None;
        }
        let node_test = match &axis_step.step_type {
            AxisStepType::ForwardStep(ForwardStep::Full(ForwardAxis::Child, nt)) => Some(nt),
            AxisStepType::ForwardStep(ForwardStep::Abbreviated(abbrev)) if !abbrev.has_at => {
                Some(&abbrev.node_test)
            }
            _ => None,
        };
        node_test.map(|nt| (nt, axis_step))
    } else {
        None
    }
}

/// Optimized evaluation of `descendant-or-self::node()/child::X[predicates]`.
///
/// Instead of evaluating `child::X[pred]` for each of ~N descendant-or-self nodes
/// (O(N) axis evaluations), this walks descendants once and groups matching nodes
/// by parent, then evaluates predicates per group. This preserves positional
/// semantics (e.g. `[1]` means first child of each parent).
fn eval_grouped_descendant_predicate<'tree>(
    context: &XpathExpressionContext<'tree>,
    node_test: &NodeTest,
    axis_step: &AxisStep,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    let context_node = match &context.item {
        XpathItem::Node(node) => node,
        _ => return Ok(XpathItemSet::new()),
    };

    let node_id = match context_node.node_id() {
        Some(id) => id,
        None => context.item_tree.root_node, // DocumentNode
    };

    let bi_axis = BiDirectionalAxis::ForwardAxis(ForwardAxis::Child);

    // Walk all descendants, filter by node test, group by parent.
    // Use a Vec of (parent_id, Vec<node>) to preserve insertion order.
    let mut group_order: Vec<Option<indextree::NodeId>> = Vec::new();
    let mut groups: std::collections::HashMap<
        Option<indextree::NodeId>,
        Vec<&'tree XpathItemTreeNode>,
    > = std::collections::HashMap::new();

    for desc_id in node_id.descendants(&context.item_tree.arena).skip(1) {
        let desc_node = context.item_tree.get(desc_id);
        if node_test.matches_node(bi_axis, desc_node, context.item_tree)? {
            let parent_id = context
                .item_tree
                .arena
                .get(desc_id)
                .and_then(|n| n.parent());
            let group = groups.entry(parent_id).or_insert_with(|| {
                group_order.push(parent_id);
                Vec::new()
            });
            group.push(desc_node);
        }
    }

    // Pre-check for constant integer position predicates (e.g., [1], [2]).
    // If ALL predicates are constant positions, we can skip full predicate
    // evaluation entirely and just pick items by position from each group.
    let constant_positions: Option<Vec<i64>> = axis_step
        .predicates
        .iter()
        .map(|p| p.try_constant_position())
        .collect();

    let mut result = XpathItemSet::new();

    if let Some(positions) = constant_positions {
        // Fast path: all predicates are constant integer positions.
        // Apply them sequentially — each narrows the set for the next.
        for parent_id in &group_order {
            let mut current: Vec<&'tree XpathItemTreeNode> = groups[parent_id].clone();
            for &pos in &positions {
                if pos >= 1 && (pos as usize) <= current.len() {
                    current = vec![current[pos as usize - 1]];
                } else {
                    current.clear();
                    break;
                }
            }
            for node in current {
                result.insert(XpathItem::Node(node));
            }
        }
    } else {
        // General path: evaluate predicates using the expression evaluator.
        // Avoid creating XpathItemSet per group — use direct context creation.
        for parent_id in &group_order {
            let siblings = &groups[parent_id];
            let group_size = siblings.len();

            for (i, &node) in siblings.iter().enumerate() {
                let pred_context = context.new_with_item_and_size(
                    XpathItem::Node(node),
                    i + 1,
                    group_size,
                    context.is_initial_step,
                );
                let mut is_match = true;
                for predicate in axis_step.predicates.iter() {
                    if !predicate.is_match(&pred_context)? {
                        is_match = false;
                        break;
                    }
                }
                if is_match {
                    result.insert(XpathItem::Node(node));
                }
            }
        }
    }

    result.sort_by_document_order();
    result.dedup();
    Ok(result)
}

fn initial_double_slash_expansion(unexpanded_expr: &RelativePathExpr) -> RelativePathExpr {
    // A leading double slash is expanded to `(fn:root(self::node()) treat as document-node())/descendant-or-self::node()/<unexpanded_expr>`
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
    let first_step = ROOT_STEP.clone();

    // Optimization: //X (no predicates) is equivalent to root/descendant::X
    // This avoids the O(N) nested loop of descendant-or-self::node()/child::X.
    if let Some(node_test) = try_extract_child_node_test(&unexpanded_expr.expr) {
        let descendant_step = make_descendant_step(node_test);
        let mut items = vec![StepPair(PathSeparator::Slash, descendant_step)];
        items.extend(unexpanded_expr.items.iter().cloned());
        return RelativePathExpr {
            expr: first_step,
            items,
        };
    }

    let second_step = DESC_OR_SELF_STEP.clone();

    let mut items = vec![StepPair(PathSeparator::Slash, second_step)];
    items.push(StepPair(
        PathSeparator::Slash,
        unexpanded_expr.expr.clone(),
    ));
    items.extend(unexpanded_expr.items.iter().cloned());

    RelativePathExpr {
        expr: first_step,
        items,
    }
}

fn relative_double_slash_expansion(unexpanded_expr: &RelativePathExpr) -> RelativePathExpr {
    // A leading double slash in a relative expression is expanded to `.//<unexpanded_expr>`
    let first_step = DOT_STEP.clone();

    // Optimization: .//<X> (no predicates) → ./descendant::<X>
    if let Some(node_test) = try_extract_child_node_test(&unexpanded_expr.expr) {
        let descendant_step = make_descendant_step(node_test);
        let mut items = vec![StepPair(PathSeparator::Slash, descendant_step)];
        items.extend(unexpanded_expr.items.iter().cloned());
        return RelativePathExpr {
            expr: first_step,
            items,
        };
    }

    let mut items = vec![StepPair(
        PathSeparator::DoubleSlash,
        unexpanded_expr.expr.clone(),
    )];
    items.extend(unexpanded_expr.items.iter().cloned());

    RelativePathExpr {
        expr: first_step,
        items,
    }
}

pub fn relative_path_expr(input: &str) -> Res<&str, RelativePathExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-RelativePathExpr

    fn slash(input: &str) -> Res<&str, PathSeparator> {
        char('/')(input).map(|(next_input, _res)| (next_input, PathSeparator::Slash))
    }

    fn double_slash(input: &str) -> Res<&str, PathSeparator> {
        tag("//")(input).map(|(next_input, _res)| (next_input, PathSeparator::DoubleSlash))
    }

    fn step_pair(input: &str) -> Res<&str, StepPair> {
        ws((alt((double_slash, slash)), step_expr))(input)
            .map(|(next_input, res)| (next_input, StepPair(res.0, res.1)))
    }

    context("relative_path_expr", tuple((step_expr, many0(step_pair))))(input).map(
        |(next_input, res)| {
            (
                next_input,
                RelativePathExpr {
                    expr: res.0,
                    items: res.1,
                },
            )
        },
    )
}

#[derive(PartialEq, Debug, Clone)]
pub struct RelativePathExpr {
    pub expr: StepExpr,
    pub items: Vec<StepPair>,
}

impl Display for RelativePathExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, "{}", x)?;
        }

        Ok(())
    }
}

impl RelativePathExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        /// Recursively evaluate a series of steps.
        fn eval_steps<'tree>(
            context: &XpathExpressionContext<'tree>,
            steps: &[StepPair],
        ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
            // If there are no steps, return the context item.
            if steps.is_empty() {
                let mut result = XpathItemSet::new();
                result.insert(context.item.clone());
                return Ok(result);
            }

            // Terminal step optimization: when only one step remains,
            // return its result directly without creating per-item contexts.
            if steps.len() == 1 {
                return steps[0].eval(context);
            }

            // Grouped descendant optimization: detect
            //   desc-or-self::node() / child::X[predicates]
            // and replace with a single descendant traversal grouped by parent.
            if steps.len() >= 2
                && matches!(steps[0].0, PathSeparator::Slash)
                && matches!(steps[1].0, PathSeparator::Slash)
                && is_desc_or_self_any_kind(&steps[0].1)
            {
                if let Some((node_test, axis_step)) =
                    try_extract_child_with_predicates(&steps[1].1)
                {
                    let matched =
                        eval_grouped_descendant_predicate(context, node_test, axis_step)?;

                    // If there are remaining steps, continue evaluating them.
                    if steps.len() > 2 {
                        let mut items = XpathItemSet::new();
                        for (i, _item) in matched.iter().enumerate() {
                            let inner_context = context.new_with_variables(
                                &matched,
                                i + 1,
                                context.is_initial_step,
                            );
                            let inner_result = eval_steps(&inner_context, &steps[2..])?;
                            items.extend(inner_result);
                        }
                        return Ok(items);
                    }
                    return Ok(matched);
                }
            }

            // Otherwise, evaluate the first step.
            let mut items = XpathItemSet::new();
            let this_result = steps[0].eval(context)?;

            // For each item in the result of the first step, recursively evaluate the rest of the steps.
            // The goal is to feed the result of the first steps into the following steps,
            // so that the final result is only the result of the last step.
            for (i, _item) in this_result.iter().enumerate() {
                // Create a context for the inner steps using an item from the current result.
                let inner_context = context.new_with_variables(
                    &this_result,
                    i + 1,
                    context.is_initial_step,
                );

                // Recursively evaluate the rest of the steps for this item.
                let inner_result = eval_steps(&inner_context, &steps[1..])?;

                // Add the result of the inner steps to the result.
                items.extend(inner_result);
            }

            Ok(items)
        }
        let e1_result = self.expr.eval(context)?;

        // If there are no items, return the result of the expression.
        if self.items.is_empty() {
            return Ok(e1_result);
        }

        // Otherwise, for each item in the result of the expression, evaluate the steps.
        let mut items = XpathItemSet::new();
        for (i, _item) in e1_result.iter().enumerate() {
            let en_context = context.new_with_variables(
                &e1_result,
                i + 1,
                context.is_initial_step,
            );
            let result = eval_steps(&en_context, &self.items)?;
            items.extend(result);
        }

        // Path expressions must return results in document order.
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
        items.sort_by_document_order();
        items.dedup();

        Ok(items)
    }
}

/// Double slash is expanded to `/descendant-or-self::node/`
///
/// # Arguments
///
/// `expr` - The step _after_ the double slash.
fn double_slash_expansion(expr: &StepExpr) -> RelativePathExpr {
    // Optimization: A//X (no predicates) → A/descendant::X
    if let Some(node_test) = try_extract_child_node_test(expr) {
        let descendant_step = make_descendant_step(node_test);
        return RelativePathExpr {
            expr: descendant_step,
            items: vec![],
        };
    }

    let expanded_double_slash = DESC_OR_SELF_STEP.clone();

    let items = vec![StepPair(PathSeparator::Slash, expr.clone())];

    RelativePathExpr {
        expr: expanded_double_slash,
        items,
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct StepPair(pub PathSeparator, pub StepExpr);

impl Display for StepPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        write!(f, "{}", self.1)
    }
}

impl StepPair {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let result: XpathItemSet<'_> = match self.0 {
            PathSeparator::Slash => self.1.eval(context)?,
            PathSeparator::DoubleSlash => {
                // Grouped descendant optimization for A//X[pred]:
                // Instead of expanding to desc-or-self::node()/child::X[pred] and
                // evaluating child::X[pred] for each of ~N nodes, walk descendants
                // once and group by parent.
                if let Some((node_test, axis_step)) =
                    try_extract_child_with_predicates(&self.1)
                {
                    eval_grouped_descendant_predicate(context, node_test, axis_step)?
                } else {
                    let expanded_e2 = double_slash_expansion(&self.1);
                    expanded_e2.eval(context)?
                }
            }
        };

        Ok(result)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PathSeparator {
    Slash,
    DoubleSlash,
}

impl Display for PathSeparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSeparator::Slash => write!(f, "/"),
            PathSeparator::DoubleSlash => write!(f, "//"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn path_expr_should_parse() {
        // arrange
        let input = "/div/span";

        // act
        let (next_input, res) = path_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn path_expr_should_parse_whitespace() {
        // arrange
        let input = "/ div / span";

        // act
        let (next_input, res) = path_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "/div/span");
    }

    #[test]
    fn relative_path_expr_should_parse() {
        // arrange
        let input = r#"child::div1/child::para"#;

        // act
        let (next_input, res) = relative_path_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn initial_double_slash_expansion_should_be_as_documented() {
        // arrange
        let given_expr = relative_path_expr("hi").unwrap().1;

        // act
        let expr = initial_double_slash_expansion(&given_expr);

        // assert
        // Optimized: //hi → root/descendant::hi (fused from desc-or-self::node()/child::hi)
        let expected_expr_text =
            r#"(fn:root(self::node()) treat as document-node())/descendant::hi"#;

        assert_eq!(expr.to_string(), expected_expr_text);
    }

    #[test]
    fn initial_slash_expansion_should_be_as_documented() {
        // arrange
        let given_expr = relative_path_expr("hi").unwrap().1;

        // act
        let expr = initial_slash_expansion(&Some(given_expr));

        // assert
        let expected_expr_text = r#"(fn:root(self::node()) treat as document-node())/hi"#;

        assert_eq!(expr.to_string(), expected_expr_text);
    }

    #[test]
    fn initial_slash_expansion_no_expr_should_be_as_documented() {
        // arrange
        let given_expr: Option<RelativePathExpr> = None;

        // act
        let expr = initial_slash_expansion(&given_expr);

        // assert
        let expected_expr_text = r#"(fn:root(self::node()) treat as document-node())"#;

        assert_eq!(expr.to_string(), expected_expr_text);
    }

    #[test]
    fn double_slash_expansion_should_be_as_documented() {
        // arrange
        let given_expr = step_expr("hello").unwrap().1;

        // act
        let expr = double_slash_expansion(&given_expr);

        // assert
        // Optimized: A//hello → A/descendant::hello (fused from desc-or-self::node()/child::hello)
        let expected_expr_text = r#"descendant::hello"#;

        assert_eq!(expr.to_string(), expected_expr_text);
    }
}
