use ordered_float::OrderedFloat;
use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// Cast integer to string: `42 cast as string`.
#[test]
fn cast_integer_to_string() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 cast as string").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("42")))
    );
}

/// Cast string to integer: `"123" cast as integer`.
#[test]
fn cast_string_to_integer() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""123" cast as integer"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(123))
    );
}

/// Cast integer to double: `42 cast as double`.
#[test]
fn cast_integer_to_double() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 cast as double").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(OrderedFloat(42.0)))
    );
}

/// Cast boolean to integer: `(1 > 0) cast as integer` (true → 1).
#[test]
fn cast_boolean_to_integer() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(1 > 0) cast as integer").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}

/// Cast string to boolean: `"true" cast as boolean`.
#[test]
fn cast_string_to_boolean() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""true" cast as boolean"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Cast with `?` and empty sequence should return empty.
#[test]
fn cast_empty_with_question_mark() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//nonexistent cast as integer?").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

/// Invalid cast should error: `"abc" cast as integer`.
#[test]
fn cast_invalid_string_to_integer_fails() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""abc" cast as integer"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err());
}

/// No cast clause — just return the base expression.
#[test]
fn cast_no_clause_returns_base() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(42))
    );
}
