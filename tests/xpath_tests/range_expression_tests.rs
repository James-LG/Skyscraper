use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// `1 to 5` should return the sequence (1, 2, 3, 4, 5).
#[test]
fn range_basic() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("1 to 5").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 5);
    for (i, item) in items.iter().enumerate() {
        assert_eq!(
            *item,
            XpathItem::AnyAtomicType(AnyAtomicType::Integer(i as i64 + 1))
        );
    }
}

/// `3 to 3` should return a single-element sequence (3).
#[test]
fn range_single_element() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("3 to 3").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(3))
    );
}

/// `5 to 1` should return an empty sequence (start > end).
#[test]
fn range_empty_when_start_greater() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("5 to 1").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

/// Range with arithmetic: `(2 + 1) to (3 * 2)` = `3 to 6`.
#[test]
fn range_with_arithmetic() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("(2 + 1) to (3 * 2)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 4);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(3))
    );
    assert_eq!(
        items[3],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(6))
    );
}

/// Range with negative numbers: `-2 to 2`.
#[test]
fn range_negative_to_positive() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("-2 to 2").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 5);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(-2))
    );
    assert_eq!(
        items[4],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
}

/// No `to` clause — just return the base expression.
#[test]
fn range_no_to_returns_base() {
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

/// Range used with `for`: `for $i in 1 to 3 return $i * 2`.
#[test]
fn range_with_for() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $i in 1 to 3 return $i * 2").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
    assert_eq!(
        items[1],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(4))
    );
    assert_eq!(
        items[2],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(6))
    );
}
