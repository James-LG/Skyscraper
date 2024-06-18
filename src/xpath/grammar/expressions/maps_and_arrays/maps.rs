//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-maps>

use std::fmt::Display;

use nom::{
    bytes::complete::tag, character::complete::char, combinator::opt, error::context, multi::many0,
};

use crate::xpath::grammar::{
    expressions::{expr_single, ExprSingle},
    recipes::Res,
    whitespace_recipes::ws,
};

pub fn map_constructor(input: &str) -> Res<&str, MapConstructor> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapConstructor

    context(
        "map_constructor",
        ws((
            tag("map"),
            char('{'),
            opt(ws((
                map_constructor_entry,
                many0(ws((char(','), map_constructor_entry))),
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
        write!(f, "map {{")?;
        for entry in &self.entries {
            write!(f, " {}", entry)?;
        }
        write!(f, " }}")
    }
}

fn map_constructor_entry(input: &str) -> Res<&str, MapConstructorEntry> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapConstructorEntry
    context(
        "map_constructor_entry",
        ws((map_key_expr, char(':'), map_value_expr)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            MapConstructorEntry {
                key: res.0,
                value: res.2,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct MapConstructorEntry {
    pub key: ExprSingle,
    pub value: ExprSingle,
}

impl Display for MapConstructorEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

fn map_key_expr(input: &str) -> Res<&str, ExprSingle> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapKeyExpr
    context("map_key_expr", expr_single)(input)
}

fn map_value_expr(input: &str) -> Res<&str, ExprSingle> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MapValueExpr
    context("map_value_expr", expr_single)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_constructor_should_parse() {
        // arrange
        let input = "map { a: b }";

        // act
        let (next_input, res) = map_constructor(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn map_constructor_should_parse_no_whitespace() {
        // arrange
        let input = "map{a: b}";

        // act
        let (next_input, res) = map_constructor(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "map { a: b }");
    }
}
