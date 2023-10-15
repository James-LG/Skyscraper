//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-arithmetic

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, multi::many0, sequence::tuple,
};

use crate::xpath::grammar::{
    expressions::sequence_expressions::combining_node_sequences::union_expr, recipes::Res,
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

    tuple((
        multiplicative_expr,
        many0(tuple((alt((plus, minus)), multiplicative_expr))),
    ))(input)
    .map(|(next_input, res)| {
        let items = res
            .1
            .into_iter()
            .map(|res| AdditiveExprPair(res.0, res.1))
            .collect();
        (next_input, AdditiveExpr { expr: res.0, items })
    })
}

pub struct AdditiveExpr {
    pub expr: MultiplicativeExpr,
    pub items: Vec<AdditiveExprPair>,
}

pub struct AdditiveExprPair(pub AdditiveExprOperator, pub MultiplicativeExpr);

pub enum AdditiveExprOperator {
    Plus,
    Minus,
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

    tuple((
        union_expr,
        many0(tuple((alt((star, div, integer_div, modulus)), union_expr))),
    ))(input)
    .map(|(next_input, res)| {
        let items = res
            .1
            .into_iter()
            .map(|res| MultiplicativeExprPair(res.0, res.1))
            .collect();
        (next_input, MultiplicativeExpr { expr: res.0, items })
    })
}

pub struct MultiplicativeExpr {
    pub expr: UnionExpr,
    pub items: Vec<MultiplicativeExprPair>,
}

pub struct MultiplicativeExprPair(pub MultiplicativeExprOperator, pub UnionExpr);

pub enum MultiplicativeExprOperator {
    Star,
    Div,
    IntegerDiv,
    Modulus,
}

pub fn unary_expr(input: &str) -> Res<&str, UnaryExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-UnaryExpr

    fn plus(input: &str) -> Res<&str, UnarySymbol> {
        char('+')(input).map(|(next_input, _res)| (next_input, UnarySymbol::Plus))
    }

    fn minus(input: &str) -> Res<&str, UnarySymbol> {
        char('-')(input).map(|(next_input, _res)| (next_input, UnarySymbol::Minus))
    }

    tuple((many0(alt((plus, minus))), value_expr))(input).map(|(next_input, res)| {
        (
            next_input,
            UnaryExpr {
                leading_symbols: res.0,
                expr: res.1,
            },
        )
    })
}

pub struct UnaryExpr {
    pub leading_symbols: Vec<UnarySymbol>,
    pub expr: ValueExpr,
}

pub enum UnarySymbol {
    Plus,
    Minus,
}

fn value_expr(input: &str) -> Res<&str, ValueExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ValueExpr

    simple_map_expr(input).map(|(next_input, res)| (next_input, ValueExpr(res)))
}

pub struct ValueExpr(pub SimpleMapExpr);
