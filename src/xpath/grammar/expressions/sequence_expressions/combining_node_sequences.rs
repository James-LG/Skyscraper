//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#combining_seq

use nom::{branch::alt, bytes::complete::tag, multi::many0, sequence::tuple};

use crate::xpath::grammar::{
    expressions::expressions_on_sequence_types::instance_of::{instanceof_expr, InstanceofExpr},
    recipes::Res,
};

pub fn union_expr(input: &str) -> Res<&str, UnionExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-UnionExpr

    tuple((
        intersect_except_expr,
        many0(tuple((
            alt((tag("union"), tag("|"))),
            intersect_except_expr,
        ))),
    ))(input)
    .map(|(next_input, res)| {
        let extras = res.1.into_iter().map(|res| res.1).collect();
        (
            next_input,
            UnionExpr {
                expr: res.0,
                extras,
            },
        )
    })
}

pub struct UnionExpr {
    pub expr: IntersectExceptExpr,
    pub extras: Vec<IntersectExceptExpr>,
}

fn intersect_except_expr(input: &str) -> Res<&str, IntersectExceptExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-IntersectExceptExpr

    fn intersect(input: &str) -> Res<&str, IntersectExceptType> {
        tag("intersect")(input)
            .map(|(next_input, _res)| (next_input, IntersectExceptType::Intersect))
    }

    fn except(input: &str) -> Res<&str, IntersectExceptType> {
        tag("except")(input).map(|(next_input, _res)| (next_input, IntersectExceptType::Except))
    }

    tuple((
        instanceof_expr,
        many0(tuple((alt((intersect, except)), instanceof_expr))),
    ))(input)
    .map(|(next_input, res)| {
        let items = res
            .1
            .into_iter()
            .map(|res| IntersectExceptPair(res.0, res.1))
            .collect();
        (next_input, IntersectExceptExpr { expr: res.0, items })
    })
}

pub struct IntersectExceptExpr {
    pub expr: InstanceofExpr,
    pub items: Vec<IntersectExceptPair>,
}

pub enum IntersectExceptType {
    Intersect,
    Except,
}

pub struct IntersectExceptPair(pub IntersectExceptType, pub InstanceofExpr);
