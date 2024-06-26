//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-comparisons>

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace0, combinator::opt,
    error::context, sequence::tuple,
};

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            expressions::string_concat_expressions::string_concat_expr,
            recipes::Res,
            terminal_symbols::symbol_separator,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

use super::{
    primary_expressions::static_function_calls::func_data,
    string_concat_expressions::StringConcatExpr,
};

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

    context(
        "comparison_expr",
        tuple((
            string_concat_expr,
            opt(tuple((
                alt((value_comp_map, node_comp_map, general_comp_map)),
                string_concat_expr,
            ))),
        )),
    )(input)
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

#[derive(PartialEq, Debug, Clone)]
pub struct ComparisonExpr {
    pub expr: StringConcatExpr,
    pub comparison: Option<ComparisonExprPair>,
}

impl Display for ComparisonExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;

        if let Some(x) = &self.comparison {
            write!(f, "{}", x)?;
        }

        Ok(())
    }
}

impl ComparisonExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        let comparison = if let Some(comparison) = &self.comparison {
            comparison
        } else {
            return Ok(result);
        };

        // Otherwise, do the comparison op.
        // Get the second expression result.
        let second_result = comparison.1.eval(context)?;

        // Atomize both results.
        let atomized1 = func_data(&result, &context.item_tree);
        let atomized2 = func_data(&second_result, &context.item_tree);

        // Do some type checking first.

        // If the either atomized set is an empty sequence,
        // the result of the value comparison is an empty sequence.
        if atomized1.is_empty() || atomized2.is_empty() {
            return Ok(XpathItemSet::new());
        }

        // If the either atomized set is a sequence of length greater than one,
        // a type error is raised.
        if atomized1.len() > 1 || atomized2.len() > 1 {
            return Err(ExpressionApplyError {
                msg: String::from("err:XPTY0004 The first operand of a value comparison is a sequence of length greater than one")
            });
        }

        let bool_value = match comparison.0 {
            ComparisonType::ValueComp(_) => todo!("ComparisonType::ValueComp"),
            ComparisonType::GeneralComp(comp) => comp.is_match(&atomized1[0], &atomized2[0]),
            ComparisonType::NodeComp(_) => todo!("ComparisonType::NodeComp"),
        };

        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Boolean(bool_value),
        )])
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ComparisonExprPair(pub ComparisonType, pub StringConcatExpr);

impl Display for ComparisonExprPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ComparisonType {
    ValueComp(ValueComp),
    GeneralComp(GeneralComp),
    NodeComp(NodeComp),
}

impl Display for ComparisonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonType::ValueComp(x) => write!(f, "{}", x),
            ComparisonType::GeneralComp(x) => write!(f, "{}", x),
            ComparisonType::NodeComp(x) => write!(f, "{}", x),
        }
    }
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

    context(
        "value_comp",
        tuple((
            symbol_separator,
            alt((
                equal,
                not_equal,
                less_than,
                less_than_equal_to,
                greater_than,
                greater_than_equal_to,
            )),
            symbol_separator,
        )),
    )(input)
    .map(|(next_input, res)| (next_input, res.1))
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ValueComp {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
}

impl Display for ValueComp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueComp::Equal => write!(f, " eq "),
            ValueComp::NotEqual => write!(f, " ne "),
            ValueComp::LessThan => write!(f, " lt "),
            ValueComp::LessThanEqualTo => write!(f, " le "),
            ValueComp::GreaterThan => write!(f, " gt "),
            ValueComp::GreaterThanEqualTo => write!(f, " ge "),
        }
    }
}

