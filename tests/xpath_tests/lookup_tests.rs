use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

// --- Map postfix lookup tests ---

/// Postfix lookup on a map with a name key: `$m?x`.
#[test]
fn postfix_lookup_map_name_key() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $m := map { "x": 42 } return $m?x"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(42))
    );
}

/// Postfix lookup on a map with integer key.
#[test]
fn postfix_lookup_map_integer_key() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $m := map { 1: "one", 2: "two" } return $m?2"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("two".to_string()))
    );
}

/// Postfix lookup on a map with wildcard: returns all values.
#[test]
fn postfix_lookup_map_wildcard() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $m := map { "a": 1, "b": 2 } return $m?*"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 items: {items:?}");
    assert!(items.iter().any(|i| *i == XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))));
    assert!(items.iter().any(|i| *i == XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))));
}

/// Postfix lookup on a map with missing key returns empty.
#[test]
fn postfix_lookup_map_missing_key() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $m := map { "x": 1 } return $m?y"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "missing key should return empty: {items:?}");
}

// --- Array postfix lookup tests ---

/// Postfix lookup on an array with integer key.
#[test]
fn postfix_lookup_array_integer_key() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $a := [10, 20, 30] return $a?2"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(20))
    );
}

/// Postfix lookup on an array with wildcard: returns all members.
#[test]
fn postfix_lookup_array_wildcard() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $a := [10, 20, 30] return $a?*"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3, "should return 3 items: {items:?}");
    assert!(items.iter().any(|i| *i == XpathItem::AnyAtomicType(AnyAtomicType::Integer(10))));
    assert!(items.iter().any(|i| *i == XpathItem::AnyAtomicType(AnyAtomicType::Integer(20))));
    assert!(items.iter().any(|i| *i == XpathItem::AnyAtomicType(AnyAtomicType::Integer(30))));
}

/// Postfix lookup on an array with out-of-bounds index should error.
#[test]
fn postfix_lookup_array_out_of_bounds() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $a := [10, 20] return $a?3"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "out-of-bounds should error: {result:?}");
}

/// Array does not support named key lookup.
#[test]
fn postfix_lookup_array_name_key_errors() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $a := [10, 20] return $a?foo"#).unwrap();

    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "named key on array should error: {result:?}"
    );
}
