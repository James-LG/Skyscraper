use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// URI-qualified fn:contains should work like unprefixed contains.
#[test]
fn uri_qualified_contains() {
    let text = r#"<html><body><div>hello world</div><div>other</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"//div[Q{http://www.w3.org/2005/xpath-functions}contains(text(), "hello")]"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match 1 div: {items:?}");
}

/// fn:contains should work with fn: prefix (currently only unprefixed works).
#[test]
fn fn_prefixed_contains() {
    let text = r#"<html><body><div>hello world</div><div>other</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"//div[fn:contains(text(), "hello")]"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match 1 div: {items:?}");
}

/// URI-qualified fn:data should atomize the context item.
#[test]
fn uri_qualified_data() {
    let text = r#"<html><body><div>42</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"Q{http://www.w3.org/2005/xpath-functions}data(//div/text())"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("42".to_string()))
    );
}

/// fn:data with fn: prefix.
#[test]
fn fn_prefixed_data() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"fn:data(//div/text())"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("hello".to_string()))
    );
}

/// fn:string with fn: prefix.
#[test]
fn fn_prefixed_string() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"fn:string(//div)"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("hello".to_string()))
    );
}

/// URI-qualified fn:string.
#[test]
fn uri_qualified_string() {
    let text = r#"<html><body><div>world</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"Q{http://www.w3.org/2005/xpath-functions}string(//div)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("world".to_string()))
    );
}

/// Unknown URI namespace should error.
#[test]
fn unknown_uri_namespace_errors() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"Q{http://example.com/unknown}foo()"#,
    )
    .unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "unknown namespace should error: {result:?}");
}
