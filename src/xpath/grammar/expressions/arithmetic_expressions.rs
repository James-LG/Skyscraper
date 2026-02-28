//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-arithmetic>

use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    error::context,
    multi::many0,
    sequence::tuple,
};

use ordered_float::OrderedFloat;

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            expressions::sequence_expressions::combining_node_sequences::union_expr,
            recipes::Res,
            terminal_symbols::symbol_separator,
            whitespace_recipes::ws,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

use super::{
    primary_expressions::static_function_calls::func_data,
    sequence_expressions::combining_node_sequences::UnionExpr,
    simple_map_operator::{simple_map_expr, SimpleMapExpr},
};

/// Convert an atomic value to f64 for arithmetic operations.
fn to_f64(val: &AnyAtomicType) -> Result<f64, ExpressionApplyError> {
    match val {
        AnyAtomicType::Integer(n) => Ok(*n as f64),
        AnyAtomicType::Float(n) => Ok(n.into_inner() as f64),
        AnyAtomicType::Double(n) => Ok(n.into_inner()),
        AnyAtomicType::String(s) => s.trim().parse::<f64>().map_err(|_| ExpressionApplyError {
            msg: format!("err:FORG0001 Cannot cast '{}' to xs:double", s),
        }),
        AnyAtomicType::Boolean(_) => Err(ExpressionApplyError {
            msg: String::from(
                "err:XPTY0004 Arithmetic operators are not defined for boolean values",
            ),
        }),
        AnyAtomicType::QName { .. } => Err(ExpressionApplyError {
            msg: String::from(
                "err:XPTY0004 Arithmetic operators are not defined for QName values",
            ),
        }),
    }
}

/// Atomize a result set to a single atomic value for arithmetic.
/// Returns `Ok(None)` for empty sequences, `Ok(Some(val))` for single values,
/// and `Err` for sequences with more than one item.
fn atomize_single<'tree>(
    result: &XpathItemSet<'tree>,
    context: &XpathExpressionContext<'tree>,
) -> Result<Option<AnyAtomicType>, ExpressionApplyError> {
    let atomized = func_data(result, &context.item_tree)?;
    if atomized.is_empty() {
        return Ok(None);
    }
    if atomized.len() > 1 {
        return Err(ExpressionApplyError {
            msg: String::from("err:XPTY0004 Arithmetic operand must be a single atomic value"),
        });
    }
    Ok(Some(atomized.into_iter().next().unwrap()))
}

pub fn additive_expr(input: &str) -> Res<&str, AdditiveExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AdditiveExpr

    fn plus(input: &str) -> Res<&str, AdditiveExprOperator> {
        ws((char('+'),))(input).map(|(next_input, _res)| (next_input, AdditiveExprOperator::Plus))
    }

    fn minus(input: &str) -> Res<&str, AdditiveExprOperator> {
        ws((char('-'),))(input).map(|(next_input, _res)| (next_input, AdditiveExprOperator::Minus))
    }

    context(
        "additive_expr",
        tuple((
            multiplicative_expr,
            many0(tuple((alt((plus, minus)), multiplicative_expr))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let items = res
            .1
            .into_iter()
            .map(|res| AdditiveExprPair(res.0, res.1))
            .collect();
        (next_input, AdditiveExpr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct AdditiveExpr {
    pub expr: MultiplicativeExpr,
    pub items: Vec<AdditiveExprPair>,
}

impl Display for AdditiveExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " {}", x)?
        }

        Ok(())
    }
}

impl AdditiveExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Atomize the first operand.
        let mut left = match atomize_single(&result, context)? {
            Some(val) => val,
            None => return Ok(XpathItemSet::new()),
        };

        for pair in &self.items {
            let right_result = pair.1.eval(context)?;
            let right = match atomize_single(&right_result, context)? {
                Some(val) => val,
                None => return Ok(XpathItemSet::new()),
            };

            left = match (&left, &right) {
                (AnyAtomicType::Integer(a), AnyAtomicType::Integer(b)) => match pair.0 {
                    AdditiveExprOperator::Plus => AnyAtomicType::Integer(
                        a.checked_add(*b).ok_or_else(|| ExpressionApplyError {
                            msg: String::from("err:FOAR0002 Integer overflow"),
                        })?,
                    ),
                    AdditiveExprOperator::Minus => AnyAtomicType::Integer(
                        a.checked_sub(*b).ok_or_else(|| ExpressionApplyError {
                            msg: String::from("err:FOAR0002 Integer overflow"),
                        })?,
                    ),
                },
                _ => {
                    let a = to_f64(&left)?;
                    let b = to_f64(&right)?;
                    let res = match pair.0 {
                        AdditiveExprOperator::Plus => a + b,
                        AdditiveExprOperator::Minus => a - b,
                    };
                    AnyAtomicType::Double(OrderedFloat(res))
                }
            };
        }

        Ok(xpath_item_set![XpathItem::AnyAtomicType(left)])
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct AdditiveExprPair(pub AdditiveExprOperator, pub MultiplicativeExpr);

impl Display for AdditiveExprPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AdditiveExprOperator {
    Plus,
    Minus,
}

impl Display for AdditiveExprOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdditiveExprOperator::Minus => write!(f, "-"),
            AdditiveExprOperator::Plus => write!(f, "+"),
        }
    }
}

