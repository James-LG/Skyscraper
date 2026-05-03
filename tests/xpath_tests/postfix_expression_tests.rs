use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// Positional predicate on parenthesized sequence: `(1 to 5)[3]` returns 3.
#[test]
fn postfix_positional_predicate() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(1 to 5)[3]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(3))
    );
}

/// First element predicate: `(//li)[1]`.
#[test]
fn postfix_first_element() {
    let text = r#"<html><body><ul><li>a</li><li>b</li><li>c</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(//li)[1]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
}

/// Last element predicate: `(//li)[3]`.
#[test]
fn postfix_last_element() {
    let text = r#"<html><body><ul><li>a</li><li>b</li><li>c</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(//li)[3]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
}

/// Boolean predicate on sequence: `(1 to 5)[. > 3]` returns 4, 5.
#[test]
fn postfix_boolean_predicate() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(1 to 5)[. > 3]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(4))
    );
    assert_eq!(
        items[1],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(5))
    );
}

/// Predicate out of range returns empty: `(1 to 3)[10]`.
#[test]
fn postfix_out_of_range_empty() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(1 to 3)[10]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

/// Chained predicates: `(1 to 10)[. > 3][. < 8]` returns 4, 5, 6, 7.
#[test]
fn postfix_chained_predicates() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(1 to 10)[. > 3][. < 8]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 4);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(4))
    );
    assert_eq!(
        items[3],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(7))
    );
}
