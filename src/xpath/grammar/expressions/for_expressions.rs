//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-for-expressions>

use std::fmt::Display;

use nom::{
    bytes::complete::tag,
    character::complete::{char, multispace0},
    error::context,
    multi::many0,
    sequence::tuple,
};

use crate::{
    xpath::{
        grammar::{recipes::Res, terminal_symbols::symbol_separator, whitespace_recipes::sep},
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

use super::{
    expr_single,
    primary_expressions::variable_references::{var_name, VarName},
    ExprSingle,
};

pub fn for_expr(input: &str) -> Res<&str, ForExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ForExpr

    context(
        "for_expr",
        sep((simple_for_clause, tag("return"), expr_single)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            ForExpr {
                clause: res.0,
                expr: res.2,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct ForExpr {
    pub clause: SimpleForClause,
    pub expr: ExprSingle,
}

impl ForExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Collect all bindings (first + extras) into a single list.
        let mut bindings = vec![&self.clause.binding];
        bindings.extend(self.clause.extras.iter());

        // Recursively evaluate bindings.
        // For multiple bindings like `for $x in E1, $y in E2 return E3`,
        // this is equivalent to `for $x in E1 return (for $y in E2 return E3)`.
        Self::eval_bindings(context, &bindings, &self.expr)
    }

    fn eval_bindings<'tree>(
        context: &XpathExpressionContext<'tree>,
        bindings: &[&SimpleForBinding],
        return_expr: &ExprSingle,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let (binding, rest) = match bindings.split_first() {
            Some(pair) => pair,
            None => {
                // No more bindings — evaluate the return expression.
                return return_expr.eval(context);
            }
        };

        // Evaluate the binding's "in" expression to get the sequence to iterate over.
        let sequence = binding.expr.eval(context)?;
        let mut result = XpathItemSet::new();

        // For each item in the sequence, bind the variable and evaluate the rest.
        let var_name = binding.var.to_string();
        for item in &sequence {
            let var_value = xpath_item_set![item.clone()];

            let inner_context = context.with_variable(var_name.clone(), var_value);

            let inner_result = Self::eval_bindings(&inner_context, rest, return_expr)?;
            result.extend(inner_result);
        }

        Ok(result)
    }
}

impl Display for ForExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} return {}", self.clause, self.expr)
    }
}

fn simple_for_clause(input: &str) -> Res<&str, SimpleForClause> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SimpleForClause

    context(
        "simple_for_clause",
        tuple((
            tag("for"),
            symbol_separator,
            simple_for_binding,
            many0(tuple((char(','), multispace0, simple_for_binding))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let extras = res.3.into_iter().map(|(_, _, binding)| binding).collect();
        (
            next_input,
            SimpleForClause {
                binding: res.2,
                extras,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleForClause {
    pub binding: SimpleForBinding,
    pub extras: Vec<SimpleForBinding>,
}

impl Display for SimpleForClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "for {}", self.binding)?;
        for extra in &self.extras {
            write!(f, ", {}", extra)?;
        }
        Ok(())
    }
}

fn simple_for_binding(input: &str) -> Res<&str, SimpleForBinding> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SimpleForClause

    context(
        "simple_for_binding",
        tuple((
            char('$'),
            var_name,
            symbol_separator,
            tag("in"),
            expr_single,
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            SimpleForBinding {
                var: res.1,
                expr: res.4,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleForBinding {
    pub var: VarName,
    pub expr: ExprSingle,
}

impl Display for SimpleForBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${} in {}", self.var, self.expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn for_expr_should_parse() {
        // arrange
        let input = r#"for $x in $z,$y in f($x) return g($x,$y)"#;

        // act
        let (next_input, res) = for_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "for $x in $z, $y in f($x) return g($x, $y)"
        );
    }

    #[test]
    fn for_expr_should_parse_whitespace() {
        // arrange
        let input = r#"for $x in $z, $y in f($x)
            return g($x, $y)"#;

        // act
        let (next_input, res) = for_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(
            res.to_string(),
            "for $x in $z, $y in f($x) return g($x, $y)"
        );
    }
}
