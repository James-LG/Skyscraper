//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-arrow-operator>

use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        data_model::{Function, XpathItem},
        expressions::primary_expressions::{
            parenthesized_expressions::parenthesized_expr, variable_references::var_ref,
        },
        recipes::Res,
        types::{eq_name, EQName},
        whitespace_recipes::ws,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::{
    arithmetic_expressions::{unary_expr, UnaryExpr},
    common::{argument_list, ArgumentList},
    postfix_expressions::invoke_function_item,
    primary_expressions::{
        parenthesized_expressions::ParenthesizedExpr,
        static_function_calls::dispatch_function,
        variable_references::VarRef,
    },
};

pub fn arrow_expr(input: &str) -> Res<&str, ArrowExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ArrowExpr

    context(
        "arrow_expr",
        tuple((
            unary_expr,
            many0(tuple((
                ws((tag("=>"), arrow_function_specifier)),
                argument_list,
            ))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let expr = res.0;
        let items = res
            .1
            .into_iter()
            .map(|res| ArrowExprItem {
                function_specifier: res.0 .1,
                arguments: res.1,
            })
            .collect();
        (next_input, ArrowExpr { expr, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct ArrowExpr {
    pub expr: UnaryExpr,
    pub items: Vec<ArrowExprItem>,
}

impl Display for ArrowExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " => {}", x)?;
        }

        Ok(())
    }
}

impl ArrowExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let mut result = self.expr.eval(context)?;

        // If there are no arrow items, return the base expression's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Chain through each arrow item: result => func(args)
        // is equivalent to func(result, args).
        for item in &self.items {
            // Build the argument list with the LHS prepended.
            let mut args = vec![result];
            for arg in &item.arguments.0 {
                args.push(arg.eval(context)?);
            }

            result = match &item.function_specifier {
                ArrowFunctionSpecifier::Name(name) => {
                    dispatch_function(name, &args, context)?
                }
                ArrowFunctionSpecifier::VarRef(var_ref) => {
                    let name = var_ref.name().to_string();
                    let func_set = context.get_variable(&name).ok_or_else(|| {
                        ExpressionApplyError::new(format!(
                            "ArrowExpr: undefined variable ${}",
                            name
                        ))
                    })?;
                    let func = extract_function_item(func_set, &format!("${}", name))?;
                    invoke_function_item(func, args, context)?
                }
                ArrowFunctionSpecifier::ParenthesizedExpr(paren_expr) => {
                    let func_set = paren_expr.eval(context)?;
                    let func =
                        extract_function_item(&func_set, "parenthesized expression")?;
                    invoke_function_item(func, args, context)?
                }
            };
        }

        Ok(result)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ArrowExprItem {
    pub function_specifier: ArrowFunctionSpecifier,
    pub arguments: ArgumentList,
}

impl Display for ArrowExprItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.function_specifier, self.arguments)
    }
}

/// Extract a single function item from an XpathItemSet, returning a descriptive
/// error if the set doesn't contain exactly one function item.
fn extract_function_item<'a, 'tree>(
    items: &'a XpathItemSet<'tree>,
    source_desc: &str,
) -> Result<&'a Function, ExpressionApplyError> {
    if items.len() != 1 {
        return Err(ExpressionApplyError::new(format!(
            "ArrowExpr: {} must be a single function item, got {} items",
            source_desc,
            items.len()
        )));
    }
    match &items[0] {
        XpathItem::Function(f) => Ok(f),
        _ => Err(ExpressionApplyError::new(format!(
            "ArrowExpr: {} is not a function item",
            source_desc
        ))),
    }
}

fn arrow_function_specifier(input: &str) -> Res<&str, ArrowFunctionSpecifier> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ArrowFunctionSpecifier

    fn name_map(input: &str) -> Res<&str, ArrowFunctionSpecifier> {
        eq_name(input).map(|(next_input, res)| (next_input, ArrowFunctionSpecifier::Name(res)))
    }

    fn var_ref_map(input: &str) -> Res<&str, ArrowFunctionSpecifier> {
        var_ref(input).map(|(next_input, res)| (next_input, ArrowFunctionSpecifier::VarRef(res)))
    }

    fn parenthesized_expr_map(input: &str) -> Res<&str, ArrowFunctionSpecifier> {
        parenthesized_expr(input)
            .map(|(next_input, res)| (next_input, ArrowFunctionSpecifier::ParenthesizedExpr(res)))
    }

    context(
        "arrow_function_specifier",
        alt((name_map, var_ref_map, parenthesized_expr_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ArrowFunctionSpecifier {
    Name(EQName),
    VarRef(VarRef),
    ParenthesizedExpr(ParenthesizedExpr),
}

impl Display for ArrowFunctionSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrowFunctionSpecifier::Name(x) => write!(f, "{}", x),
            ArrowFunctionSpecifier::VarRef(x) => write!(f, "{}", x),
            ArrowFunctionSpecifier::ParenthesizedExpr(x) => write!(f, "{}", x),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn arrow_expr_should_parse() {
        // arrange
        let input = r#"$string=>upper-case()=>normalize-unicode()=>tokenize("\s+")"#;

        // act
        let (next_input, res) = arrow_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            r#"$string => upper-case() => normalize-unicode() => tokenize("\s+")"#
        );
    }

    #[test]
    fn arrow_expr_should_parse_whitespace() {
        // arrange
        let input = r#"$string => upper-case() => normalize-unicode() => tokenize("\s+")"#;

        // act
        let (next_input, res) = arrow_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            r#"$string => upper-case() => normalize-unicode() => tokenize("\s+")"#
        );
    }
}
