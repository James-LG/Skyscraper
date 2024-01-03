//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-for-expressions

use std::fmt::Display;

use nom::{
    bytes::complete::tag,
    character::complete::{char, multispace0},
    error::context,
    multi::many0,
    sequence::tuple,
};

use crate::xpath::grammar::{
    recipes::Res, terminal_symbols::symbol_separator, whitespace_recipes::sep,
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