fn general_comp(input: &str) -> Res<&str, GeneralComp> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-GeneralComp

    fn equal(input: &str) -> Res<&str, GeneralComp> {
        tuple((multispace0, tag("="), multispace0))(input)
            .map(|(next_input, _res)| (next_input, GeneralComp::Equal))
    }

    fn not_equal(input: &str) -> Res<&str, GeneralComp> {
        tuple((multispace0, tag("!="), multispace0))(input)
            .map(|(next_input, _res)| (next_input, GeneralComp::NotEqual))
    }

    fn less_than(input: &str) -> Res<&str, GeneralComp> {
        tuple((multispace0, tag("<"), multispace0))(input)
            .map(|(next_input, _res)| (next_input, GeneralComp::LessThan))
    }

    fn less_than_equal_to(input: &str) -> Res<&str, GeneralComp> {
        tuple((multispace0, tag("<="), multispace0))(input)
            .map(|(next_input, _res)| (next_input, GeneralComp::LessThanEqualTo))
    }

    fn greater_than(input: &str) -> Res<&str, GeneralComp> {
        tuple((multispace0, tag(">"), multispace0))(input)
            .map(|(next_input, _res)| (next_input, GeneralComp::GreaterThan))
    }

    fn greater_than_equal_to(input: &str) -> Res<&str, GeneralComp> {
        tuple((multispace0, tag(">="), multispace0))(input)
            .map(|(next_input, _res)| (next_input, GeneralComp::GreaterThanEqualTo))
    }

    context(
        "general_comp",
        alt((
            equal,
            not_equal,
            less_than,
            less_than_equal_to,
            greater_than,
            greater_than_equal_to,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GeneralComp {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
}

impl Display for GeneralComp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneralComp::Equal => write!(f, "="),
            GeneralComp::NotEqual => write!(f, "!="),
            GeneralComp::LessThan => write!(f, "<"),
            GeneralComp::LessThanEqualTo => write!(f, "<="),
            GeneralComp::GreaterThan => write!(f, ">"),
            GeneralComp::GreaterThanEqualTo => write!(f, ">="),
        }
    }
}

impl GeneralComp {
    pub(crate) fn is_match<'tree>(&self, first: &AnyAtomicType, second: &AnyAtomicType) -> bool {
        match self {
            GeneralComp::Equal => first == second,
            GeneralComp::NotEqual => first != second,
            GeneralComp::LessThan => first < second,
            GeneralComp::LessThanEqualTo => first <= second,
            GeneralComp::GreaterThan => first > second,
            GeneralComp::GreaterThanEqualTo => first >= second,
        }
    }
}

fn node_comp(input: &str) -> Res<&str, NodeComp> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NodeComp

    fn is(input: &str) -> Res<&str, NodeComp> {
        tuple((symbol_separator, tag("is"), symbol_separator))(input)
            .map(|(next_input, _res)| (next_input, NodeComp::Is))
    }

    fn precedes(input: &str) -> Res<&str, NodeComp> {
        tuple((multispace0, tag("<<"), multispace0))(input)
            .map(|(next_input, _res)| (next_input, NodeComp::Precedes))
    }

    fn follows(input: &str) -> Res<&str, NodeComp> {
        tuple((multispace0, tag(">>"), multispace0))(input)
            .map(|(next_input, _res)| (next_input, NodeComp::Follows))
    }

    context("node_comp", alt((is, precedes, follows)))(input)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum NodeComp {
    Is,
    Precedes,
    Follows,
}

impl Display for NodeComp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeComp::Is => write!(f, " is "),
            NodeComp::Precedes => write!(f, "<<"),
            NodeComp::Follows => write!(f, ">>"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn comparison_expr_should_parse() {
        // arrange
        let input = r#"$book1/author eq "Kennedy""#;

        // act
        let (next_input, res) = comparison_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"$book1/author eq "Kennedy""#);
    }

    #[test]
    fn comparison_expr_should_parse_node_comp_precedes() {
        // arrange
        let input = r#"$book1/author<<"Kennedy""#;

        // act
        let (next_input, res) = comparison_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"$book1/author<<"Kennedy""#);
    }

    #[test]
    fn comparison_expr_should_parse_node_comp_precedes_whitespace() {
        // arrange
        let input = r#"$book1/author << "Kennedy""#;

        // act
        let (next_input, res) = comparison_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"$book1/author<<"Kennedy""#);
    }

    #[test]
    fn comparison_expr_should_parse_node_comp_is() {
        // arrange
        let input = r#"$book1/author is "Kennedy""#;

        // act
        let (next_input, res) = comparison_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"$book1/author is "Kennedy""#);
    }

    #[test]
    fn comparison_expr_should_parse_general_comp() {
        // arrange
        let input = r#"$book1/author="Kennedy""#;

        // act
        let (next_input, res) = comparison_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"$book1/author="Kennedy""#);
    }

    #[test]
    fn comparison_expr_should_parse_general_comp_whitespace() {
        // arrange
        let input = r#"$book1/author = "Kennedy""#;

        // act
        let (next_input, res) = comparison_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"$book1/author="Kennedy""#);
    }
}
