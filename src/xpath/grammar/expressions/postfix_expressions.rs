//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-postfix-expression>

use std::fmt::Display;

use nom::{branch::alt, character::complete::char, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::{AnyAtomicType, Function, XpathItem},
        expressions::{
            common::argument_list, maps_and_arrays::lookup_operator::postfix_lookup::lookup,
            primary_expressions::primary_expr,
        },
        recipes::Res,
        whitespace_recipes::ws,
    },
    ExpressionApplyError, XpathExpressionContext, XpathItemSet,
};

use super::{
    common::ArgumentList, expr, maps_and_arrays::lookup_operator::postfix_lookup::Lookup,
    primary_expressions::{
        static_function_calls::dispatch_function, PrimaryExpr,
    },
    Expr,
};

use crate::xpath::grammar::types::eq_name;

pub fn postfix_expr(input: &str) -> Res<&str, PostfixExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PostfixExpr

    fn predicate_map(input: &str) -> Res<&str, PostfixExprItem> {
        predicate(input).map(|(next_input, res)| (next_input, PostfixExprItem::Predicate(res)))
    }

    fn argument_list_map(input: &str) -> Res<&str, PostfixExprItem> {
        argument_list(input)
            .map(|(next_input, res)| (next_input, PostfixExprItem::ArgumentList(res)))
    }

    fn lookup_map(input: &str) -> Res<&str, PostfixExprItem> {
        lookup(input).map(|(next_input, res)| (next_input, PostfixExprItem::Lookup(res)))
    }

    context(
        "postfix_expr",
        tuple((
            primary_expr,
            many0(alt((predicate_map, argument_list_map, lookup_map))),
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            PostfixExpr {
                expr: res.0,
                items: res.1,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct PostfixExpr {
    pub expr: PrimaryExpr,
    pub items: Vec<PostfixExprItem>,
}

impl Display for PostfixExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, "{}", x)?;
        }

        Ok(())
    }
}

impl PostfixExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let mut result = self.expr.eval(context)?;

        // If there are no postfix items, return the primary expression's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Apply each postfix item in sequence.
        for item in &self.items {
            match item {
                PostfixExprItem::Predicate(predicate) => {
                    let mut filtered = XpathItemSet::new();
                    for (i, _) in result.iter().enumerate() {
                        let predicate_context = context.new_with_variables(
                            &result,
                            i + 1,
                            false,
                        );
                        if predicate.is_match(&predicate_context)? {
                            filtered.insert(result[i].clone());
                        }
                    }
                    result = filtered;
                }
                PostfixExprItem::ArgumentList(args) => {
                    result = eval_dynamic_function_call(&result, args, context)?;
                }
                PostfixExprItem::Lookup(_) => {
                    return Err(ExpressionApplyError {
                        msg: String::from(
                            "PostfixExpr: postfix lookup not yet supported",
                        ),
                    });
                }
            }
        }

        Ok(result)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum PostfixExprItem {
    Predicate(Predicate),
    ArgumentList(ArgumentList),
    Lookup(Lookup),
}

impl Display for PostfixExprItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostfixExprItem::Predicate(x) => write!(f, "{}", x),
            PostfixExprItem::ArgumentList(x) => write!(f, "{}", x),
            PostfixExprItem::Lookup(x) => write!(f, "{}", x),
        }
    }
}

