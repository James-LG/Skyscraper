//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-cast>

use std::fmt::Display;

use nom::{
    bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    sequence::tuple,
};

use ordered_float::OrderedFloat;

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            expressions::{
                arrow_operator::{arrow_expr, ArrowExpr},
                primary_expressions::static_function_calls::func_data,
            },
            recipes::Res,
            types::{simple_type_name, EQName, SimpleTypeName},
            whitespace_recipes::sep,
            xml_names::QName,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

pub fn cast_expr(input: &str) -> Res<&str, CastExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-CastExpr

    context(
        "cast_expr",
        sep((arrow_expr, opt(sep((tag("cast"), tag("as"), single_type))))),
    )(input)
    .map(|(next_input, res)| {
        let cast = res.1.map(|res| res.2);
        (next_input, CastExpr { expr: res.0, cast })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct CastExpr {
    pub expr: ArrowExpr,
    pub cast: Option<SingleType>,
}

impl Display for CastExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(x) = &self.cast {
            write!(f, " cast as {}", x)?;
        }

        Ok(())
    }
}

impl CastExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's no `cast as` clause, return the base expression's eval.
        let single_type = match &self.cast {
            Some(t) => t,
            None => return Ok(result),
        };

        // If `?` is present and the sequence is empty, return empty.
        if single_type.has_question_mark && result.is_empty() {
            return Ok(result);
        }

        // Atomize the result to get a single atomic value.
        let atomized = func_data(&result, context.item_tree);
        if atomized.len() != 1 {
            return Err(ExpressionApplyError {
                msg: format!(
                    "cast as: expected single atomic value, got {}",
                    atomized.len()
                ),
            });
        }

        let source = &atomized[0];
        let target_name = Self::resolve_type_name(&single_type.type_name)?;
        let casted = Self::cast_atomic(source, &target_name)?;

        Ok(xpath_item_set![XpathItem::AnyAtomicType(casted)])
    }

    pub(crate) fn resolve_type_name(type_name: &SimpleTypeName) -> Result<String, ExpressionApplyError> {
        match &type_name.0 .0 {
            EQName::QName(qname) => match qname {
                QName::UnprefixedName(name) => Ok(name.clone()),
                QName::PrefixedName(prefixed) => {
                    if prefixed.prefix == "xs" {
                        Ok(prefixed.local_part.clone())
                    } else {
                        Err(ExpressionApplyError {
                            msg: format!("cast as: unsupported type prefix '{}'", prefixed.prefix),
                        })
                    }
                }
            },
            EQName::UriQualifiedName(uqn) => {
                if uqn.uri == "http://www.w3.org/2001/XMLSchema" {
                    Ok(uqn.name.clone())
                } else {
                    Err(ExpressionApplyError {
                        msg: format!(
                            "cast as: unsupported type namespace '{}'",
                            uqn.uri
                        ),
                    })
                }
            }
        }
    }

    pub(crate) fn cast_atomic(
        source: &AnyAtomicType,
        target_type: &str,
    ) -> Result<AnyAtomicType, ExpressionApplyError> {
        match target_type {
            "string" => Ok(AnyAtomicType::String(source.to_string())),
            "integer" => match source {
                AnyAtomicType::Integer(_) => Ok(source.clone()),
                AnyAtomicType::String(s) => s.trim().parse::<i64>().map(AnyAtomicType::Integer).map_err(|_| {
                    ExpressionApplyError {
                        msg: format!("cast as integer: cannot cast string '{}'", s),
                    }
                }),
                AnyAtomicType::Float(f) => Ok(AnyAtomicType::Integer(f.0 as i64)),
                AnyAtomicType::Double(d) => Ok(AnyAtomicType::Integer(d.0 as i64)),
                AnyAtomicType::Boolean(b) => Ok(AnyAtomicType::Integer(if *b { 1 } else { 0 })),
            },
            "double" => match source {
                AnyAtomicType::Double(_) => Ok(source.clone()),
                AnyAtomicType::Integer(n) => Ok(AnyAtomicType::Double(OrderedFloat(*n as f64))),
                AnyAtomicType::Float(f) => Ok(AnyAtomicType::Double(OrderedFloat(f.0 as f64))),
                AnyAtomicType::String(s) => s.trim().parse::<f64>().map(|v| AnyAtomicType::Double(OrderedFloat(v))).map_err(|_| {
                    ExpressionApplyError {
                        msg: format!("cast as double: cannot cast string '{}'", s),
                    }
                }),
                AnyAtomicType::Boolean(b) => Ok(AnyAtomicType::Double(OrderedFloat(if *b { 1.0 } else { 0.0 }))),
            },
            "float" => match source {
                AnyAtomicType::Float(_) => Ok(source.clone()),
                AnyAtomicType::Integer(n) => Ok(AnyAtomicType::Float(OrderedFloat(*n as f32))),
                AnyAtomicType::Double(d) => Ok(AnyAtomicType::Float(OrderedFloat(d.0 as f32))),
                AnyAtomicType::String(s) => s.trim().parse::<f32>().map(|v| AnyAtomicType::Float(OrderedFloat(v))).map_err(|_| {
                    ExpressionApplyError {
                        msg: format!("cast as float: cannot cast string '{}'", s),
                    }
                }),
                AnyAtomicType::Boolean(b) => Ok(AnyAtomicType::Float(OrderedFloat(if *b { 1.0 } else { 0.0 }))),
            },
            "boolean" => match source {
                AnyAtomicType::Boolean(_) => Ok(source.clone()),
                AnyAtomicType::Integer(n) => Ok(AnyAtomicType::Boolean(*n != 0)),
                AnyAtomicType::String(s) => match s.trim() {
                    "true" | "1" => Ok(AnyAtomicType::Boolean(true)),
                    "false" | "0" => Ok(AnyAtomicType::Boolean(false)),
                    _ => Err(ExpressionApplyError {
                        msg: format!("cast as boolean: cannot cast string '{}'", s),
                    }),
                },
                AnyAtomicType::Float(f) => Ok(AnyAtomicType::Boolean(f.0 != 0.0)),
                AnyAtomicType::Double(d) => Ok(AnyAtomicType::Boolean(d.0 != 0.0)),
            },
            _ => Err(ExpressionApplyError {
                msg: format!("cast as: unsupported target type '{}'", target_type),
            }),
        }
    }
}

pub fn single_type(input: &str) -> Res<&str, SingleType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SingleType
    context("single_type", tuple((simple_type_name, opt(char('?')))))(input).map(
        |(next_input, res)| {
            (
                next_input,
                SingleType {
                    type_name: res.0,
                    has_question_mark: res.1.is_some(),
                },
            )
        },
    )
}

#[derive(PartialEq, Debug, Clone)]
pub struct SingleType {
    pub type_name: SimpleTypeName,
    pub has_question_mark: bool,
}

impl Display for SingleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_name)?;
        if self.has_question_mark {
            write!(f, "?")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_type_should_parse() {
        // arrange
        let input = "integer?";

        // act
        let (next_input, res) = single_type(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn cast_expr_should_parse() {
        // arrange
        let input = "fn:root() cast as integer?";

        // act
        let (next_input, res) = cast_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
