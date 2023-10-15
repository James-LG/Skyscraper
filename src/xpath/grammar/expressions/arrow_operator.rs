//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-arrow-operator

use nom::{branch::alt, bytes::complete::tag, multi::many0, sequence::tuple};

use crate::xpath::grammar::{
    expressions::primary_expressions::{
        parenthesized_expressions::parenthesized_expr, variable_references::var_ref,
    },
    recipes::Res,
    types::{eq_name, EQName},
};

use super::{
    arithmetic_expressions::{unary_expr, UnaryExpr},
    common::{argument_list, ArgumentList},
    primary_expressions::{
        parenthesized_expressions::ParenthesizedExpr, variable_references::VarRef,
    },
};

pub fn arrow_expr(input: &str) -> Res<&str, ArrowExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ArrowExpr

    tuple((
        unary_expr,
        many0(tuple((tag("=>"), arrow_function_specifier, argument_list))),
    ))(input)
    .map(|(next_input, res)| {
        let expr = res.0;
        let items = res
            .1
            .into_iter()
            .map(|res| ArrowExprItem {
                function_specifier: res.1,
                arguments: res.2,
            })
            .collect();
        (next_input, ArrowExpr { expr, items })
    })
}

pub struct ArrowExpr {
    pub expr: UnaryExpr,
    pub items: Vec<ArrowExprItem>,
}

pub struct ArrowExprItem {
    pub function_specifier: ArrowFunctionSpecifier,
    pub arguments: ArgumentList,
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

    alt((name_map, var_ref_map, parenthesized_expr_map))(input)
}

pub enum ArrowFunctionSpecifier {
    Name(EQName),
    VarRef(VarRef),
    ParenthesizedExpr(ParenthesizedExpr),
}
