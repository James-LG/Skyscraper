//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-function-test

use crate::xpath::grammar::recipes::Res;

use super::sequence_type::{sequence_type, SequenceType};

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, multi::many0,
    sequence::tuple,
};

pub fn function_test(input: &str) -> Res<&str, FunctionTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-FunctionTest

    fn any_function_test(input: &str) -> Res<&str, FunctionTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AnyFunctionTest

        tuple((tag("function"), char('('), char('*'), char(')')))(input)
            .map(|(next_input, _res)| (next_input, FunctionTest::AnyFunctionTest))
    }

    fn typed_function_test_map(input: &str) -> Res<&str, FunctionTest> {
        typed_function_test(input)
            .map(|(next_input, res)| (next_input, FunctionTest::TypedFunctionTest(res)))
    }

    alt((any_function_test, typed_function_test_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum FunctionTest {
    AnyFunctionTest,
    TypedFunctionTest(TypedFunctionTest),
}

pub fn typed_function_test(input: &str) -> Res<&str, TypedFunctionTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-TypedFunctionTest

    tuple((
        tag("function"),
        char('('),
        opt(tuple((
            sequence_type,
            many0(tuple((char(','), sequence_type))),
        ))),
        char(')'),
        tag("as"),
        sequence_type,
    ))(input)
    .map(|(next_input, res)| {
        let mut params = Vec::new();
        if let Some(p) = res.2 {
            params.push(p.0);
            for q in p.1 {
                params.push(q.1);
            }
        }
        (
            next_input,
            TypedFunctionTest {
                params,
                ret_val: res.5,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct TypedFunctionTest {
    pub params: Vec<SequenceType>,
    pub ret_val: SequenceType,
}
