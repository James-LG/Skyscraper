//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-map-operator

use nom::{character::complete::char, multi::many0, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::path_expressions::{path_expr, PathExpr};

pub fn simple_map_expr(input: &str) -> Res<&str, SimpleMapExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SimpleMapExpr

    tuple((path_expr, many0(tuple((char('!'), path_expr)))))(input).map(|(next_input, res)| {
        let expr = res.0;
        let extras = res.1.into_iter().map(|res| res.1).collect();
        (next_input, SimpleMapExpr { expr, extras })
    })
}

pub struct SimpleMapExpr {
    pub expr: PathExpr,
    pub extras: Vec<PathExpr>,
}
