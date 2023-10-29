//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-inline-func

use nom::{
    bytes::complete::tag, character::complete::char, combinator::opt, multi::many0, sequence::tuple,
};

use crate::xpath::grammar::{
    recipes::Res,
    types::{
        eq_name,
        sequence_type::{sequence_type, SequenceType},
        EQName,
    },
};

use super::enclosed_expressions::{enclosed_expr, EnclosedExpr};

pub fn inline_function_expr(input: &str) -> Res<&str, InlineFunctionExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-InlineFunctionExpr

    tuple((
        tag("function"),
        char('('),
        opt(param_list),
        char(')'),
        opt(tuple((tag("as"), sequence_type))),
        function_body,
    ))(input)
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

#[derive(PartialEq, Debug)]
pub struct InlineFunctionExpr {
    pub param_list: Option<ParamList>,
    pub sequence_type: Option<SequenceType>,
    pub body: FunctionBody,
}

pub fn param_list(input: &str) -> Res<&str, ParamList> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ParamList
    tuple((param, many0(tuple((char(','), param)))))(input).map(|(next_input, res)| {
        let mut params = vec![res.0];
        let extras = res.1.into_iter().map(|res| res.1);
        params.extend(extras);
        (next_input, ParamList(params))
    })
}

#[derive(PartialEq, Debug)]
pub struct ParamList(Vec<Param>);

pub fn param(input: &str) -> Res<&str, Param> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Param
    tuple((char('$'), eq_name, opt(type_declaration)))(input).map(|(next_input, res)| {
        (
            next_input,
            Param {
                name: res.1,
                type_declaration: res.2,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct Param {
    pub name: EQName,
    pub type_declaration: Option<TypeDeclaration>,
}

pub fn type_declaration(input: &str) -> Res<&str, TypeDeclaration> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-TypeDeclaration

    tuple((tag("as"), sequence_type))(input)
        .map(|(next_input, res)| (next_input, TypeDeclaration(res.1)))
}

#[derive(PartialEq, Debug)]
pub struct TypeDeclaration(pub SequenceType);

pub fn function_body(input: &str) -> Res<&str, FunctionBody> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-FunctionBody
    enclosed_expr(input).map(|(next_input, res)| (next_input, FunctionBody(res)))
}

#[derive(PartialEq, Debug)]
pub struct FunctionBody(pub EnclosedExpr);
