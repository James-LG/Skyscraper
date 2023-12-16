//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-maps

use std::fmt::Display;

use nom::{
    bytes::complete::tag, character::complete::char, combinator::opt, error::context, multi::many0,
    sequence::tuple,
};

use crate::xpath::grammar::{
    expressions::{expr_single, ExprSingle},
    recipes::Res,
};

pub fn map_constructor(input: &str) -> Res<&str, MapConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapConstructor

    context(
        "map_constructor",
        tuple((
            tag("map"),
            char('{'),
            opt(tuple((
                map_constructor_entry,
                many0(tuple((char(','), map_constructor_entry))),
            ))),
            char('}'),
        )),
    )(input)
    .map(|(next_input, res)| {
        let mut entries = Vec::new();
        if let Some(res) = res.2 {
            entries.push(res.0);
            let extras = res.1.into_iter().map(|res| res.1);
            entries.extend(extras);
        }
        (next_input, MapConstructor { entries })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct MapConstructor {
    pub entries: Vec<MapConstructorEntry>,
}

impl Display for MapConstructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt MapConstructor")
    }
}

fn map_constructor_entry(input: &str) -> Res<&str, MapConstructorEntry> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapConstructorEntry
    context(
        "map_constructor_entry",
        tuple((map_key_expr, map_value_expr)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            MapConstructorEntry {
                key: res.0,
                value: res.1,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct MapConstructorEntry {
    pub key: ExprSingle,
    pub value: ExprSingle,
}

fn map_key_expr(input: &str) -> Res<&str, ExprSingle> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapKeyExpr
    context("map_key_expr", expr_single)(input)
}

fn map_value_expr(input: &str) -> Res<&str, ExprSingle> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapValueExpr
    context("map_value_expr", expr_single)(input)
}