pub fn predicate(input: &str) -> Res<&str, Predicate> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Predicate
    context("predicate", ws((char('['), expr, char(']'))))(input)
        .map(|(next_input, res)| (next_input, Predicate(res.1)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct Predicate(Expr);

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

impl Predicate {
    pub(crate) fn is_match<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<bool, ExpressionApplyError> {
        let res = self.0.eval(&context)?;

        // The predicate truth value is derived by applying the following rules, in order:
        // 1. If the value of the predicate expression is a singleton atomic value of a numeric type or derived from a numeric type,
        //    the predicate truth value is true if the value of the predicate expression is equal (by the eq operator) to the context position,
        //    and is false otherwise.
        // 2. Otherwise, the predicate truth value is the effective boolean value of the predicate expression.

        // Step 1. If the value is a number, check if it matches the context position.
        if res.len() == 1 {
            match &res[0] {
                XpathItem::AnyAtomicType(atomic_type) => match atomic_type {
                    AnyAtomicType::Integer(n) => return Ok(*n == context.position as i64),
                    AnyAtomicType::Float(n) => return Ok(*n == context.position as f32),
                    AnyAtomicType::Double(n) => return Ok(*n == context.position as f64),
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(res.boolean())
    }
}

/// Evaluate a dynamic function call: the result set should contain a single
/// function item, and the argument list provides the call arguments.
fn eval_dynamic_function_call<'tree>(
    result: &XpathItemSet<'tree>,
    args: &ArgumentList,
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    if result.len() != 1 {
        return Err(ExpressionApplyError::new(format!(
            "Dynamic function call requires exactly one item, got {}",
            result.len()
        )));
    }

    let func = match &result[0] {
        XpathItem::Function(f) => f,
        other => {
            return Err(ExpressionApplyError::new(format!(
                "Dynamic function call requires a function item, got {:?}",
                other
            )));
        }
    };

    // Evaluate the call arguments.
    let mut arg_values = Vec::new();
    for arg in &args.0 {
        arg_values.push(arg.eval(context)?);
    }

    match func {
        Function::Named { name, arity } => {
            if arg_values.len() as u32 != *arity {
                return Err(ExpressionApplyError::new(format!(
                    "Function {}#{} expects {} arguments, got {}",
                    name,
                    arity,
                    arity,
                    arg_values.len()
                )));
            }
            // Re-parse the name to an EQName and dispatch.
            let (_, parsed_name) = eq_name(name).map_err(|e| {
                ExpressionApplyError::new(format!(
                    "Failed to parse function name '{}': {}",
                    name, e
                ))
            })?;
            dispatch_function(&parsed_name, &arg_values, context)
        }
        Function::Inline {
            params,
            body_source,
        } => {
            if arg_values.len() != params.len() {
                return Err(ExpressionApplyError::new(format!(
                    "Inline function expects {} arguments, got {}",
                    params.len(),
                    arg_values.len()
                )));
            }
            // Re-parse the body source and evaluate.
            if body_source.is_empty() {
                return Ok(XpathItemSet::new());
            }
            // Bind parameters to argument values.
            let bindings = params.iter().cloned().zip(arg_values);
            let inner_context = context.with_variables_iter(bindings);
            let (_, body_expr) = expr(body_source).map_err(|e| {
                ExpressionApplyError::new(format!(
                    "Failed to parse inline function body '{}': {}",
                    body_source, e
                ))
            })?;
            body_expr.eval(&inner_context)
        }
        Function::Map { entries } => {
            // Maps can be called as functions: $map("key") returns the value
            // for that key. Exactly one argument is required.
            if arg_values.len() != 1 {
                return Err(ExpressionApplyError::new(format!(
                    "Map lookup requires exactly 1 argument, got {}",
                    arg_values.len()
                )));
            }
            let key_set = &arg_values[0];
            if key_set.len() != 1 {
                return Err(ExpressionApplyError::new(format!(
                    "Map lookup key must be a single item, got {}",
                    key_set.len()
                )));
            }
            let key = match &key_set[0] {
                XpathItem::AnyAtomicType(a) => a,
                _ => {
                    return Err(ExpressionApplyError::new(
                        "Map lookup key must be an atomic value".to_string(),
                    ));
                }
            };
            // Find the entry with the matching key.
            for (k, v) in entries {
                if k == key {
                    return Ok(v
                        .iter()
                        .map(|a| XpathItem::AnyAtomicType(a.clone()))
                        .collect());
                }
            }
            // Key not found — return empty sequence.
            Ok(XpathItemSet::new())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn postfix_expr_should_parse1() {
        // arrange
        let input = "$products[price gt 100]";

        // act
        let (next_input, res) = postfix_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn predicate_should_parse1() {
        // arrange
        let input = "[price gt 100]";

        // act
        let (next_input, res) = predicate(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn predicate_should_parse2() {
        // arrange
        let input = "[2]";

        // act
        let (next_input, res) = predicate(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn predicate_should_parse3() {
        // arrange
        let input = "[ 2 ]";

        // act
        let (next_input, res) = predicate(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "[2]");
    }
}
