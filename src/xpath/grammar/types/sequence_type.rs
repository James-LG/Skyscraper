//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-sequencetype-syntax>

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
        data_model::{AnyAtomicType, Function, XpathItem},
        recipes::Res,
        types::{
            array_test::array_test, common::atomic_or_union_type, function_test::function_test,
            kind_test, map_test::map_test,
        },
        whitespace_recipes::ws,
        XpathItemTree,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError,
};

use super::{
    array_test::{ArrayTest, TypedArrayTest},
    function_test::{FunctionTest, TypedFunctionTest},
    map_test::{MapTest, TypedMapTest},
    AtomicOrUnionType, KindTest,
};

pub fn sequence_type(input: &str) -> Res<&str, SequenceType> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SequenceType

    fn empty_sequence_map(input: &str) -> Res<&str, SequenceType> {
        ws((tag("empty-sequence"), char('('), char(')')))(input)
            .map(|(next_input, _res)| (next_input, SequenceType::EmptySequence))
    }

    fn sequence_value_map(input: &str) -> Res<&str, SequenceType> {
        ws((item_type, opt(occurrence_indicator)))(input).map(|(next_input, res)| {
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
    pub(crate) fn is_match<'tree>(
        &self,
        item_set: &XpathItemSet<'tree>,
        item_tree: &'tree XpathItemTree,
    ) -> Result<bool, ExpressionApplyError> {
        match self {
            // The sequence type empty-sequence() matches a value that is the empty sequence.
            SequenceType::EmptySequence => Ok(item_set.is_empty()),
            SequenceType::Sequence(x) => {
                let cardinality_ok = match x.occurrence {
                    // No indicator: exactly one item.
                    None => item_set.len() == 1,
                    Some(OccurrenceIndicator::ZeroOrOne) => item_set.len() <= 1,
                    Some(OccurrenceIndicator::ZeroOrMore) => true,
                    Some(OccurrenceIndicator::OneOrMore) => !item_set.is_empty(),
                };

                if !cardinality_ok {
                    return Ok(false);
                }

                // Every item in the sequence must match the ItemType.
                if item_set.is_empty() {
                    return Ok(true);
                }

                for item in item_set {
                    let single = crate::xpath_item_set![item.clone()];
                    if !x.item_type.is_match(&single, item_tree)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
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

/// Check whether a local type name is a recognized XSD built-in atomic type.
fn is_recognized_xsd_atomic_type(name: &str) -> bool {
    matches!(
        name,
        // Implemented types
        "integer" | "string" | "boolean" | "float" | "double"
        // Abstract / special types
        | "anyAtomicType" | "untypedAtomic"
        // Union type (XPath 3.1)
        | "numeric"
        // Numeric types
        | "decimal"
        // Integer subtypes
        | "nonPositiveInteger" | "negativeInteger" | "long" | "int"
        | "short" | "byte" | "nonNegativeInteger" | "unsignedLong"
        | "unsignedInt" | "unsignedShort" | "unsignedByte" | "positiveInteger"
        // String subtypes
        | "normalizedString" | "token" | "language" | "NMTOKEN" | "Name"
        | "NCName" | "ID" | "IDREF" | "ENTITY"
        // Date/time types
        | "date" | "dateTime" | "time" | "duration"
        | "yearMonthDuration" | "dayTimeDuration"
        | "gYearMonth" | "gYear" | "gMonthDay" | "gDay" | "gMonth"
        // Other types
        | "QName" | "anyURI" | "base64Binary" | "hexBinary" | "NOTATION"
        // XSD 1.1 / XPath 3.1 special type (no value space)
        | "error"
    )
}

/// Check if an atomic value matches a type name.
///
/// Returns `Ok(true)` if the atomic value is of the given type, `Ok(false)` if
/// it is a recognized type that doesn't match, or `Err` with `err:XPST0051`
/// if the type name is not a recognized XSD built-in atomic type.
fn atomic_matches_type_name(
    atomic: &AnyAtomicType,
    type_name: &AtomicOrUnionType,
) -> Result<bool, ExpressionApplyError> {
    match type_name.local_name() {
        // Implemented types
        Some("integer") => Ok(matches!(atomic, AnyAtomicType::Integer(_))),
        Some("string") => Ok(matches!(atomic, AnyAtomicType::String(_))),
        Some("boolean") => Ok(matches!(atomic, AnyAtomicType::Boolean(_))),
        Some("float") => Ok(matches!(atomic, AnyAtomicType::Float(_))),
        Some("double") => Ok(matches!(atomic, AnyAtomicType::Double(_))),

        // xs:anyAtomicType — matches any atomic value.
        Some("anyAtomicType") => Ok(true),

        // xs:decimal — xs:integer is a subtype of xs:decimal per XSD type hierarchy.
        Some("decimal") => Ok(matches!(atomic, AnyAtomicType::Integer(_))),

        // xs:numeric — union of xs:double, xs:float, xs:decimal (and subtypes).
        Some("numeric") => Ok(matches!(
            atomic,
            AnyAtomicType::Integer(_) | AnyAtomicType::Float(_) | AnyAtomicType::Double(_)
        )),

        // Recognized but unimplemented types — no values of these types exist
        // in this implementation, so no atomic value can match them.
        Some(name) if is_recognized_xsd_atomic_type(name) => Ok(false),

        // Unknown type name — raise err:XPST0051.
        // Per spec this is a static error, but we raise it at evaluation time
        // since the implementation has no separate static analysis pass.
        _ => Err(ExpressionApplyError::new(format!(
            "err:XPST0051 Unknown atomic type '{type_name}'"
        ))),
    }
}

/// Get the arity of a function item.
fn function_arity(f: &Function) -> u32 {
    match f {
        Function::Named { arity, .. } => *arity,
        Function::Inline { params, .. } => params.len() as u32,
        Function::Map { .. } => 1,
        Function::Array { .. } => 1,
    }
}

impl ItemType {
    pub(crate) fn is_match<'tree>(
        &self,
        item_set: &XpathItemSet<'tree>,
        item_tree: &'tree XpathItemTree,
    ) -> Result<bool, ExpressionApplyError> {
        match self {
            // item() matches any single item.
            ItemType::Item => Ok(true),
            ItemType::KindTest(x) => {
                let result = x.filter(item_set, item_tree)?;
                Ok(!result.is_empty())
            }
            ItemType::FunctionTest(x) => match x.as_ref() {
                FunctionTest::AnyFunctionTest => {
                    // function(*) matches any function item.
                    Ok(item_set
                        .iter()
                        .all(|item| matches!(item, XpathItem::Function(_))))
                }
                FunctionTest::TypedFunctionTest(typed) => {
                    // function(T1, T2, ...) as R matches a function with matching arity.
                    // We validate arity against the number of declared parameter types.
                    // Full parameter/return type covariance/contravariance checking is
                    // not possible for Named/Inline since we don't store type signatures.
                    Self::is_match_typed_function_test(item_set, typed)
                }
            },
            ItemType::MapTest(x) => match x.as_ref() {
                MapTest::AnyMapTest => {
                    // map(*) matches any map.
                    Ok(item_set
                        .iter()
                        .all(|item| matches!(item, XpathItem::Function(Function::Map { .. }))))
                }
                MapTest::TypedMapTest(typed) => {
                    // map(K, V) matches maps whose keys are all of type K
                    // and whose values all match sequence type V.
                    Self::is_match_typed_map_test(item_set, item_tree, typed)
                }
            },
            ItemType::ArrayTest(x) => match x.as_ref() {
                ArrayTest::AnyArrayTest => {
                    // array(*) matches any array.
                    Ok(item_set
                        .iter()
                        .all(|item| matches!(item, XpathItem::Function(Function::Array { .. }))))
                }
                ArrayTest::TypedArrayTest(typed) => {
                    // array(T) matches arrays whose members all match sequence type T.
                    Self::is_match_typed_array_test(item_set, item_tree, typed)
                }
            },
            ItemType::AtomicOrUnionType(x) => {
                for item in item_set {
                    match item {
                        XpathItem::AnyAtomicType(atomic) => {
                            if !atomic_matches_type_name(atomic, x)? {
                                return Ok(false);
                            }
                        }
                        _ => return Ok(false),
                    }
                }
                Ok(true)
            }
        }
    }

    /// Typed function test: `function(P1, P2, ...) as R`.
    ///
    /// Validates that each item is a function whose arity equals the number of
    /// declared parameter types.
    fn is_match_typed_function_test(
        item_set: &XpathItemSet<'_>,
        typed: &TypedFunctionTest,
    ) -> Result<bool, ExpressionApplyError> {
        let expected_arity = typed.params.len() as u32;
        Ok(item_set.iter().all(|item| {
            if let XpathItem::Function(f) = item {
                function_arity(f) == expected_arity
            } else {
                false
            }
        }))
    }

    /// Typed map test: `map(K, V)`.
    ///
    /// Validates that each item is a map whose keys all match atomic type K
    /// and whose value sequences all match sequence type V.
    fn is_match_typed_map_test<'tree>(
        item_set: &XpathItemSet<'tree>,
        item_tree: &'tree XpathItemTree,
        typed: &TypedMapTest,
    ) -> Result<bool, ExpressionApplyError> {
        for item in item_set.iter() {
            match item {
                XpathItem::Function(Function::Map { entries }) => {
                    for (key, values) in entries {
                        if !atomic_matches_type_name(key, &typed.atomic_or_union_type)? {
                            return Ok(false);
                        }
                        let value_set: XpathItemSet = values
                            .iter()
                            .map(|v| XpathItem::AnyAtomicType(v.clone()))
                            .collect();
                        if !typed.sequence_type.is_match(&value_set, item_tree)? {
                            return Ok(false);
                        }
                    }
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    /// Typed array test: `array(T)`.
    ///
    /// Validates that each item is an array whose members all match sequence type T.
    fn is_match_typed_array_test<'tree>(
        item_set: &XpathItemSet<'tree>,
        item_tree: &'tree XpathItemTree,
        typed: &TypedArrayTest,
    ) -> Result<bool, ExpressionApplyError> {
        for item in item_set.iter() {
            match item {
                XpathItem::Function(Function::Array { members }) => {
                    for member in members {
                        let member_set: XpathItemSet = member
                            .iter()
                            .map(|v| XpathItem::AnyAtomicType(v.clone()))
                            .collect();
                        if !typed.0.is_match(&member_set, item_tree)? {
                            return Ok(false);
                        }
                    }
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
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
            element_test::{ElementNameOrWildcard, ElementTest, ElementTestItem},
            DocumentTest, DocumentTestValue, EQName, PITest, PITestValue,
        },
        xml_names::QName,
    };

    use super::*;

    #[test]
    fn sequence_type_test_should_parse_empty() {
        // arrange
        let input = "empty-sequence()";

        // act
        let (next_input, res) = sequence_type(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "empty-sequence()");
    }

    #[test]
    fn sequence_type_should_parse_empty_whitespace() {
        // arrange
        let input = "empty-sequence ( )";

        // act
        let (next_input, res) = sequence_type(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "empty-sequence()");
    }

    #[test]
    fn sequence_type_test_should_parse_value() {
        // arrange
        let input = "item()?";

        // act
        let (next_input, res) = sequence_type(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "item()?");
    }

    #[test]
    fn sequence_type_should_parse_value_whitespace() {
        // arrange
        let input = "item() ?";

        // act
        let (next_input, res) = sequence_type(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "item()?");
    }

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
                        item: Some(ElementTestItem {
                            element_name_or_wildcard: ElementNameOrWildcard::ElementName(
                                ElementName(EQName::QName(QName::UnprefixedName(String::from(
                                    "book"
                                ))))
                            ),
                            type_name: None
                        })
                    }))
                }))
            ))
        )
    }
}
