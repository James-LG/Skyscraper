use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

pub fn reverse_axis(input: &str) -> Res<&str, ReverseAxis> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ReverseStep

    fn parent_map(input: &str) -> Res<&str, ReverseAxis> {
        tuple((tag("parent"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ReverseAxis::Parent))
    }

    fn ancestor_map(input: &str) -> Res<&str, ReverseAxis> {
        tuple((tag("ancestor"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ReverseAxis::Ancestor))
    }

    fn preceding_sibling_map(input: &str) -> Res<&str, ReverseAxis> {
        tuple((tag("preceding-sibling"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ReverseAxis::PrecedingSibling))
    }

    fn preceding_map(input: &str) -> Res<&str, ReverseAxis> {
        tuple((tag("preceding"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ReverseAxis::Preceding))
    }

    fn ancestor_or_self_map(input: &str) -> Res<&str, ReverseAxis> {
        tuple((tag("ancestor-or-self"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ReverseAxis::AncestorOrSelf))
    }

    context(
        "reverse_axis",
        alt((
            parent_map,
            ancestor_map,
            preceding_sibling_map,
            preceding_map,
            ancestor_or_self_map,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ReverseAxis {
    Parent,
    Ancestor,
    PrecedingSibling,
    Preceding,
    AncestorOrSelf,
}

impl Display for ReverseAxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReverseAxis::Parent => write!(f, "parent::"),
            ReverseAxis::Ancestor => write!(f, "ancestor::"),
            ReverseAxis::PrecedingSibling => write!(f, "preceding-sibling::"),
            ReverseAxis::Preceding => write!(f, "preceding::"),
            ReverseAxis::AncestorOrSelf => write!(f, "ancestor-or-self::"),
        }
    }
}
