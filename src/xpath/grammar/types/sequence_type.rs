//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-sequencetype-syntax

use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{opt, recognize},
    error::context,
    sequence::tuple,
};

use crate::xpath::{
    grammar::{
        recipes::Res,
        types::{
            array_test::array_test, common::atomic_or_union_type, function_test::function_test,
            kind_test, map_test::map_test,
        },
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::{
    array_test::ArrayTest, function_test::FunctionTest, map_test::MapTest, AtomicOrUnionType,
    KindTest,
};

pub fn sequence_type(input: &str) -> Res<&str, SequenceType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SequenceType

    fn empty_sequence_map(input: &str) -> Res<&str, SequenceType> {
        tuple((tag("empty-sequence"), char('('), char(')')))(input)
            .map(|(next_input, _res)| (next_input, SequenceType::EmptySequence))
    }

    fn sequence_value_map(input: &str) -> Res<&str, SequenceType> {
        tuple((item_type, opt(occurrence_indicator)))(input).map(|(next_input, res)| {
            (
                next_input,
                SequenceType::Sequence(SequenceTypeValue {
                    item_type: res.0,
                    occurrence: res.1,
                }),
            )
        })
    }

    context(
        "sequence_type",
        alt((empty_sequence_map, sequence_value_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum SequenceType {
    EmptySequence,
    Sequence(SequenceTypeValue),
}

impl Display for SequenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SequenceType::EmptySequence => write!(f, "empty-sequence()"),
            SequenceType::Sequence(x) => write!(f, "{}", x),
        }
    }
}

impl SequenceType {
    pub(crate) fn eval<'tree>(
        &self,
        item_set: &XpathItemSet,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<bool, ExpressionApplyError> {
        match self {
            // The sequence type empty-sequence() matches a value that is the empty sequence.
            SequenceType::EmptySequence => Ok(item_set.is_empty()),
            SequenceType::Sequence(x) => match x.occurrence {
                Some(_) => todo!("SequenceType::Sequence::is_match occurrence"),

                // An ItemType with no OccurrenceIndicator matches any value that contains exactly one item if the ItemType matches that item.
                None => {
                    let item_type_result = x.item_type.eval(context)?;
                    Ok(item_set.len() == 1 && item_type_result)
                }
            },
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct SequenceTypeValue {
    pub item_type: ItemType,
    pub occurrence: Option<OccurrenceIndicator>,
}

impl Display for SequenceTypeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.item_type)?;
        if let Some(x) = &self.occurrence {
            write!(f, "{}", x)?;
        }

        Ok(())
    }
}

pub fn item_type(input: &str) -> Res<&str, ItemType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ItemType

    fn item_map(input: &str) -> Res<&str, ItemType> {
        recognize(tuple((tag("item"), char('('), char(')'))))(input)
            .map(|(next_input, _res)| (next_input, ItemType::Item))
    }

    fn kind_test_map(input: &str) -> Res<&str, ItemType> {
        kind_test(input).map(|(next_input, res)| (next_input, ItemType::KindTest(res)))
    }

    fn function_test_map(input: &str) -> Res<&str, ItemType> {
        function_test(input)
            .map(|(next_input, res)| (next_input, ItemType::FunctionTest(Box::new(res))))
    }

    fn map_test_map(input: &str) -> Res<&str, ItemType> {
        map_test(input).map(|(next_input, res)| (next_input, ItemType::MapTest(Box::new(res))))
    }

    fn array_test_map(input: &str) -> Res<&str, ItemType> {
        array_test(input).map(|(next_input, res)| (next_input, ItemType::ArrayTest(Box::new(res))))
    }

    fn atomic_or_union_type_map(input: &str) -> Res<&str, ItemType> {
        atomic_or_union_type(input)
            .map(|(next_input, res)| (next_input, ItemType::AtomicOrUnionType(res)))
    }

    context(
        "item_type",
        alt((
            kind_test_map,
            item_map,
            function_test_map,
            map_test_map,
            array_test_map,
            atomic_or_union_type_map,
            parenthesized_item_type,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ItemType {
    Item,
    KindTest(KindTest),
    FunctionTest(Box<FunctionTest>),
    MapTest(Box<MapTest>),
    ArrayTest(Box<ArrayTest>),
    AtomicOrUnionType(AtomicOrUnionType),
}

impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Item => write!(f, "item()"),
            ItemType::KindTest(x) => write!(f, "{}", x),
            ItemType::FunctionTest(x) => write!(f, "{}", x),
            ItemType::MapTest(x) => write!(f, "{}", x),
            ItemType::ArrayTest(x) => write!(f, "{}", x),
            ItemType::AtomicOrUnionType(x) => write!(f, "{}", x),
        }
    }
}

impl ItemType {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<bool, ExpressionApplyError> {
        match self {
            // item() matches any single item.
            ItemType::Item => Ok(true),
            ItemType::KindTest(x) => {
                let result = x.eval(context)?;
                Ok(!result.is_none())
            }
            ItemType::FunctionTest(_x) => todo!("ItemType::FunctionTest::is_match"),
            ItemType::MapTest(_x) => todo!("ItemType::MapTest::is_match"),
            ItemType::ArrayTest(_x) => todo!("ItemType::ArrayTest::is_match"),
            ItemType::AtomicOrUnionType(_x) => todo!("ItemType::AtomicOrUnionType::is_match"),
        }
    }
}

pub fn parenthesized_item_type(input: &str) -> Res<&str, ItemType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-ParenthesizedItemType
    context(
        "parenthesized_item_type",
        tuple((char('('), item_type, char(')'))),
    )(input)
    .map(|(next_input, res)| (next_input, res.1))
}

pub fn occurrence_indicator(input: &str) -> Res<&str, OccurrenceIndicator> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-OccurrenceIndicator

    fn zero_or_one_map(input: &str) -> Res<&str, OccurrenceIndicator> {
        char('?')(input).map(|(next_input, _res)| (next_input, OccurrenceIndicator::ZeroOrOne))
    }

    fn zero_or_more_map(input: &str) -> Res<&str, OccurrenceIndicator> {
        char('*')(input).map(|(next_input, _res)| (next_input, OccurrenceIndicator::ZeroOrMore))
    }

    fn one_or_more_map(input: &str) -> Res<&str, OccurrenceIndicator> {
        char('+')(input).map(|(next_input, _res)| (next_input, OccurrenceIndicator::OneOrMore))
    }

    context(
        "occurrence_indicator",
        alt((zero_or_one_map, zero_or_more_map, one_or_more_map)),
    )(input)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum OccurrenceIndicator {
    ZeroOrOne,
    ZeroOrMore,
    OneOrMore,
}

impl Display for OccurrenceIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OccurrenceIndicator::ZeroOrOne => write!(f, "?"),
            OccurrenceIndicator::ZeroOrMore => write!(f, "*"),
            OccurrenceIndicator::OneOrMore => write!(f, "+"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::xpath::grammar::{
        types::{
            common::ElementName,
            element_test::{ElementNameOrWildcard, ElementTest},
            DocumentTest, DocumentTestValue, EQName, PITest, PITestValue,
        },
        xml_names::QName,
    };

    use super::*;

    #[test]
    fn item_type_example1() {
        // arrange
        let input = "item()";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(res, Ok(("", ItemType::Item)))
    }

    #[test]
    fn item_type_example2() {
        // arrange
        let input = "node()";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(res, Ok(("", ItemType::KindTest(KindTest::AnyKindTest))))
    }

    #[test]
    fn item_type_example3() {
        // arrange
        let input = "text()";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(res, Ok(("", ItemType::KindTest(KindTest::TextTest))))
    }

    #[test]
    fn item_type_example4() {
        // arrange
        let input = "processing-instruction()";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(
            res,
            Ok((
                "",
                ItemType::KindTest(KindTest::PITest(PITest { val: None }))
            ))
        )
    }

    #[test]
    fn item_type_example5() {
        // arrange
        let input = "processing-instruction(N)";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(
            res,
            Ok((
                "",
                ItemType::KindTest(KindTest::PITest(PITest {
                    val: Some(PITestValue::NCName(String::from("N")))
                }))
            ))
        )
    }

    #[test]
    fn item_type_example6() {
        // arrange
        let input = "comment()";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(res, Ok(("", ItemType::KindTest(KindTest::CommentTest))))
    }

    #[test]
    fn item_type_example7() {
        // arrange
        let input = "namespace-node()";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(
            res,
            Ok(("", ItemType::KindTest(KindTest::NamespaceNodeTest)))
        )
    }

    #[test]
    fn item_type_example8() {
        // arrange
        let input = "document-node()";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(
            res,
            Ok((
                "",
                ItemType::KindTest(KindTest::DocumentTest(DocumentTest { value: None }))
            ))
        )
    }

    #[test]
    fn item_type_example9() {
        // arrange
        let input = "document-node(element(book))";

        // act
        let res = item_type(input);

        // assert
        assert_eq!(
            res,
            Ok((
                "",
                ItemType::KindTest(KindTest::DocumentTest(DocumentTest {
                    value: Some(DocumentTestValue::ElementTest(ElementTest {
                        name_or_wildcard: Some(ElementNameOrWildcard::ElementName(ElementName(
                            EQName::QName(QName::UnprefixedName(String::from("book")))
                        ))),
                        type_name: None
                    }))
                }))
            ))
        )
    }
}
