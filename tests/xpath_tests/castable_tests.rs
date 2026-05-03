use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// `"123" castable as integer` should return true.
#[test]
fn castable_string_to_integer_true() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""123" castable as integer"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `"abc" castable as integer` should return false.
#[test]
fn castable_string_to_integer_false() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""abc" castable as integer"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// `42 castable as string` should return true (integers are always castable to string).
#[test]
fn castable_integer_to_string_true() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("42 castable as string").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Empty sequence with `?` is castable.
#[test]
fn castable_empty_with_question_mark() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//nonexistent castable as integer?").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `"42" castable as Q{http://www.w3.org/2001/XMLSchema}integer` should return true.
#[test]
fn castable_uri_qualified_type() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#""42" castable as Q{http://www.w3.org/2001/XMLSchema}integer"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Castable combined with if: `if ("42" castable as integer) then "42" cast as integer else 0`.
#[test]
fn castable_with_if_and_cast() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"if ("42" castable as integer) then "42" cast as integer else 0"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(42))
    );
}
