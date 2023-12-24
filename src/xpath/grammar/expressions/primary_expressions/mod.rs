//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-primary-expressions

use std::fmt::Display;

use nom::{branch::alt, character::complete::char, error::context};

use crate::{
    xpath::{
        grammar::{
            data_model::XpathItem,
            expressions::{
                maps_and_arrays::{
                    arrays::array_constructor, lookup_operator::unary_lookup::unary_lookup,
                    maps::map_constructor,
                },
                primary_expressions::{
                    inline_function_expressions::inline_function_expr, literals::literal,
                    named_function_references::named_function_ref,
                    parenthesized_expressions::parenthesized_expr,
                    static_function_calls::function_call, variable_references::var_ref,
                },
            },
            recipes::{max, Res},
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XPathExpressionContext,
    },
    xpath_item_set,
};

use self::{
    inline_function_expressions::InlineFunctionExpr, literals::Literal,
    named_function_references::NamedFunctionRef, parenthesized_expressions::ParenthesizedExpr,
    static_function_calls::FunctionCall, variable_references::VarRef,
};

use super::maps_and_arrays::{
    arrays::ArrayConstructor, lookup_operator::unary_lookup::UnaryLookup, maps::MapConstructor,
};

pub mod enclosed_expressions;
mod inline_function_expressions;
mod literals;
mod named_function_references;
pub mod parenthesized_expressions;
mod static_function_calls;
pub mod variable_references;

pub fn primary_expr(input: &str) -> Res<&str, PrimaryExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PrimaryExpr

    fn literal_map(input: &str) -> Res<&str, PrimaryExpr> {
        literal(input).map(|(next_input, res)| (next_input, PrimaryExpr::Literal(res)))
    }

    fn var_ref_map(input: &str) -> Res<&str, PrimaryExpr> {
        var_ref(input).map(|(next_input, res)| (next_input, PrimaryExpr::VarRef(res)))
    }

    fn parenthesized_expr_map(input: &str) -> Res<&str, PrimaryExpr> {
        parenthesized_expr(input)
            .map(|(next_input, res)| (next_input, PrimaryExpr::ParenthesizedExpr(res)))
    }

    fn context_item_expr(input: &str) -> Res<&str, PrimaryExpr> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ContextItemExpr
        char('.')(input).map(|(next_input, _res)| (next_input, PrimaryExpr::ContextItemExpr))
    }

    fn function_call_map(input: &str) -> Res<&str, PrimaryExpr> {
        function_call(input).map(|(next_input, res)| (next_input, PrimaryExpr::FunctionCall(res)))
    }

    fn function_item_expr_map(input: &str) -> Res<&str, PrimaryExpr> {
        function_item_expr(input)
            .map(|(next_input, res)| (next_input, PrimaryExpr::FunctionItemExpr(res)))
    }

    fn map_constructor_map(input: &str) -> Res<&str, PrimaryExpr> {
        map_constructor(input)
            .map(|(next_input, res)| (next_input, PrimaryExpr::MapConstructor(res)))
    }

    fn array_constructor_map(input: &str) -> Res<&str, PrimaryExpr> {
        array_constructor(input)
            .map(|(next_input, res)| (next_input, PrimaryExpr::ArrayConstructor(res)))
    }

    fn unary_lookup_map(input: &str) -> Res<&str, PrimaryExpr> {
        unary_lookup(input).map(|(next_input, res)| (next_input, PrimaryExpr::UnaryLookup(res)))
    }

    context(
        "primary_expr",
        max((
            literal_map,
            var_ref_map,
            parenthesized_expr_map,
            context_item_expr,
            function_call_map,
            function_item_expr_map,
            map_constructor_map,
            array_constructor_map,
            unary_lookup_map,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum PrimaryExpr {
    Literal(Literal),
    VarRef(VarRef),
    ParenthesizedExpr(ParenthesizedExpr),
    ContextItemExpr,
    FunctionCall(FunctionCall),
    FunctionItemExpr(FunctionItemExpr),
    MapConstructor(MapConstructor),
    ArrayConstructor(ArrayConstructor),
    UnaryLookup(UnaryLookup),
}

impl Display for PrimaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimaryExpr::Literal(x) => write!(f, "{}", x),
            PrimaryExpr::VarRef(x) => write!(f, "{}", x),
            PrimaryExpr::ParenthesizedExpr(x) => write!(f, "{}", x),
            PrimaryExpr::ContextItemExpr => write!(f, "."),
            PrimaryExpr::FunctionCall(x) => write!(f, "{}", x),
            PrimaryExpr::FunctionItemExpr(x) => write!(f, "{}", x),
            PrimaryExpr::MapConstructor(x) => write!(f, "{}", x),
            PrimaryExpr::ArrayConstructor(x) => write!(f, "{}", x),
            PrimaryExpr::UnaryLookup(x) => write!(f, "{}", x),
        }
    }
}

impl PrimaryExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        match self {
            PrimaryExpr::Literal(literal) => {
                Ok(xpath_item_set![XpathItem::AnyAtomicType(literal.value())])
            }
            PrimaryExpr::VarRef(_) => todo!("PrimaryExpr::VarRef eval"),
            PrimaryExpr::ParenthesizedExpr(expr) => expr.eval(context),
            PrimaryExpr::ContextItemExpr => todo!("PrimaryExpr::LiteContextItemExprral eval"),
            PrimaryExpr::FunctionCall(expr) => expr.eval(context),
            PrimaryExpr::FunctionItemExpr(_) => todo!("PrimaryExpr::FunctionItemExpr eval"),
            PrimaryExpr::MapConstructor(_) => todo!("PrimaryExpr::MapConstructor eval"),
            PrimaryExpr::ArrayConstructor(_) => todo!("PrimaryExpr::ArrayConstructor eval"),
            PrimaryExpr::UnaryLookup(_) => todo!("PrimaryExpr::UnaryLookup eval"),
        }
    }
}

fn function_item_expr(input: &str) -> Res<&str, FunctionItemExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-FunctionItemExpr

    fn named_function_ref_map(input: &str) -> Res<&str, FunctionItemExpr> {
        named_function_ref(input)
            .map(|(next_input, res)| (next_input, FunctionItemExpr::NamedFunctionRef(res)))
    }

    fn inline_function_expr_map(input: &str) -> Res<&str, FunctionItemExpr> {
        inline_function_expr(input)
            .map(|(next_input, res)| (next_input, FunctionItemExpr::InlineFunctionExpr(res)))
    }

    context(
        "function_item_expr",
        alt((named_function_ref_map, inline_function_expr_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionItemExpr {
    NamedFunctionRef(NamedFunctionRef),
    InlineFunctionExpr(InlineFunctionExpr),
}

impl Display for FunctionItemExpr {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt FunctionItemExpr")
    }
}
