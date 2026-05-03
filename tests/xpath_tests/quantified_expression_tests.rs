use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// `some $x in (1, 2, 3) satisfies $x = 2` should return true.
#[test]
fn some_with_matching_item() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("some $x in (1, 2, 3) satisfies $x = 2").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `some $x in (1, 2, 3) satisfies $x = 5` should return false.
#[test]
fn some_with_no_matching_item() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("some $x in (1, 2, 3) satisfies $x = 5").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// `every $x in (2, 4, 6) satisfies $x > 1` should return true.
#[test]
fn every_all_satisfy() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("every $x in (2, 4, 6) satisfies $x > 1").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `every $x in (2, 4, 6) satisfies $x > 3` should return false (2 fails).
#[test]
fn every_not_all_satisfy() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("every $x in (2, 4, 6) satisfies $x > 3").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// `some` over an empty sequence should return false.
#[test]
fn some_empty_sequence() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("some $x in //nonexistent satisfies $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

/// `every` over an empty sequence should return true (vacuous truth).
#[test]
fn every_empty_sequence() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("every $x in //nonexistent satisfies $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `some` with node sequences: some li has text "b".
#[test]
fn some_with_nodes() {
    let text = r#"<html><body><ul><li>a</li><li>b</li><li>c</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"some $x in //li satisfies $x/text() = "b""#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `every` with node sequences: every li has a text node.
#[test]
fn every_with_nodes() {
    let text = r#"<html><body><ul><li>a</li><li>b</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("every $x in //li satisfies $x/text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `some` with multiple bindings.
/// `some $x in (1, 2), $y in (3, 4) satisfies $x + $y = 6` should be true (2+4=6).
#[test]
fn some_multiple_bindings() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("some $x in (1, 2), $y in (3, 4) satisfies $x + $y = 6").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

/// `every` with multiple bindings.
/// `every $x in (1, 2), $y in (3, 4) satisfies $x + $y > 2` should be true.
#[test]
fn every_multiple_bindings() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("every $x in (1, 2), $y in (3, 4) satisfies $x + $y > 2").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}
