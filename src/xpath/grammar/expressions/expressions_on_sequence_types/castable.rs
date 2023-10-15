//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-castable

use nom::{bytes::complete::tag, combinator::opt, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::cast::{cast_expr, single_type, CastExpr, SingleType};

pub fn castable_expr(input: &str) -> Res<&str, CastableExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-CastableExpr

    tuple((
        cast_expr,
        opt(tuple((tag("castable"), tag("as"), single_type))),
    ))(input)
    .map(|(next_input, res)| {
        let cast_type = res.1.map(|res| res.2);
        (
            next_input,
            CastableExpr {
                expr: res.0,
                cast_type,
            },
        )
    })
}

pub struct CastableExpr {
    pub expr: CastExpr,
    pub cast_type: Option<SingleType>,
}
