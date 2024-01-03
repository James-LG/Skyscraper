//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-inline-func

use std::fmt::Display;

use nom::{
    bytes::complete::tag, character::complete::char, combinator::opt, error::context, multi::many0,
};

use crate::xpath::grammar::{
    recipes::Res,
    types::{
        eq_name,
        sequence_type::{sequence_type, SequenceType},
        EQName,
    },
    whitespace_recipes::{sep, ws},
};

use super::enclosed_expressions::{enclosed_expr, EnclosedExpr};

pub fn inline_function_expr(input: &str) -> Res<&str, InlineFunctionExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-InlineFunctionExpr

    context(
        "inline_function_expr",
        ws((
            tag("function"),
            char('('),
            opt(param_list),
            char(')'),
            opt(sep((tag("as"), sequence_type))),
            function_body,
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            InlineFunctionExpr {
                param_list: res.2,
                sequence_type: res.4.map(|res| res.1),
                body: res.5,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct InlineFunctionExpr {
    pub param_list: Option<ParamList>,
    pub sequence_type: Option<SequenceType>,
    pub body: FunctionBody,
}

impl Display for InlineFunctionExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function(")?;
        if let Some(param_list) = &self.param_list {
            write!(f, "{}", param_list)?;
        }
        write!(f, ")")?;
        if let Some(sequence_type) = &self.sequence_type {
            write!(f, " as {}", sequence_type)?;
        }
        write!(f, " {}", self.body)?;

        Ok(())
    }
}

pub fn param_list(input: &str) -> Res<&str, ParamList> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ParamList
    context("param_list", ws((param, many0(ws((char(','), param))))))(input).map(
        |(next_input, res)| {
            let mut params = vec![res.0];
            let extras = res.1.into_iter().map(|res| res.1);
            params.extend(extras);
            (next_input, ParamList(params))
        },
    )
}

#[derive(PartialEq, Debug, Clone)]
pub struct ParamList(Vec<Param>);

impl Display for ParamList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, param) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }

        Ok(())
    }
}

pub fn param(input: &str) -> Res<&str, Param> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Param
    context(
        "param",
        sep((ws((char('$'), eq_name)), opt(type_declaration))),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            Param {
                name: res.0 .1,
                type_declaration: res.1,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct Param {
    pub name: EQName,
    pub type_declaration: Option<TypeDeclaration>,
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.name)?;
        if let Some(type_declaration) = &self.type_declaration {
            write!(f, " {}", type_declaration)?;
        }

        Ok(())
    }
}

pub fn type_declaration(input: &str) -> Res<&str, TypeDeclaration> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-TypeDeclaration
    context("type_declaration", sep((tag("as"), sequence_type)))(input)
        .map(|(next_input, res)| (next_input, TypeDeclaration(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct TypeDeclaration(pub SequenceType);

impl Display for TypeDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "as {}", self.0)
    }
}

pub fn function_body(input: &str) -> Res<&str, FunctionBody> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-FunctionBody
    context("function_body", enclosed_expr)(input)
        .map(|(next_input, res)| (next_input, FunctionBody(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionBody(pub EnclosedExpr);

impl Display for FunctionBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_function_expr_should_parse() {
        // arrange
        let input = "function($a as xs:double,$b as xs:double) as xs:double { $a*$b }";

        // act
        let (next_input, res) = inline_function_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "function($a as xs:double, $b as xs:double) as xs:double { $a * $b }"
        );
    }

    #[test]
    fn inline_function_expr_should_parse_whitespace() {
        // arrange
        let input = "function ( $a as xs:double, $b as xs:double ) as xs:double { $a * $b }";

        // act
        let (next_input, res) = inline_function_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "function($a as xs:double, $b as xs:double) as xs:double { $a * $b }"
        );
    }
}
