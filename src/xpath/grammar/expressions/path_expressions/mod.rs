//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    multi::many0, sequence::tuple,
};

use crate::xpath::grammar::{expressions::path_expressions::steps::step_expr, recipes::Res};

use self::steps::StepExpr;

pub mod abbreviated_syntax;
pub mod steps;

pub fn path_expr(input: &str) -> Res<&str, PathExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PathExpr

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
            .map(|(next_input, res)| (next_input, StepPair(Some(res.0), res.1)))
    }

    context("relative_path_expr", tuple((step_expr, many0(step_pair))))(input).map(
        |(next_input, res)| {
            let pair = StepPair(None, res.0);
            let items = res.1;
            (next_input, RelativePathExpr { pair, items })
        },
    )
}

#[derive(PartialEq, Debug)]
pub struct RelativePathExpr {
    pub pair: StepPair,
    pub items: Vec<StepPair>,
}

impl Display for RelativePathExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pair)?;
        for x in &self.items {
            write!(f, "{}", x)?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub struct StepPair(pub Option<PathSeparator>, pub StepExpr);

impl Display for StepPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(x) = self.0 {
            write!(f, "{}", x)?;
        }

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
}
