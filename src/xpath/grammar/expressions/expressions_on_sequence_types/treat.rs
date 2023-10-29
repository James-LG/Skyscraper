//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-treat

use std::fmt::Display;

use nom::{bytes::complete::tag, combinator::opt, sequence::tuple};

use crate::xpath::grammar::{
    recipes::Res,
    types::sequence_type::{sequence_type, SequenceType},
};

use super::castable::{castable_expr, CastableExpr};

pub fn treat_expr(input: &str) -> Res<&str, TreatExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-TreatExpr

    tuple((
        castable_expr,
        opt(tuple((tag("treat"), tag("as"), sequence_type))),
    ))(input)
    .map(|(next_input, res)| {
        let treat_type = res.1.map(|res| res.2);
        (
            next_input,
            TreatExpr {
                expr: res.0,
                treat_type,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct TreatExpr {
    pub expr: CastableExpr,
    pub treat_type: Option<SequenceType>,
}

impl Display for TreatExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(x) = &self.treat_type {
            write!(f, " treat as {}", x)?;
        }

        Ok(())
    }
}
