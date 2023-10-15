//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-unary-lookup

use nom::{branch::alt, character::complete::char, sequence::tuple};

use crate::xpath::grammar::{
    expressions::primary_expressions::parenthesized_expressions::{
        parenthesized_expr, ParenthesizedExpr,
    },
    recipes::Res,
    terminal_symbols::integer_literal,
    xml_names::nc_name,
};

pub fn unary_lookup(input: &str) -> Res<&str, UnaryLookup> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-UnaryLookup
    tuple((char('?'), key_specifier))(input)
        .map(|(next_input, res)| (next_input, UnaryLookup(res.1)))
}

pub struct UnaryLookup(pub KeySpecifier);

pub fn key_specifier(input: &str) -> Res<&str, KeySpecifier> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-KeySpecifier

    fn nc_name_map(input: &str) -> Res<&str, KeySpecifier> {
        nc_name(input).map(|(next_input, res)| (next_input, KeySpecifier::Name(res.to_string())))
    }

    fn integer_literal_map(input: &str) -> Res<&str, KeySpecifier> {
        integer_literal(input).map(|(next_input, res)| (next_input, KeySpecifier::Integer(res)))
    }

    fn parenthesized_expr_map(input: &str) -> Res<&str, KeySpecifier> {
        parenthesized_expr(input)
            .map(|(next_input, res)| (next_input, KeySpecifier::ParenthesizedExpr(res)))
    }

    fn wildcard_map(input: &str) -> Res<&str, KeySpecifier> {
        char('*')(input).map(|(next_input, _res)| (next_input, KeySpecifier::Wildcard))
    }

    alt((
        nc_name_map,
        integer_literal_map,
        parenthesized_expr_map,
        wildcard_map,
    ))(input)
}

pub enum KeySpecifier {
    Name(String),
    Integer(u32),
    ParenthesizedExpr(ParenthesizedExpr),
    Wildcard,
}