fn multiplicative_expr(input: &str) -> Res<&str, MultiplicativeExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-MultiplicativeExpr

    fn star(input: &str) -> Res<&str, MultiplicativeExprOperator> {
        tuple((multispace0, char('*'), multispace0))(input)
            .map(|(next_input, _res)| (next_input, MultiplicativeExprOperator::Star))
    }

    fn div(input: &str) -> Res<&str, MultiplicativeExprOperator> {
        tuple((symbol_separator, tag("div"), symbol_separator))(input)
            .map(|(next_input, _res)| (next_input, MultiplicativeExprOperator::Div))
    }

    fn integer_div(input: &str) -> Res<&str, MultiplicativeExprOperator> {
        tuple((symbol_separator, tag("idiv"), symbol_separator))(input)
            .map(|(next_input, _res)| (next_input, MultiplicativeExprOperator::IntegerDiv))
    }

    fn modulus(input: &str) -> Res<&str, MultiplicativeExprOperator> {
        tuple((symbol_separator, tag("mod"), symbol_separator))(input)
            .map(|(next_input, _res)| (next_input, MultiplicativeExprOperator::Modulus))
    }

    context(
        "multiplicative_expr",
        tuple((
            union_expr,
            many0(tuple((alt((star, div, integer_div, modulus)), union_expr))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let items = res
            .1
            .into_iter()
            .map(|res| MultiplicativeExprPair(res.0, res.1))
            .collect();
        (next_input, MultiplicativeExpr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct MultiplicativeExpr {
    pub expr: UnionExpr,
    pub items: Vec<MultiplicativeExprPair>,
}

impl Display for MultiplicativeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " {}", x)?
        }

        Ok(())
    }
}

impl MultiplicativeExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Atomize the first operand.
        let mut left = match atomize_single(&result, context)? {
            Some(val) => val,
            None => return Ok(XpathItemSet::new()),
        };

        for pair in &self.items {
            let right_result = pair.1.eval(context)?;
            let right = match atomize_single(&right_result, context)? {
                Some(val) => val,
                None => return Ok(XpathItemSet::new()),
            };

            left = match pair.0 {
                MultiplicativeExprOperator::Star => match (&left, &right) {
                    (AnyAtomicType::Integer(a), AnyAtomicType::Integer(b)) => {
                        AnyAtomicType::Integer(
                            a.checked_mul(*b).ok_or_else(|| ExpressionApplyError {
                                msg: String::from("err:FOAR0002 Integer overflow"),
                            })?,
                        )
                    }
                    _ => {
                        let a = to_f64(&left)?;
                        let b = to_f64(&right)?;
                        AnyAtomicType::Double(OrderedFloat(a * b))
                    }
                },
                MultiplicativeExprOperator::Div => {
                    match (&left, &right) {
                        (AnyAtomicType::Integer(_), AnyAtomicType::Integer(b)) if *b == 0 => {
                            return Err(ExpressionApplyError {
                                msg: String::from("err:FOAR0002 Division by zero"),
                            });
                        }
                        _ => {}
                    }
                    let a = to_f64(&left)?;
                    let b = to_f64(&right)?;
                    AnyAtomicType::Double(OrderedFloat(a / b))
                }
                MultiplicativeExprOperator::IntegerDiv => {
                    match (&left, &right) {
                        (AnyAtomicType::Integer(a), AnyAtomicType::Integer(b)) => {
                            if *b == 0 {
                                return Err(ExpressionApplyError {
                                    msg: String::from("err:FOAR0002 Division by zero"),
                                });
                            }
                            AnyAtomicType::Integer(a / b)
                        }
                        _ => {
                            let a = to_f64(&left)?;
                            let b = to_f64(&right)?;
                            if b == 0.0 {
                                return Err(ExpressionApplyError {
                                    msg: String::from("err:FOAR0002 Division by zero"),
                                });
                            }
                            AnyAtomicType::Integer((a / b).trunc() as i64)
                        }
                    }
                }
                MultiplicativeExprOperator::Modulus => match (&left, &right) {
                    (AnyAtomicType::Integer(a), AnyAtomicType::Integer(b)) => {
                        if *b == 0 {
                            return Err(ExpressionApplyError {
                                msg: String::from("err:FOAR0002 Division by zero"),
                            });
                        }
                        AnyAtomicType::Integer(a % b)
                    }
                    _ => {
                        let a = to_f64(&left)?;
                        let b = to_f64(&right)?;
                        AnyAtomicType::Double(OrderedFloat(a % b))
                    }
                },
            };
        }

        Ok(xpath_item_set![XpathItem::AnyAtomicType(left)])
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct MultiplicativeExprPair(pub MultiplicativeExprOperator, pub UnionExpr);

impl Display for MultiplicativeExprPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MultiplicativeExprOperator {
    Star,
    Div,
    IntegerDiv,
    Modulus,
}

impl Display for MultiplicativeExprOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiplicativeExprOperator::Star => write!(f, "*"),
            MultiplicativeExprOperator::Div => write!(f, "div"),
            MultiplicativeExprOperator::IntegerDiv => write!(f, "idiv"),
            MultiplicativeExprOperator::Modulus => write!(f, "mod"),
        }
    }
}

