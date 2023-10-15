//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-cast

use nom::{bytes::complete::tag, character::complete::char, combinator::opt, sequence::tuple};

use crate::xpath::grammar::{
    expressions::arrow_operator::{arrow_expr, ArrowExpr},
    recipes::Res,
    types::{simple_type_name, SimpleTypeName},
};

pub fn cast_expr(input: &str) -> Res<&str, CastExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-CastExpr

    tuple((
        arrow_expr,
        opt(tuple((tag("cast"), tag("as"), single_type))),
    ))(input)
    .map(|(next_input, res)| {
        let cast = res.1.map(|res| res.2);
        (next_input, CastExpr { expr: res.0, cast })
    })
}

pub struct CastExpr {
    pub expr: ArrowExpr,
    pub cast: Option<SingleType>,
}

pub fn single_type(input: &str) -> Res<&str, SingleType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SingleType
    tuple((simple_type_name, opt(char('?'))))(input).map(|(next_input, res)| {
        (
            next_input,
            SingleType {
                type_name: res.0,
                has_question_mark: res.1.is_some(),
            },
        )
    })
}

pub struct SingleType {
    pub type_name: SimpleTypeName,
    pub has_question_mark: bool,
}
