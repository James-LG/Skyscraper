//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#axes

use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

pub fn forward_axis(input: &str) -> Res<&str, ForwardAxis> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ForwardAxis

    fn child(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("child"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::Child))
    }

    fn descendant(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("descendant"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::Descendant))
    }

    fn attribute(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("attribute"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::Attribute))
    }

    fn self_axis(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("self"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::SelfAxis))
    }

    fn descendant_or_self(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("descendant-or-self"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::DescendantOrSelf))
    }

    fn following_sibling(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("following-sibling"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::FollowingSibling))
    }

    fn following(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("descendant-or-self"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::Following))
    }

    fn namespace(input: &str) -> Res<&str, ForwardAxis> {
        tuple((tag("namespace"), tag("::")))(input)
            .map(|(next_input, _res)| (next_input, ForwardAxis::Namespace))
    }

    context(
        "forward_axis",
        alt((
            child,
            descendant,
            attribute,
            self_axis,
            descendant_or_self,
            following_sibling,
            following,
            namespace,
        )),
    )(input)
}

#[derive(PartialEq, Debug)]
pub enum ForwardAxis {
    Child,
    Descendant,
    Attribute,
    SelfAxis,
    DescendantOrSelf,
    FollowingSibling,
    Following,
    Namespace,
}

impl Display for ForwardAxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForwardAxis::Child => write!(f, "child::"),
            ForwardAxis::Descendant => write!(f, "descendant::"),
            ForwardAxis::Attribute => write!(f, "attribute::"),
            ForwardAxis::SelfAxis => write!(f, "self::"),
            ForwardAxis::DescendantOrSelf => write!(f, "descendant-or-self::"),
            ForwardAxis::FollowingSibling => write!(f, "following-sibling::"),
            ForwardAxis::Following => write!(f, "following::"),
            ForwardAxis::Namespace => write!(f, "namespace::"),
        }
    }
}

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

#[derive(PartialEq, Debug)]
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
