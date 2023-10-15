//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-string-concat-expr

use nom::{bytes::complete::tag, multi::many0, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::sequence_expressions::constructing_sequences::{range_expr, RangeExpr};

pub fn string_concat_expr(input: &str) -> Res<&str, StringConcatExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-StringConcatExpr

    tuple((range_expr, many0(tuple((tag("||"), range_expr)))))(input).map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, StringConcatExpr { expr: res.0, items })
    })
}

pub struct StringConcatExpr {
    pub expr: RangeExpr,
    pub items: Vec<RangeExpr>,
}
