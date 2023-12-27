//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    multi::many0, sequence::tuple,
};

use crate::xpath::xpath_item_set::XpathItemSet;
use crate::xpath::{
    grammar::{expressions::path_expressions::steps::step_expr::step_expr, recipes::Res},
    ExpressionApplyError, XpathExpressionContext,
};

use self::steps::step_expr::StepExpr;

pub mod abbreviated_syntax;
pub mod steps;

pub fn path_expr(input: &str) -> Res<&str, PathExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-PathExpr

    fn leading_slash(input: &str) -> Res<&str, PathExpr> {
        tuple((char('/'), opt(relative_path_expr)))(input)
            .map(|(next_input, res)| (next_input, PathExpr::LeadingSlash(res.1)))
    }

    fn leading_double_slash(input: &str) -> Res<&str, PathExpr> {
        tuple((tag("//"), relative_path_expr))(input)
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
                let expanded_expr = initial_slash_expansion(expr);
                expanded_expr.eval(context)
            }
            PathExpr::LeadingDoubleSlash(expr) => {
                let expanded_expr = initial_double_slash_expansion(expr);
                expanded_expr.eval(context)
            }
            PathExpr::Plain(expr) => expr.eval(context),
        }
    }
}

fn initial_slash_expansion(unexpanded_expr: &Option<RelativePathExpr>) -> RelativePathExpr {
    // A leading slash is expanded to `(fn:root(self::node()) treat as document-node())/`
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
    let first_step = step_expr("(fn:root(self::node()) treat as document-node())")
        .expect("slash expansion step 1 failed")
        .1;

    let items = match unexpanded_expr {
        Some(x) => {
            let mut items = vec![StepPair(PathSeparator::Slash, x.expr.clone())];
            items.extend(x.items.iter().map(|x| x.clone()));
            items
        }
        None => vec![],
    };

    RelativePathExpr {
        expr: first_step,
        items,
    }
}

fn initial_double_slash_expansion(unexpanded_expr: &RelativePathExpr) -> RelativePathExpr {
    // A leading double slash is expanded to `(fn:root(self::node()) treat as document-node())/descendant-or-self::node()/`
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
    let first_step = step_expr("(fn:root(self::node()) treat as document-node())")
        .expect("double slash expansion step 1 failed")
        .1;

    let second_step = step_expr("descendant-or-self::node()")
        .expect("double slash expansion step 2 failed")
        .1;

    let mut items = vec![StepPair(PathSeparator::Slash, second_step)];
    items.push(StepPair(PathSeparator::Slash, unexpanded_expr.expr.clone()));
    items.extend(unexpanded_expr.items.iter().map(|x| x.clone()));

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
        tuple((alt((double_slash, slash)), step_expr))(input)
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

            // Otherwise, evaluate the first step.
            let mut items = XpathItemSet::new();
            let this_result = steps[0].eval(context)?;

            // For each item in the result of the first step, recursively evaluate the rest of the steps.
            // The goal is to feed the result of the first steps into the following steps,
            // so that the final result is only the result of the last step.
            for (i, _item) in this_result.iter().enumerate() {
                // Create a context for the inner steps using an item from the current result.
                let inner_context =
                    XpathExpressionContext::new(context.item_tree, &this_result, i + 1);

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
            let en_context = XpathExpressionContext::new(context.item_tree, &e1_result, i + 1);
            let result = eval_steps(&en_context, &self.items)?;
            items.extend(result);
        }

        Ok(items)
    }
}

/// Double slash is expanded to `/descendant-or-self::node/`
///
/// # Arguments
///
/// `expr` - The step _after_ the double slash.
fn double_slash_expansion(expr: &StepExpr) -> RelativePathExpr {
    let expanded_double_slash = step_expr("descendant-or-self::node()")
        .expect("double slash expansion step 1 failed")
        .1;

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
                let expanded_e2 = double_slash_expansion(&self.1);
                expanded_e2.eval(context)?
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
        let expected_expr_text =
            r#"(fn:root(self::node()) treat as document-node())/descendant-or-self::node()/hi"#;

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
        let expected_expr_text = r#"descendant-or-self::node()/hello"#;

        assert_eq!(expr.to_string(), expected_expr_text);
    }
}
