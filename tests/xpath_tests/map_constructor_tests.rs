use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// Basic map constructor with string keys and integer values.
/// `let $m := map { "x": 1, "y": 2 } return $m("x")` should return 1.
#[test]
fn map_constructor_basic_lookup() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $m := map { "x": 1, "y": 2 } return $m("x")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}

/// Map lookup with second key.
#[test]
fn map_constructor_lookup_second_key() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $m := map { "x": 1, "y": 2 } return $m("y")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
}

/// Map lookup with missing key should return empty sequence.
#[test]
fn map_constructor_missing_key() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $m := map { "x": 1 } return $m("z")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "missing key should return empty: {items:?}");
}

/// Empty map constructor.
#[test]
fn map_constructor_empty() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $m := map {} return $m("x")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "empty map lookup should return empty: {items:?}");
}

/// Map with integer keys.
#[test]
fn map_constructor_integer_keys() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $m := map { 1: "one", 2: "two" } return $m(1)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("one".to_string()))
    );
}

/// Map with string values.
#[test]
fn map_constructor_string_values() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $m := map { "greeting": "hello world" } return $m("greeting")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("hello world".to_string()))
    );
}

/// Map constructor with duplicate keys should error (err:XQDY0137).
#[test]
fn map_constructor_duplicate_key_errors() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $m := map { "x": 1, "x": 2 } return $m("x")"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "duplicate key should produce an error: {result:?}");
}

/// Map constructor should parse and the map item should be assignable to a variable.
#[test]
fn map_constructor_as_variable() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    // Just verify that creating a map and returning it doesn't panic.
    let xpath = xpath::parse(r#"let $m := map { "a": 1 } return $m("a")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}
