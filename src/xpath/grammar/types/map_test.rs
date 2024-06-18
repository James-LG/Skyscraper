//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-map-test>

use std::fmt::Display;

use nom::{branch::alt, bytes::complete::tag, character::complete::char, error::context};

use crate::xpath::grammar::{recipes::Res, whitespace_recipes::ws};

use super::{
    common::atomic_or_union_type,
    sequence_type::{sequence_type, SequenceType},
    AtomicOrUnionType,
};

pub fn map_test(input: &str) -> Res<&str, MapTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-MapTest

    fn any_map_test(input: &str) -> Res<&str, MapTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AnyMapTest

        ws((tag("map"), char('('), char('*'), char(')')))(input)
            .map(|(next_input, _res)| (next_input, MapTest::AnyMapTest))
    }

    fn typed_map_test_map(input: &str) -> Res<&str, MapTest> {
        typed_map_test(input).map(|(next_input, res)| (next_input, MapTest::TypedMapTest(res)))
    }

    context("map_test", alt((any_map_test, typed_map_test_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum MapTest {
    AnyMapTest,
    TypedMapTest(TypedMapTest),
}

impl Display for MapTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapTest::AnyMapTest => write!(f, "map(*)"),
            MapTest::TypedMapTest(x) => write!(f, "{}", x),
        }
    }
}

fn typed_map_test(input: &str) -> Res<&str, TypedMapTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-TypedMapTest

    context(
        "typed_map_test",
        ws((
            tag("map"),
            char('('),
            atomic_or_union_type,
            char(','),
            sequence_type,
            char(')'),
        )),
    )(input)
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

#[derive(PartialEq, Debug, Clone)]
pub struct TypedMapTest {
    pub atomic_or_union_type: AtomicOrUnionType,
    pub sequence_type: SequenceType,
}

impl Display for TypedMapTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "map({}, {})",
            self.atomic_or_union_type, self.sequence_type
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn map_test_should_parse_any() {
        // arrange
        let input = "map(*)";

        // act
        let (next_input, res) = map_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "map(*)");
    }

    #[test]
    fn map_test_should_parse_any_whitespace() {
        // arrange
        let input = "map ( * )";

        // act
        let (next_input, res) = map_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "map(*)");
    }

    #[test]
    fn map_test_should_parse_typed_name() {
        // arrange
        let input = "map(xs:integer,xs:integer)";

        // act
        let (next_input, res) = map_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "map(xs:integer, xs:integer)");
    }

    #[test]
    fn map_test_should_parse_typed_whitespace() {
        // arrange
        let input = "map ( xs:integer, xs:integer )";

        // act
        let (next_input, res) = map_test(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "map(xs:integer, xs:integer)");
    }
}
