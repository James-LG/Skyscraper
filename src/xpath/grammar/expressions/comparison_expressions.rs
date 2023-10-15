//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-comparisons

use nom::{branch::alt, bytes::complete::tag, combinator::opt, sequence::tuple};

use crate::xpath::grammar::{
    expressions::string_concat_expressions::string_concat_expr, recipes::Res,
};

use super::string_concat_expressions::StringConcatExpr;

pub fn comparison_expr(input: &str) -> Res<&str, ComparisonExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ComparisonExpr

    fn value_comp_map(input: &str) -> Res<&str, ComparisonType> {
        value_comp(input).map(|(next_input, res)| (next_input, ComparisonType::ValueComp(res)))
    }

    fn general_comp_map(input: &str) -> Res<&str, ComparisonType> {
        general_comp(input).map(|(next_input, res)| (next_input, ComparisonType::GeneralComp(res)))
    }

    fn node_comp_map(input: &str) -> Res<&str, ComparisonType> {
        node_comp(input).map(|(next_input, res)| (next_input, ComparisonType::NodeComp(res)))
    }

    tuple((
        string_concat_expr,
        opt(tuple((
            alt((value_comp_map, general_comp_map, node_comp_map)),
            string_concat_expr,
        ))),
    ))(input)
    .map(|(next_input, res)| {
        let comparison = res.1.map(|res| ComparisonExprPair(res.0, res.1));
        (
            next_input,
            ComparisonExpr {
                expr: res.0,
                comparison,
            },
        )
    })
}

pub struct ComparisonExpr {
    pub expr: StringConcatExpr,
    pub comparison: Option<ComparisonExprPair>,
}

pub struct ComparisonExprPair(pub ComparisonType, pub StringConcatExpr);

pub enum ComparisonType {
    ValueComp(ValueComp),
    GeneralComp(GeneralComp),
    NodeComp(NodeComp),
}

fn value_comp(input: &str) -> Res<&str, ValueComp> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ValueComp

    fn equal(input: &str) -> Res<&str, ValueComp> {
        tag("eq")(input).map(|(next_input, _res)| (next_input, ValueComp::Equal))
    }

    fn not_equal(input: &str) -> Res<&str, ValueComp> {
        tag("ne")(input).map(|(next_input, _res)| (next_input, ValueComp::NotEqual))
    }

    fn less_than(input: &str) -> Res<&str, ValueComp> {
        tag("lt")(input).map(|(next_input, _res)| (next_input, ValueComp::LessThan))
    }

    fn less_than_equal_to(input: &str) -> Res<&str, ValueComp> {
        tag("le")(input).map(|(next_input, _res)| (next_input, ValueComp::LessThanEqualTo))
    }

    fn greater_than(input: &str) -> Res<&str, ValueComp> {
        tag("gt")(input).map(|(next_input, _res)| (next_input, ValueComp::GreaterThan))
    }

    fn greater_than_equal_to(input: &str) -> Res<&str, ValueComp> {
        tag("ge")(input).map(|(next_input, _res)| (next_input, ValueComp::GreaterThanEqualTo))
    }

    alt((
        equal,
        not_equal,
        less_than,
        less_than_equal_to,
        greater_than,
        greater_than_equal_to,
    ))(input)
}

pub enum ValueComp {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
}

fn general_comp(input: &str) -> Res<&str, GeneralComp> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-GeneralComp

    fn equal(input: &str) -> Res<&str, GeneralComp> {
        tag("=")(input).map(|(next_input, _res)| (next_input, GeneralComp::Equal))
    }

    fn not_equal(input: &str) -> Res<&str, GeneralComp> {
        tag("!=")(input).map(|(next_input, _res)| (next_input, GeneralComp::NotEqual))
    }

    fn less_than(input: &str) -> Res<&str, GeneralComp> {
        tag("<")(input).map(|(next_input, _res)| (next_input, GeneralComp::LessThan))
    }

    fn less_than_equal_to(input: &str) -> Res<&str, GeneralComp> {
        tag("<=")(input).map(|(next_input, _res)| (next_input, GeneralComp::LessThanEqualTo))
    }

    fn greater_than(input: &str) -> Res<&str, GeneralComp> {
        tag(">")(input).map(|(next_input, _res)| (next_input, GeneralComp::GreaterThan))
    }

    fn greater_than_equal_to(input: &str) -> Res<&str, GeneralComp> {
        tag(">=")(input).map(|(next_input, _res)| (next_input, GeneralComp::GreaterThanEqualTo))
    }

    alt((
        equal,
        not_equal,
        less_than,
        less_than_equal_to,
        greater_than,
        greater_than_equal_to,
    ))(input)
}

pub enum GeneralComp {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
}

fn node_comp(input: &str) -> Res<&str, NodeComp> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NodeComp

    fn is(input: &str) -> Res<&str, NodeComp> {
        tag("is")(input).map(|(next_input, _res)| (next_input, NodeComp::Is))
    }

    fn precedes(input: &str) -> Res<&str, NodeComp> {
        tag("<<")(input).map(|(next_input, _res)| (next_input, NodeComp::Precedes))
    }

    fn follows(input: &str) -> Res<&str, NodeComp> {
        tag(">>")(input).map(|(next_input, _res)| (next_input, NodeComp::Follows))
    }

    alt((is, precedes, follows))(input)
}

pub enum NodeComp {
    Is,
    Precedes,
    Follows,
}
