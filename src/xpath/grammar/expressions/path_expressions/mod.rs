//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    multi::many0, sequence::tuple,
};

use crate::xpath::{
    grammar::{
        data_model::Node, expressions::path_expressions::steps::step_expr::step_expr, recipes::Res,
    },
    Expression, ExpressionApplyError, XPathExpressionContext, XPathResult,
};

use self::steps::step_expr::StepExpr;

use super::Expr;

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

#[derive(PartialEq, Debug)]
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

impl Expression for PathExpr {
    fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        // Leading slashes mean different things than slashes in the middle of a path expression
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
        match self {
            PathExpr::LeadingSlash(_) => todo!("PathExpr::LeadingSlash"),
            PathExpr::LeadingDoubleSlash(expr) => {
                let nodes = vec![Node::TreeNode(context.item_tree.root())];
                let initial_expr = initial_double_slash_expansion();
                let context = XPathExpressionContext {
                    item_tree: context.item_tree,
                    searchable_nodes: nodes,
                };
                expr.eval(&context)
            }
            PathExpr::Plain(_) => todo!("PathExpr::Plain"),
        }
    }
}

fn initial_double_slash_expansion() -> Expr {
    // A leading double slash is expanded to `(fn:root(self::node()) treat as document-node())/descendant-or-self::node()/`
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions
    todo!("initial_double_slash_expansion")
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
        tuple((alt((slash, double_slash)), step_expr))(input)
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

#[derive(PartialEq, Debug)]
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

impl Expression for RelativePathExpr {
    fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        let nodes = self.expr.eval(context)?;

        for pair in self.items.iter() {
            // TODO: Double slash is expanded to `/descendant-or-self::node/`
            if let PathSeparator::DoubleSlash = pair.0 {
                todo!("RelativePathExpr::eval double slash")
            }
        }
        todo!("RelativePathExpr::eval")
    }
}

#[derive(PartialEq, Debug)]
pub struct StepPair(pub PathSeparator, pub StepExpr);

impl Display for StepPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        write!(f, "{}", self.1)
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
    use crate::xpath::parse;

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
        let expected_expr_text = r#"(fn:root(self::node()) treat as document-node())"#; // /descendant-or-self::node()/"#;

        let parsed_expected_expr = parse(expected_expr_text).unwrap();

        // act
        let expr = initial_double_slash_expansion();

        // assert
        assert_eq!(expr.to_string(), expected_expr_text);
    }
}