pub fn unary_expr(input: &str) -> Res<&str, UnaryExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-UnaryExpr

    fn plus(input: &str) -> Res<&str, UnarySymbol> {
        ws((char('+'),))(input).map(|(next_input, _res)| (next_input, UnarySymbol::Plus))
    }

    fn minus(input: &str) -> Res<&str, UnarySymbol> {
        ws((char('-'),))(input).map(|(next_input, _res)| (next_input, UnarySymbol::Minus))
    }

    context("unary_expr", tuple((many0(alt((plus, minus))), value_expr)))(input).map(
        |(next_input, res)| {
            (
                next_input,
                UnaryExpr {
                    leading_symbols: res.0,
                    expr: res.1,
                },
            )
        },
    )
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnaryExpr {
    pub leading_symbols: Vec<UnarySymbol>,
    pub expr: ValueExpr,
}

impl Display for UnaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in &self.leading_symbols {
            write!(f, "{}", x)?;
        }

        write!(f, "{}", self.expr)
    }
}

impl UnaryExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there are no leading symbols, return the eval as-is.
        if self.leading_symbols.is_empty() {
            return Ok(result);
        }

        // Atomize the operand.
        let val = match atomize_single(&result, context)? {
            Some(val) => val,
            None => return Ok(XpathItemSet::new()),
        };

        // Count the number of minus signs to determine if negation is needed.
        let negate = self
            .leading_symbols
            .iter()
            .filter(|s| matches!(s, UnarySymbol::Minus))
            .count()
            % 2
            == 1;

        if !negate {
            // Even number of minus signs (or all plus) — validate numeric, return as-is.
            let _ = to_f64(&val)?;
            return Ok(xpath_item_set![XpathItem::AnyAtomicType(val)]);
        }

        // Negate the value.
        let negated = match val {
            AnyAtomicType::Integer(n) => AnyAtomicType::Integer(-n),
            AnyAtomicType::Float(n) => AnyAtomicType::Float(OrderedFloat(-n.into_inner())),
            AnyAtomicType::Double(n) => AnyAtomicType::Double(OrderedFloat(-n.into_inner())),
            other => {
                let n = to_f64(&other)?;
                AnyAtomicType::Double(OrderedFloat(-n))
            }
        };

        Ok(xpath_item_set![XpathItem::AnyAtomicType(negated)])
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum UnarySymbol {
    Plus,
    Minus,
}

impl Display for UnarySymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnarySymbol::Plus => write!(f, "+"),
            UnarySymbol::Minus => write!(f, "-"),
        }
    }
}

fn value_expr(input: &str) -> Res<&str, ValueExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ValueExpr

    context("value_expr", simple_map_expr)(input)
        .map(|(next_input, res)| (next_input, ValueExpr(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct ValueExpr(pub SimpleMapExpr);

impl Display for ValueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ValueExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        self.0.eval(context)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn additive_expr_should_parse() {
        // arrange
        let input = "A+B";

        // act
        let (next_input, res) = additive_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "A + B");
    }

    #[test]
    fn additive_expr_should_parse_whitespace() {
        // arrange
        let input = "A + B - C";

        // act
        let (next_input, res) = additive_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "A + B - C");
    }

    #[test]
    fn multiplicative_expr_should_parse() {
        // arrange
        let input = "A*B";

        // act
        let (next_input, res) = multiplicative_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "A * B");
    }

    #[test]
    fn multiplicative_expr_should_parse_whitespace() {
        // arrange
        let input = "A * B div C idiv D mod E";

        // act
        let (next_input, res) = multiplicative_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "A * B div C idiv D mod E");
    }

    #[test]
    fn unary_expr_should_parse_minus() {
        // arrange
        let input = "-+A";

        // act
        let (next_input, res) = unary_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "-+A");
    }

    #[test]
    fn unary_expr_should_parse_minus_whitespace() {
        // arrange
        let input = "- + A";

        // act
        let (next_input, res) = unary_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "-+A");
    }
}
