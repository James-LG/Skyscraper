//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#combining_seq

use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        expressions::expressions_on_sequence_types::instance_of::{
            instanceof_expr, InstanceofExpr,
        },
        recipes::Res,
    },
    Expression, ExpressionApplyError, XPathExpressionContext, XPathResult,
};

pub fn union_expr(input: &str) -> Res<&str, UnionExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-UnionExpr

    fn union_operator_map(input: &str) -> Res<&str, UnionExprOperatorType> {
        tag("union")(input).map(|(next_input, _res)| (next_input, UnionExprOperatorType::Union))
    }

    fn bar_operator_map(input: &str) -> Res<&str, UnionExprOperatorType> {
        tag("|")(input).map(|(next_input, _res)| (next_input, UnionExprOperatorType::Bar))
    }

    context(
        "union_expr",
        tuple((
            intersect_except_expr,
            many0(tuple((
                alt((union_operator_map, bar_operator_map)),
                intersect_except_expr,
            ))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let items = res
            .1
            .into_iter()
            .map(|res| UnionExprPair(res.0, res.1))
            .collect();
        (next_input, UnionExpr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnionExpr {
    pub expr: IntersectExceptExpr,
    pub items: Vec<UnionExprPair>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnionExprPair(UnionExprOperatorType, pub IntersectExceptExpr);

impl Display for UnionExprPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

impl Expression for UnionExpr {
    fn eval<'tree>(
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
        todo!("UnionExpr::eval union operator")
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum UnionExprOperatorType {
    Union,
    Bar,
}

impl Display for UnionExprOperatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnionExprOperatorType::Union => write!(f, "union"),
            UnionExprOperatorType::Bar => write!(f, "|"),
        }
    }
}

impl Display for UnionExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, "{}", x)?
        }

        Ok(())
    }
}

fn intersect_except_expr(input: &str) -> Res<&str, IntersectExceptExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-IntersectExceptExpr

    fn intersect(input: &str) -> Res<&str, IntersectExceptType> {
        tag("intersect")(input)
            .map(|(next_input, _res)| (next_input, IntersectExceptType::Intersect))
    }

    fn except(input: &str) -> Res<&str, IntersectExceptType> {
        tag("except")(input).map(|(next_input, _res)| (next_input, IntersectExceptType::Except))
    }

    context(
        "intersect_except_expr",
        tuple((
            instanceof_expr,
            many0(tuple((alt((intersect, except)), instanceof_expr))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let items = res
            .1
            .into_iter()
            .map(|res| IntersectExceptPair(res.0, res.1))
            .collect();
        (next_input, IntersectExceptExpr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct IntersectExceptExpr {
    pub expr: InstanceofExpr,
    pub items: Vec<IntersectExceptPair>,
}

impl Display for IntersectExceptExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " {}", x)?
        }

        Ok(())
    }
}

impl Expression for IntersectExceptExpr {
    fn eval<'tree>(
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
        todo!("IntersectExceptExpr::eval intersect or except operator")
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct IntersectExceptPair(pub IntersectExceptType, pub InstanceofExpr);

impl Display for IntersectExceptPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum IntersectExceptType {
    Intersect,
    Except,
}

impl Display for IntersectExceptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntersectExceptType::Intersect => write!(f, "intersect"),
            IntersectExceptType::Except => write!(f, "except"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn union_expr_should_parse1() {
        // arrange
        let input = "chapter|appendix";

        // act
        let (next_input, res) = union_expr(input).unwrap();

        // assert
        assert_eq!(res.to_string(), input);
        assert_eq!(next_input, "");
    }
}
