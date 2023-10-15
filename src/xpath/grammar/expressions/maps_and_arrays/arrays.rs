//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-arrays

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, multi::many0,
    sequence::tuple,
};

use crate::xpath::grammar::{
    expressions::{
        expr_single,
        primary_expressions::enclosed_expressions::{enclosed_expr, EnclosedExpr},
        ExprSingle,
    },
    recipes::Res,
};

pub fn array_constructor(input: &str) -> Res<&str, ArrayConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ArrayConstructor

    fn square_array_constructor_map(input: &str) -> Res<&str, ArrayConstructor> {
        square_array_constructor(input)
            .map(|(next_input, res)| (next_input, ArrayConstructor::SquareArrayConstructor(res)))
    }

    fn curly_array_constructor_map(input: &str) -> Res<&str, ArrayConstructor> {
        curly_array_constructor(input)
            .map(|(next_input, res)| (next_input, ArrayConstructor::CurlyArrayConstructor(res)))
    }

    alt((square_array_constructor_map, curly_array_constructor_map))(input)
}

pub enum ArrayConstructor {
    SquareArrayConstructor(SquareArrayConstructor),
    CurlyArrayConstructor(CurlyArrayConstructor),
}

fn square_array_constructor(input: &str) -> Res<&str, SquareArrayConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SquareArrayConstructor
    tuple((
        char('['),
        opt(tuple((expr_single, many0(tuple((char(','), expr_single)))))),
    ))(input)
    .map(|(next_input, res)| {
        let mut entries = Vec::new();
        if let Some(res) = res.1 {
            entries.push(res.0);
            let extras = res.1.into_iter().map(|res| res.1);
            entries.extend(extras);
        }
        (next_input, SquareArrayConstructor { entries })
    })
}

pub struct SquareArrayConstructor {
    pub entries: Vec<ExprSingle>,
}

fn curly_array_constructor(input: &str) -> Res<&str, CurlyArrayConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-CurlyArrayConstructor
    tuple((tag("array"), enclosed_expr))(input)
        .map(|(next_input, res)| (next_input, CurlyArrayConstructor(res.1)))
}

pub struct CurlyArrayConstructor(EnclosedExpr);
