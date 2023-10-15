use nom::{branch::alt, bytes::complete::tag, character::complete::char, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::sequence_type::{sequence_type, SequenceType};

pub fn array_test(input: &str) -> Res<&str, ArrayTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ArrayTest

    fn any_array_test_(input: &str) -> Res<&str, ArrayTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AnyArrayTest

        tuple((tag("map"), char('('), char('*'), char(')')))(input)
            .map(|(next_input, _res)| (next_input, ArrayTest::AnyArrayTest))
    }

    fn typed_array_test_map(input: &str) -> Res<&str, ArrayTest> {
        typed_array_test(input)
            .map(|(next_input, res)| (next_input, ArrayTest::TypedArrayTest(res)))
    }

    alt((any_array_test_, typed_array_test_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum ArrayTest {
    AnyArrayTest,
    TypedArrayTest(TypedArrayTest),
}

fn typed_array_test(input: &str) -> Res<&str, TypedArrayTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-TypedArrayTest

    tuple((tag("array"), char('('), sequence_type, char(')')))(input)
        .map(|(next_input, res)| (next_input, TypedArrayTest(res.2)))
}

#[derive(PartialEq, Debug)]
pub struct TypedArrayTest(pub SequenceType);
