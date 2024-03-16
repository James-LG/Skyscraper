use std::fmt::Display;

use nom::{branch::alt, character::complete::char, combinator::opt, error::context, multi::many0};

use crate::xpath::{
    grammar::{expressions::expr_single, recipes::Res, whitespace_recipes::ws},
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::ExprSingle;

pub fn argument_list(input: &str) -> Res<&str, ArgumentList> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ArgumentList

    context(
        "argument_list",
        ws((
            char('('),
            opt(ws((argument, many0(ws((char(','), argument)))))),
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| {
        let mut arguments = Vec::new();
        if let Some(res) = res.1 {
            arguments.push(res.0);
            let extras = res.1.into_iter().map(|res| res.1);
            arguments.extend(extras);
        }
        (next_input, ArgumentList(arguments))
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct ArgumentList(pub Vec<Argument>);

impl Display for ArgumentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, x) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", x)?;
        }
        write!(f, ")")?;

        Ok(())
    }
}

pub fn argument(input: &str) -> Res<&str, Argument> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Argument

    fn argument_placeholder(input: &str) -> Res<&str, Argument> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ArgumentPlaceholder
        char('?')(input).map(|(next_input, _res)| (next_input, Argument::ArgumentPlaceHolder))
    }

    fn expr_single_map(input: &str) -> Res<&str, Argument> {
        expr_single(input).map(|(next_input, res)| (next_input, Argument::ExprSingle(res)))
    }

    context("argument", alt((expr_single_map, argument_placeholder)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Argument {
    ExprSingle(ExprSingle),
    ArgumentPlaceHolder,
}

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Argument::ExprSingle(x) => write!(f, "{}", x),
            Argument::ArgumentPlaceHolder => write!(f, "?"),
        }
    }
}

impl Argument {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        match &self {
            Argument::ExprSingle(expr_single) => expr_single.eval(context),
            Argument::ArgumentPlaceHolder => todo!("Argument::ArgumentPlaceHolder eval"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn argument_list_should_parse() {
        // arrange
        let input = "(2,3)";

        // act
        let (next_input, res) = argument_list(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "(2, 3)");
    }

    #[test]
    fn argument_list_should_parse_whitespace() {
        // arrange
        let input = "(2, 3)";

        // act
        let (next_input, res) = argument_list(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "(2, 3)");
    }
}
