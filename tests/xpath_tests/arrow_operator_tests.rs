use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// `"hello world" => contains("world")` is equivalent to `contains("hello world", "world")`.
#[test]
fn arrow_contains_true() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello world" => contains("world")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `"hello world" => contains("xyz")` should return false.
#[test]
fn arrow_contains_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello world" => contains("xyz")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// Chaining multiple arrow operations: `"hello" => contains("ell") => contains("tru")`.
/// First arrow: `contains("hello", "ell")` → true.
/// Second arrow: `contains("true", "tru")` → true (boolean "true" stringified).
#[test]
fn arrow_chained() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" => contains("ell") => contains("tru")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Arrow with integer LHS: `42 => contains("4")`.
/// Integer 42 is stringified to "42", then `contains("42", "4")` → true.
#[test]
fn arrow_integer_stringified() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"42 => contains("4")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Arrow with no items should just return the base expression.
#[test]
fn arrow_no_items_returns_base() {
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

/// Arrow combined with `let`: `let $x := "world" return "hello world" => contains($x)`.
#[test]
fn arrow_with_let_variable() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $x := "world" return "hello world" => contains($x)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Arrow with VarRef function specifier: `let $f := fn:contains#2 return "hello" => $f("ell")`.
/// The variable `$f` holds a named function reference, used as the arrow target.
#[test]
fn arrow_varref_function_specifier() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $f := fn:contains#2 return "hello" => $f("ell")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Arrow with VarRef specifier returning false.
#[test]
fn arrow_varref_function_specifier_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"let $f := fn:contains#2 return "hello" => $f("xyz")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// Arrow with ParenthesizedExpr function specifier:
/// `"hello" => (fn:contains#2)("ell")`.
#[test]
fn arrow_parenthesized_function_specifier() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#""hello" => (fn:contains#2)("ell")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Arrow with inline function via ParenthesizedExpr:
/// `"hello" => (function($s, $sub) { contains($s, $sub) })("ell")`.
#[test]
fn arrow_parenthesized_inline_function_specifier() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#""hello" => (function($s, $sub) { contains($s, $sub) })("ell")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Arrow with `if`: `if (1) then "hello" => contains("ell") else 0`.
#[test]
fn arrow_with_if() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"if (1) then "hello" => contains("ell") else 0"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}
