//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-arithmetic

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, error::context, multi::many0,
    sequence::tuple,
};

use crate::xpath::{
    grammar::{
        expressions::sequence_expressions::combining_node_sequences::union_expr, recipes::Res,
    },
    ExpressionApplyError, XPathExpressionContext, XPathResult,
};

use super::{
    sequence_expressions::combining_node_sequences::UnionExpr,
    simple_map_operator::{simple_map_expr, SimpleMapExpr},
};

pub fn additive_expr(input: &str) -> Res<&str, AdditiveExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AdditiveExpr

    fn plus(input: &str) -> Res<&str, AdditiveExprOperator> {
        char('+')(input).map(|(next_input, _res)| (next_input, AdditiveExprOperator::Plus))
    }

    fn minus(input: &str) -> Res<&str, AdditiveExprOperator> {
        char('-')(input).map(|(next_input, _res)| (next_input, AdditiveExprOperator::Minus))
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
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Otherwise, do the operation.
        todo!("AdditiveExpr::eval additive operator")
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
        char('*')(input).map(|(next_input, _res)| (next_input, MultiplicativeExprOperator::Star))
    }

    fn div(input: &str) -> Res<&str, MultiplicativeExprOperator> {
        tag("div")(input).map(|(next_input, _res)| (next_input, MultiplicativeExprOperator::Div))
    }

    fn integer_div(input: &str) -> Res<&str, MultiplicativeExprOperator> {
        tag("idiv")(input)
            .map(|(next_input, _res)| (next_input, MultiplicativeExprOperator::IntegerDiv))
    }

    fn modulus(input: &str) -> Res<&str, MultiplicativeExprOperator> {
        tag("mod")(input)
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
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return Ok(result);
        }

        // Otherwise, do the operation.
        todo!("MultiplicativeExpr::eval multiplicative operator")
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
        char('+')(input).map(|(next_input, _res)| (next_input, UnarySymbol::Plus))
    }

    fn minus(input: &str) -> Res<&str, UnarySymbol> {
        char('-')(input).map(|(next_input, _res)| (next_input, UnarySymbol::Minus))
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
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        // If there's only one parameter, return it's eval.
        if self.leading_symbols.is_empty() {
            return Ok(result);
        }

        // Otherwise, do the operation.
        todo!("UnaryExpr::eval unary operator")
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
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        self.0.eval(context)
    }
}
