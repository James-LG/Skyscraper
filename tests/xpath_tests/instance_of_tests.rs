use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// A node is an instance of `node()`.
#[test]
fn instance_of_node_true() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("/html instance of node()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// An integer is not an instance of `node()`.
#[test]
fn instance_of_node_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of node()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// Anything is an instance of `item()`.
#[test]
fn instance_of_item_true() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of item()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A text node is an instance of `text()`.
#[test]
fn instance_of_text_true() {
    let text = r#"<html><body>hello</body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/text() instance of text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// An element node is not an instance of `text()`.
#[test]
fn instance_of_text_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("/html instance of text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// Empty sequence matches `empty-sequence()`.
#[test]
fn instance_of_empty_sequence_true() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//nonexistent instance of empty-sequence()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Non-empty sequence does not match `empty-sequence()`.
#[test]
fn instance_of_empty_sequence_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("/html instance of empty-sequence()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// Using `instance of` with `if`: `if (42 instance of item()) then 1 else 0`.
#[test]
fn instance_of_with_if() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (42 instance of item()) then 1 else 0").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}

// ============================================================
// Typed map tests: map(K, V)
// ============================================================

/// A map with integer keys and string values matches `map(xs:integer, xs:string)`.
#[test]
fn instance_of_typed_map_matching() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"map { 1: "a", 2: "b" } instance of map(xs:integer, xs:string)"#)
            .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A map with string keys does not match `map(xs:integer, xs:string)`.
#[test]
fn instance_of_typed_map_wrong_key_type() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"map { "x": "a", "y": "b" } instance of map(xs:integer, xs:string)"#)
            .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// A map with integer values does not match `map(xs:integer, xs:string)`.
#[test]
fn instance_of_typed_map_wrong_value_type() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"map { 1: 100, 2: 200 } instance of map(xs:integer, xs:string)"#)
            .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// An empty map matches any typed map test.
#[test]
fn instance_of_typed_map_empty() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"map {} instance of map(xs:integer, xs:string)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `map(*)` still matches any map regardless of key/value types.
#[test]
fn instance_of_any_map_test() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"map { "a": 1, "b": 2 } instance of map(*)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A non-map item is not an instance of a typed map test.
#[test]
fn instance_of_typed_map_non_map() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"[1, 2, 3] instance of map(xs:integer, xs:integer)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// Typed array tests: array(T)
// ============================================================

/// An array of integers matches `array(xs:integer)`.
#[test]
fn instance_of_typed_array_matching() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("[1, 2, 3] instance of array(xs:integer)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// An array of strings does not match `array(xs:integer)`.
#[test]
fn instance_of_typed_array_wrong_member_type() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"["a", "b", "c"] instance of array(xs:integer)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// An empty array matches any typed array test.
#[test]
fn instance_of_typed_array_empty() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("[] instance of array(xs:string)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `array(*)` still matches any array regardless of member types.
#[test]
fn instance_of_any_array_test() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"["a", "b"] instance of array(*)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A non-array item is not an instance of a typed array test.
#[test]
fn instance_of_typed_array_non_array() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"map { 1: 2 } instance of array(xs:integer)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// Typed function tests: function(T1, T2, ...) as R
// ============================================================

/// A named function reference with matching arity matches a typed function test.
#[test]
fn instance_of_typed_function_matching_arity() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        "fn:abs#1 instance of function(xs:integer) as xs:integer",
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A named function reference with wrong arity does not match.
#[test]
fn instance_of_typed_function_wrong_arity() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        "fn:abs#1 instance of function(xs:integer, xs:integer) as xs:integer",
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// An inline function with matching arity matches a typed function test.
#[test]
fn instance_of_typed_function_inline() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        "function($x) { $x + 1 } instance of function(xs:integer) as xs:integer",
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A zero-arity inline function matches `function() as xs:integer`.
#[test]
fn instance_of_typed_function_zero_arity() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        "function() { 42 } instance of function() as xs:integer",
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A map (arity 1) matches `function(xs:integer) as xs:string`.
#[test]
fn instance_of_typed_function_map_as_function() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"map { 1: "a" } instance of function(xs:integer) as xs:string"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A map (arity 1) does not match a typed function test with arity 2.
#[test]
fn instance_of_typed_function_map_wrong_arity() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"map { 1: "a" } instance of function(xs:integer, xs:integer) as xs:string"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// `function(*)` still matches any function regardless of arity.
#[test]
fn instance_of_any_function_test() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        "fn:abs#1 instance of function(*)",
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A non-function is not an instance of a typed function test.
#[test]
fn instance_of_typed_function_non_function() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        "42 instance of function(xs:integer) as xs:integer",
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// Unknown / unrecognized type names: err:XPST0051
// ============================================================

/// An unknown type name in `instance of` should raise err:XPST0051.
#[test]
fn instance_of_unknown_type_raises_xpst0051() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of xs:foobar").unwrap();

    let err = xpath.apply(&document).unwrap_err();
    assert!(
        err.to_string().contains("XPST0051"),
        "expected XPST0051, got: {}",
        err
    );
}

/// An unknown unprefixed type name should also raise err:XPST0051.
#[test]
fn instance_of_unknown_unprefixed_type_raises_xpst0051() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of foobar").unwrap();

    let err = xpath.apply(&document).unwrap_err();
    assert!(
        err.to_string().contains("XPST0051"),
        "expected XPST0051, got: {}",
        err
    );
}

// ============================================================
// xs:anyAtomicType — matches any atomic value
// ============================================================

/// An integer is an instance of xs:anyAtomicType.
#[test]
fn instance_of_any_atomic_type_integer() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of xs:anyAtomicType").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A string is an instance of xs:anyAtomicType.
#[test]
fn instance_of_any_atomic_type_string() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" instance of xs:anyAtomicType"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A node is NOT an instance of xs:anyAtomicType.
#[test]
fn instance_of_any_atomic_type_node_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("/html instance of xs:anyAtomicType").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// xs:decimal — xs:integer is a subtype
// ============================================================

/// An integer is an instance of xs:decimal (subtype relationship).
#[test]
fn instance_of_decimal_integer() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of xs:decimal").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A string is not an instance of xs:decimal.
#[test]
fn instance_of_decimal_string_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" instance of xs:decimal"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// xs:numeric — union of xs:double, xs:float, xs:decimal
// ============================================================

/// An integer is an instance of xs:numeric.
#[test]
fn instance_of_numeric_integer() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of xs:numeric").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A double is an instance of xs:numeric.
#[test]
fn instance_of_numeric_double() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("3.14e0 instance of xs:numeric").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// A string is not an instance of xs:numeric.
#[test]
fn instance_of_numeric_string_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" instance of xs:numeric"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// Recognized but unimplemented types — return false
// ============================================================

/// An integer is not an instance of xs:date (recognized but unimplemented).
#[test]
fn instance_of_unimplemented_type_returns_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 instance of xs:date").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// A string is not an instance of xs:untypedAtomic (recognized but unimplemented).
#[test]
fn instance_of_untyped_atomic_returns_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" instance of xs:untypedAtomic"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}
