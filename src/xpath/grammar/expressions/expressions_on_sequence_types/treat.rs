//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-treat

use std::fmt::Display;

use nom::{bytes::complete::tag, combinator::opt, error::context, sequence::tuple};

use crate::xpath::{
    grammar::{
        recipes::{ws, Res},
        types::sequence_type::{sequence_type, SequenceType},
    },
    Expression, ExpressionApplyError, XPathExpressionContext, XPathResult,
};

use super::castable::{castable_expr, CastableExpr};

pub fn treat_expr(input: &str) -> Res<&str, TreatExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-TreatExpr

    context(
        "treat_expr",
        tuple((
            castable_expr,
            opt(tuple((ws(tag("treat")), ws(tag("as")), sequence_type))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let treat_type = res.1.map(|res| res.2);
        (
            next_input,
            TreatExpr {
                expr: res.0,
                treat_type,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct TreatExpr {
    pub expr: CastableExpr,
    pub treat_type: Option<SequenceType>,
}

impl Display for TreatExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(x) = &self.treat_type {
            write!(f, " treat as {}", x)?;
        }

        Ok(())
    }
}

impl Expression for TreatExpr {
    fn eval<'tree>(
        &self,
        context: &XPathExpressionContext<'tree>,
    ) -> Result<XPathResult<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        let treat_type = match &self.treat_type {
            // If there's only one parameter, return it's eval.
            None => return Ok(result),
            // Otherwise, do the operation.
            Some(treat_type) => treat_type,
        };

        if !treat_type.eval(&result, context)? {
            return Err(ExpressionApplyError {
                msg: format!(
                    "err:XPDY0050 Cannot treat {:?} as {}",
                    result,
                    treat_type.to_string()
                ),
            });
        }

        Ok(result)
    }
}
