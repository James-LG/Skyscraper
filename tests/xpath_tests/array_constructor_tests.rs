use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// Square array constructor with integer lookup.
/// `let $a := [10, 20, 30] return $a(2)` should return 20.
#[test]
fn square_array_lookup() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"let $a := [10, 20, 30] return $a(2)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(20))
    );
}

/// Square array constructor: first element.
#[test]
fn square_array_lookup_first() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"let $a := [10, 20, 30] return $a(1)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(10))
    );
}

/// Square array constructor: last element.
#[test]
fn square_array_lookup_last() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"let $a := [10, 20, 30] return $a(3)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(30))
    );
}

/// Square array with string members.
#[test]
fn square_array_string_members() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $a := ["hello", "world"] return $a(1)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("hello".to_string()))
    );
}

/// Empty square array: out-of-bounds should error.
#[test]
fn square_array_empty_out_of_bounds() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"let $a := [] return $a(1)"#).unwrap();

    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "out-of-bounds on empty array should error: {result:?}"
    );
}

/// Array index out of bounds should error.
#[test]
fn square_array_index_out_of_bounds() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"let $a := [1, 2] return $a(3)"#).unwrap();

    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "out-of-bounds index should error: {result:?}"
    );
}

/// Array index 0 should error (1-indexed).
#[test]
fn square_array_index_zero_errors() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"let $a := [1, 2] return $a(0)"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "index 0 should error: {result:?}");
}

/// Curly array constructor: `array { 1, 2, 3 }`.
/// Each item in the sequence becomes one member.
#[test]
fn curly_array_lookup() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $a := array { 1, 2, 3 } return $a(2)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
}

/// Curly array with empty expression produces empty array.
#[test]
fn curly_array_empty() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"let $a := array {} return $a(1)"#).unwrap();

    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "out-of-bounds on empty curly array should error: {result:?}"
    );
}
