//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-quantified-expressions>

use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    error::context,
    multi::many0,
    sequence::tuple,
};

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            expressions::{expr_single, primary_expressions::variable_references::var_name},
            recipes::Res,
            terminal_symbols::symbol_separator,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

use super::{primary_expressions::variable_references::VarName, ExprSingle};

pub fn quantified_expr(input: &str) -> Res<&str, QuantifiedExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-QuantifiedExpr

    fn some_quantifier(input: &str) -> Res<&str, Quantifier> {
        tag("some")(input).map(|(next_input, _res)| (next_input, Quantifier::Some))
    }

    fn every_quantifier(input: &str) -> Res<&str, Quantifier> {
        tag("every")(input).map(|(next_input, _res)| (next_input, Quantifier::Every))
    }

    context(
        "quantified_expr",
        tuple((
            alt((some_quantifier, every_quantifier)),
            symbol_separator,
            char('$'),
            var_name,
            symbol_separator,
            tag("in"),
            symbol_separator,
            expr_single,
            many0(tuple((
                char(','),
                multispace0,
                char('$'),
                var_name,
                symbol_separator,
                tag("in"),
                symbol_separator,
                expr_single,
            ))),
            symbol_separator,
            tag("satisfies"),
            symbol_separator,
            expr_single,
        )),
    )(input)
    .map(|(next_input, res)| {
        let extras = res
            .8
            .into_iter()
            .map(|r| QuantifiedExprItem {
                var: r.3,
                expr: r.7,
            })
            .collect();
        (
            next_input,
            QuantifiedExpr {
                quantifier: res.0,
                item: QuantifiedExprItem {
                    var: res.3,
                    expr: res.7,
                },
                extras,
                satisfies: res.12,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct QuantifiedExpr {
    pub quantifier: Quantifier,
    pub item: QuantifiedExprItem,
    pub extras: Vec<QuantifiedExprItem>,
    pub satisfies: ExprSingle,
}

impl QuantifiedExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Collect all bindings (first + extras) into a single list.
        let mut bindings = vec![&self.item];
        bindings.extend(self.extras.iter());

        // Recursively evaluate bindings, then check the satisfies condition.
        // For `some $x in E1, $y in E2 satisfies E3`,
        // this is equivalent to `some $x in E1 satisfies (some $y in E2 satisfies E3)`.
        let result =
            Self::eval_bindings(context, &bindings, &self.satisfies, &self.quantifier)?;

        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Boolean(result),
        )])
    }

    fn eval_bindings<'tree>(
        context: &XpathExpressionContext<'tree>,
        bindings: &[&QuantifiedExprItem],
        satisfies_expr: &ExprSingle,
        quantifier: &Quantifier,
    ) -> Result<bool, ExpressionApplyError> {
        let (binding, rest) = match bindings.split_first() {
            Some(pair) => pair,
            None => {
                // No more bindings — evaluate the satisfies expression and get its EBV.
                let result = satisfies_expr.eval(context)?;
                return result.boolean();
            }
        };

        // Evaluate the binding's "in" expression to get the sequence to iterate over.
        let sequence = binding.expr.eval(context)?;
        let var_name = binding.var.to_string();

        match quantifier {
            Quantifier::Some => {
                // Return true if any item satisfies the condition.
                for item in &sequence {
                    let var_value = xpath_item_set![item.clone()];
                    let inner_context = context.with_variable(var_name.clone(), var_value);
                    if Self::eval_bindings(&inner_context, rest, satisfies_expr, quantifier)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Quantifier::Every => {
                // Return true only if every item satisfies the condition.
                for item in &sequence {
                    let var_value = xpath_item_set![item.clone()];
                    let inner_context = context.with_variable(var_name.clone(), var_value);
                    if !Self::eval_bindings(&inner_context, rest, satisfies_expr, quantifier)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }
}

impl Display for QuantifiedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.quantifier, self.item)?;
        for extra in &self.extras {
            write!(f, ", {}", extra)?;
        }
        write!(f, " satisfies {}", self.satisfies)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Quantifier {
    Some,
    Every,
}

impl Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Quantifier::Some => write!(f, "some"),
            Quantifier::Every => write!(f, "every"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct QuantifiedExprItem {
    pub var: VarName,
    pub expr: ExprSingle,
}

impl Display for QuantifiedExprItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${} in {}", self.var, self.expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quantified_expr_should_parse_every() {
        // arrange
        let input = "every $part in /parts/part satisfies $part/@discounted";

        // act
        let (next_input, res) = quantified_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "every $part in /parts/part satisfies $part/@discounted"
        );
    }

    #[test]
    fn quantified_expr_should_parse_some() {
        // arrange
        let input = r#"some $emp in /emps/employee
        satisfies $part/@discounted"#;

        // act
        let (next_input, res) = quantified_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "some $emp in /emps/employee satisfies $part/@discounted"
        );
    }

    #[test]
    fn quantified_expr_should_parse_multiple_bindings() {
        // arrange
        let input = "some $x in (1, 2), $y in (3, 4) satisfies $x + $y = 6";

        // act
        let (next_input, res) = quantified_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "some $x in (1, 2), $y in (3, 4) satisfies $x + $y=6"
        );
    }
}
