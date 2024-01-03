//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-let-expressions

use std::fmt::Display;

use nom::{
    bytes::complete::tag,
    character::complete::{char, multispace0},
    error::context,
    multi::many0,
    sequence::tuple,
};

use crate::xpath::grammar::{
    recipes::Res,
    terminal_symbols::symbol_separator,
    whitespace_recipes::{sep, ws},
};

use super::{
    expr_single,
    primary_expressions::variable_references::{var_name, VarName},
    ExprSingle,
};

pub fn let_expr(input: &str) -> Res<&str, LetExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-LetExpr

    context(
        "let_expr",
        sep((simple_let_clause, tag("return"), expr_single)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            LetExpr {
                clause: res.0,
                expr: res.2,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct LetExpr {
    pub clause: SimpleLetClause,
    pub expr: ExprSingle,
}

impl Display for LetExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} return {}", self.clause, self.expr)
    }
}

fn simple_let_clause(input: &str) -> Res<&str, SimpleLetClause> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SimpleLetClause

    context(
        "simple_let_clause",
        tuple((
            tag("let"),
            symbol_separator,
            simple_let_binding,
            many0(tuple((char(','), multispace0, simple_let_binding))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let extras = res.3.into_iter().map(|(_, _, binding)| binding).collect();
        (
            next_input,
            SimpleLetClause {
                binding: res.2,
                extras,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleLetClause {
    pub binding: SimpleLetBinding,
    pub extras: Vec<SimpleLetBinding>,
}

impl Display for SimpleLetClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {}", self.binding)?;
        for binding in &self.extras {
            write!(f, ", {}", binding)?;
        }
        Ok(())
    }
}

fn simple_let_binding(input: &str) -> Res<&str, SimpleLetBinding> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SimpleLetBinding

    context(
        "simple_let_binding",
        ws((char('$'), var_name, tag(":="), expr_single)),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            SimpleLetBinding {
                var: res.1,
                expr: res.3,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleLetBinding {
    pub var: VarName,
    pub expr: ExprSingle,
}

impl Display for SimpleLetBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${} := {}", self.var, self.expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn let_expr_should_parse() {
        // arrange
        let input = r#"let $x:=4,$y:=3 return $x+$y"#;

        // act
        let (next_input, res) = let_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"let $x := 4, $y := 3 return $x + $y"#);
    }

    #[test]
    fn let_expr_should_parse_whitespace() {
        // arrange
        let input = r#"let $x := 4, $y := 3 
            return $x + $y"#;

        // act
        let (next_input, res) = let_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), r#"let $x := 4, $y := 3 return $x + $y"#);
    }
}
