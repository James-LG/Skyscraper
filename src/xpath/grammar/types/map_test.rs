//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-map-test

use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, character::complete::char, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::{
    common::atomic_or_union_type,
    sequence_type::{sequence_type, SequenceType},
    AtomicOrUnionType,
};

pub fn map_test(input: &str) -> Res<&str, MapTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-MapTest

    fn any_map_test(input: &str) -> Res<&str, MapTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AnyMapTest

        tuple((tag("map"), char('('), char('*'), char(')')))(input)
            .map(|(next_input, _res)| (next_input, MapTest::AnyMapTest))
    }

    fn typed_map_test_map(input: &str) -> Res<&str, MapTest> {
        typed_map_test(input).map(|(next_input, res)| (next_input, MapTest::TypedMapTest(res)))
    }

    alt((any_map_test, typed_map_test_map))(input)
}

#[derive(PartialEq, Debug)]
pub enum MapTest {
    AnyMapTest,
    TypedMapTest(TypedMapTest),
}

impl Display for MapTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt MapTest")
    }
}

fn typed_map_test(input: &str) -> Res<&str, TypedMapTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-TypedMapTest

    tuple((
        tag("map"),
        char('('),
        atomic_or_union_type,
        char(','),
        sequence_type,
        char(')'),
    ))(input)
    .map(|(next_input, res)| {
        (
            next_input,
            TypedMapTest {
                atomic_or_union_type: res.2,
                sequence_type: res.4,
            },
        )
    })
}

#[derive(PartialEq, Debug)]
pub struct TypedMapTest {
    pub atomic_or_union_type: AtomicOrUnionType,
    pub sequence_type: SequenceType,
}
