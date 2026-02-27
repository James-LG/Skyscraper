use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

// ============================================================
// Inline function expressions
// ============================================================

/// Basic inline function: identity function.
/// `let $f := function($x) { $x } return $f(42)` should return 42.
#[test]
fn inline_function_identity() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $f := function($x) { $x } return $f(42)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(42))
    );
}

/// Inline function with arithmetic in the body.
/// `let $double := function($x) { $x + $x } return $double(21)` should return 42.
#[test]
fn inline_function_arithmetic() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("let $double := function($x) { $x + $x } return $double(21)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(42))
    );
}

/// Inline function with multiple parameters.
/// `let $add := function($a, $b) { $a + $b } return $add(10, 32)` should return 42.
#[test]
fn inline_function_multiple_params() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("let $add := function($a, $b) { $a + $b } return $add(10, 32)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(42))
    );
}

/// Inline function with no parameters.
/// `let $f := function() { 99 } return $f()` should return 99.
#[test]
fn inline_function_no_params() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $f := function() { 99 } return $f()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(99))
    );
}

/// Inline function with string concatenation.
/// `let $greet := function($name) { "hello " || $name } return $greet("world")` should return "hello world".
#[test]
fn inline_function_string_concat() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $greet := function($name) { "hello " || $name } return $greet("world")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("hello world".to_string()))
    );
}

/// Wrong number of arguments to inline function should error.
#[test]
fn inline_function_wrong_arity_errors() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("let $f := function($x) { $x } return $f(1, 2)").unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "should error on wrong arity");
}

// ============================================================
// Named function references
// ============================================================

/// Named function reference for `contains`.
/// `let $f := contains#2 return $f("hello world", "world")` should return true.
#[test]
fn named_function_ref_contains() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $f := contains#2 return $f("hello world", "world")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Named function reference with wrong arity should error.
#[test]
fn named_function_ref_wrong_arity_errors() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $f := contains#2 return $f("hello")"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "should error on wrong arity");
}

// ============================================================
// Parsing / Display tests
// ============================================================

/// Named function reference should parse and display correctly.
#[test]
fn function_item_expr_display_named() {
    let xpath = xpath::parse("let $f := fn:abs#1 return $f").unwrap();
    assert_eq!(xpath.to_string(), "let $f := fn:abs#1 return $f");
}

/// Inline function should parse and display correctly.
#[test]
fn function_item_expr_display_inline() {
    let xpath = xpath::parse("let $f := function($x) { $x } return $f").unwrap();
    assert_eq!(
        xpath.to_string(),
        "let $f := function($x) { $x } return $f"
    );
}
