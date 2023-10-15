//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-path-expressions

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, multi::many0,
    sequence::tuple,
};

use crate::xpath::grammar::{expressions::path_expressions::steps::step_expr, recipes::Res};

use self::steps::StepExpr;

mod abbreviated_syntax;
mod steps;

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

    alt((leading_double_slash, leading_slash, plain))(input)
}

pub enum PathExpr {
    LeadingSlash(Option<RelativePathExpr>),
    LeadingDoubleSlash(RelativePathExpr),
    Plain(RelativePathExpr),
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

    tuple((step_expr, many0(step_pair)))(input).map(|(next_input, res)| {
        let pair = StepPair(None, res.0);
        let extras = res.1;
        (next_input, RelativePathExpr { pair, extras })
    })
}

pub struct RelativePathExpr {
    pub pair: StepPair,
    pub extras: Vec<StepPair>,
}

pub struct StepPair(pub Option<PathSeparator>, pub StepExpr);

pub enum PathSeparator {
    Slash,
    DoubleSlash,
}
