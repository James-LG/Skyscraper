use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// Partial application of `contains` with a fixed first argument.
/// `contains("hello world", ?)` creates a function, then calling it with "world".
#[test]
fn partial_contains_first_arg_fixed() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $f := contains("hello world", ?) return $f("world")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Partial application of `contains` with a fixed second argument.
#[test]
fn partial_contains_second_arg_fixed() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"let $f := contains(?, "world") return $f("hello world")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// Partial application produces a function item.
#[test]
fn partial_application_is_function() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"contains("hello", ?) instance of function(*)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}
