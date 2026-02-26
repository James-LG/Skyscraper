use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// `if (1) then 10 else 20` should return 10 (1 is truthy).
#[test]
fn if_truthy_returns_then() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (1) then 10 else 20").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(10))
    );
}

/// `if (0) then 10 else 20` should return 20 (0 is falsy).
#[test]
fn if_falsy_returns_else() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (0) then 10 else 20").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(20))
    );
}

/// Condition based on node existence: `if (//div) then 1 else 2`.
#[test]
fn if_node_exists_returns_then() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (//div) then 1 else 2").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}

/// Condition based on missing node: `if (//nonexistent) then 1 else 2`.
#[test]
fn if_node_missing_returns_else() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (//nonexistent) then 1 else 2").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
}

/// `if` returning node sequences: `if (1) then //li else //div`.
#[test]
fn if_returns_node_sequence() {
    let text = r#"<html><body><ul><li>a</li><li>b</li></ul><div>c</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (1) then //li else //div").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 li elements: {items:?}");
}

/// String EBV: `if ("") then 1 else 2` should return 2 (empty string is falsy).
#[test]
fn if_empty_string_is_falsy() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"if ("") then 1 else 2"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
}

/// String EBV: `if ("hello") then 1 else 2` should return 1 (non-empty string is truthy).
#[test]
fn if_nonempty_string_is_truthy() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"if ("hello") then 1 else 2"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}

/// Nested if: `if (1) then if (0) then 1 else 2 else 3` should return 2.
#[test]
fn if_nested() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (1) then if (0) then 1 else 2 else 3").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
}

/// Comparison in condition: `if (2 > 1) then 10 else 20`.
#[test]
fn if_with_comparison_condition() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (2 > 1) then 10 else 20").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(10))
    );
}

/// `if` combined with `let`: `let $x := 5 return if ($x > 3) then $x else 0`.
#[test]
fn if_with_let() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := 5 return if ($x > 3) then $x else 0").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(5))
    );
}
