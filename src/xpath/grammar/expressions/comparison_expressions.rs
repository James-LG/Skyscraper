//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-comparisons>

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace0, combinator::opt,
    error::context, sequence::tuple,
};

use ordered_float::OrderedFloat;

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            XpathItemTreeNode,
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
        let atomized1 = func_data(&result, &context.item_tree)?;
        let atomized2 = func_data(&second_result, &context.item_tree)?;

        let bool_value = match comparison.0 {
            ComparisonType::GeneralComp(comp) => {
                // XPath 3.1 §3.7.2: General comparisons are existentially quantified.
                // The result is true if any pair (a, b) from atomized1 x atomized2
                // satisfies the corresponding value comparison.
                if atomized1.is_empty() || atomized2.is_empty() {
                    false
                } else {
                    let mut found = false;
                    for a in atomized1.iter() {
                        for b in atomized2.iter() {
                            if comp.is_match(a, b) {
                                found = true;
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                    found
                }
            }
            ComparisonType::ValueComp(comp) => {
                // XPath 3.1 §3.7.1: Value comparisons require singleton operands.
                if atomized1.is_empty() || atomized2.is_empty() {
                    return Ok(XpathItemSet::new());
                }
                if atomized1.len() > 1 || atomized2.len() > 1 {
                    return Err(ExpressionApplyError {
                        msg: String::from("err:XPTY0004 The first operand of a value comparison is a sequence of length greater than one")
                    });
                }
                comp.is_match(&atomized1[0], &atomized2[0])
            }
            ComparisonType::NodeComp(comp) => {
                // Node comparisons require both operands to be single nodes.
                if result.is_empty() || second_result.is_empty() {
                    return Ok(XpathItemSet::new());
                }
                if result.len() > 1 || second_result.len() > 1 {
                    return Err(ExpressionApplyError {
                        msg: String::from("err:XPTY0004 Node comparison requires singleton node operands"),
                    });
                }
                let node1 = match &result[0] {
                    XpathItem::Node(n) => n,
                    _ => {
                        return Err(ExpressionApplyError {
                            msg: String::from(
                                "err:XPTY0004 Node comparison requires node operands",
                            ),
                        })
                    }
                };
                let node2 = match &second_result[0] {
                    XpathItem::Node(n) => n,
                    _ => {
                        return Err(ExpressionApplyError {
                            msg: String::from(
                                "err:XPTY0004 Node comparison requires node operands",
                            ),
                        })
                    }
                };
                comp.is_match(node1, node2)
            }
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

impl ValueComp {
    pub(crate) fn is_match(&self, first: &AnyAtomicType, second: &AnyAtomicType) -> bool {
        let (eq, lt, gt) = match self {
            ValueComp::Equal => (true, false, false),
            ValueComp::NotEqual => (false, true, true),
            ValueComp::LessThan => (false, true, false),
            ValueComp::LessThanEqualTo => (true, true, false),
            ValueComp::GreaterThan => (false, false, true),
            ValueComp::GreaterThanEqualTo => (true, false, true),
        };
        compare_atomic(first, second, eq, lt, gt)
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

    // Multi-character operators must be tried before their single-character
    // prefixes (e.g. "<=" before "<") so the shorter match doesn't greedily
    // consume part of the longer token.
    context(
        "general_comp",
        alt((
            not_equal,
            less_than_equal_to,
            greater_than_equal_to,
            equal,
            less_than,
            greater_than,
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
    pub(crate) fn is_match(&self, first: &AnyAtomicType, second: &AnyAtomicType) -> bool {
        let (eq, lt, gt) = match self {
            GeneralComp::Equal => (true, false, false),
            GeneralComp::NotEqual => (false, true, true),
            GeneralComp::LessThan => (false, true, false),
            GeneralComp::LessThanEqualTo => (true, true, false),
            GeneralComp::GreaterThan => (false, false, true),
            GeneralComp::GreaterThanEqualTo => (true, false, true),
        };
        compare_atomic(first, second, eq, lt, gt)
    }
}

/// Compare two atomic values, applying type coercion when needed.
///
/// The `eq`, `lt`, `gt` flags indicate which orderings are accepted — e.g.
/// `(true, true, false)` means "less-than-or-equal".  Coercion is applied
/// first via [`coerce_for_comparison`]; then `partial_cmp` (or `PartialEq`
/// for pure equality/inequality) determines the result.  Returns `false`
/// for incompatible types that cannot be ordered.
fn compare_atomic(
    first: &AnyAtomicType,
    second: &AnyAtomicType,
    eq: bool,
    lt: bool,
    gt: bool,
) -> bool {
    // Ordering comparisons (<, >, <=, >=) cast string operands to double per
    // XPath 3.1 §3.7.  Equality (=) and inequality (!=) compare strings as strings.
    let is_ordering = (lt || gt) && !(lt && gt && !eq);

    let (a, b);
    let (lhs, rhs) = if let Some(coerced) = coerce_for_comparison(first, second, is_ordering) {
        a = coerced.0;
        b = coerced.1;
        (&a, &b)
    } else {
        (first, second)
    };

    // Per IEEE 754 / XPath spec: any comparison involving NaN returns false,
    // except `ne` (not-equal) which returns true. OrderedFloat violates this
    // by making NaN == NaN, so we must guard explicitly.
    let either_nan = matches!(
        lhs,
        AnyAtomicType::Float(f) if f.is_nan()
    ) || matches!(
        lhs,
        AnyAtomicType::Double(d) if d.is_nan()
    ) || matches!(
        rhs,
        AnyAtomicType::Float(f) if f.is_nan()
    ) || matches!(
        rhs,
        AnyAtomicType::Double(d) if d.is_nan()
    );

    if either_nan {
        // `ne` is encoded as (!eq, lt, gt); for NaN ne anything, result is true.
        // All other comparisons involving NaN return false.
        return !eq && lt && gt;
    }

    // Pure equality / inequality can use PartialEq directly, which works
    // across all variant combinations (different variants → not equal).
    if eq && !lt && !gt {
        return lhs == rhs;
    }
    if !eq && lt && gt {
        return lhs != rhs;
    }

    // Ordering comparisons use partial_cmp. Returns false for incompatible
    // types (partial_cmp returns None).
    match lhs.partial_cmp(rhs) {
        Some(std::cmp::Ordering::Equal) => eq,
        Some(std::cmp::Ordering::Less) => lt,
        Some(std::cmp::Ordering::Greater) => gt,
        None => false,
    }
}

/// Coerce a pair of atomic values for comparison per XPath 3.1 §3.7.
///
/// - String (xs:untypedAtomic) vs numeric → cast string to xs:double, promote
///   numeric operand to xs:double.
/// - Mixed numeric types (e.g. Integer vs Double) → promote both to xs:double.
/// - Otherwise, return `None` (no coercion needed, compare directly).
fn coerce_for_comparison(
    first: &AnyAtomicType,
    second: &AnyAtomicType,
    is_ordering: bool,
) -> Option<(AnyAtomicType, AnyAtomicType)> {
    fn is_numeric(v: &AnyAtomicType) -> bool {
        matches!(
            v,
            AnyAtomicType::Integer(_) | AnyAtomicType::Float(_) | AnyAtomicType::Double(_)
        )
    }

    fn to_double(v: &AnyAtomicType) -> AnyAtomicType {
        match v {
            AnyAtomicType::Integer(i) => {
                AnyAtomicType::Double(OrderedFloat(*i as f64))
            }
            AnyAtomicType::Float(f) => {
                AnyAtomicType::Double(OrderedFloat(f.0 as f64))
            }
            AnyAtomicType::Double(_) => v.clone(),
            _ => unreachable!("to_double called with non-numeric value"),
        }
    }

    // Per XPath 3.1, general comparisons cast untypedAtomic to the type of the
    // other operand. A failed cast to xs:double produces NaN (matching fn:number
    // semantics for implicit casts), unlike explicit xs:double() which raises
    // FORG0001. This is intentionally different from the arithmetic error path.
    fn string_to_double(s: &str) -> AnyAtomicType {
        let d = s.trim().parse::<f64>().unwrap_or(f64::NAN);
        AnyAtomicType::Double(OrderedFloat(d))
    }

    match (first, second) {
        // String (untyped) vs numeric: cast string to double, promote numeric to double.
        (AnyAtomicType::String(s), other) if is_numeric(other) => {
            Some((string_to_double(s), to_double(other)))
        }
        (other, AnyAtomicType::String(s)) if is_numeric(other) => {
            Some((to_double(other), string_to_double(s)))
        }
        // String vs String ordering: cast both to double per XPath 3.1 §3.7.2.
        (AnyAtomicType::String(s1), AnyAtomicType::String(s2)) if is_ordering => {
            Some((string_to_double(s1), string_to_double(s2)))
        }
        // Mixed numeric types: promote both to double.
        (a, b)
            if is_numeric(a)
                && is_numeric(b)
                && std::mem::discriminant(a) != std::mem::discriminant(b) =>
        {
            Some((to_double(a), to_double(b)))
        }
        // Same type or non-numeric: no coercion needed.
        _ => None,
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

impl NodeComp {
    pub(crate) fn is_match(
        &self,
        first: &XpathItemTreeNode,
        second: &XpathItemTreeNode,
    ) -> bool {
        match (first.node_id(), second.node_id()) {
            (Some(id1), Some(id2)) => match self {
                NodeComp::Is => id1 == id2,
                NodeComp::Precedes => id1 < id2,
                NodeComp::Follows => id1 > id2,
            },
            // If either operand has no node_id, the comparison is false.
            _ => false,
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
