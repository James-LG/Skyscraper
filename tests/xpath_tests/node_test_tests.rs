use skyscraper::{html, xpath};

/// Unprefixed name test matches elements (existing behavior, regression test).
#[test]
fn name_test_unprefixed() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match 1 div: {items:?}");
}

/// Wildcard `*` matches all element children.
#[test]
fn wildcard_simple_matches_elements() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match div and span: {items:?}");
}

/// Wildcard `*:local` matches elements with the given local name regardless of namespace.
#[test]
fn wildcard_prefixed_name_matches_local() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/*:div").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match only div: {items:?}");
}

/// Wildcard `*:local` does not match different local names.
#[test]
fn wildcard_prefixed_name_no_match() {
    let text = r#"<html><body><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/*:span").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should match nothing: {items:?}");
}

/// Prefixed name test `html:div` matches by local name in HTML context.
#[test]
fn name_test_prefixed() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//html:div").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match 1 div: {items:?}");
}

/// Name test on a non-node context item should not panic.
/// This uses a for expression to iterate over atomic values.
#[test]
fn name_test_non_node_no_panic() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    // (1, 2, 3) are atomic values; applying a name test step should just return empty.
    let xpath = xpath::parse("(1, 2, 3)/div").unwrap();

    let result = xpath.apply(&document);
    // This might error for other reasons, but should not panic with a todo!().
    // The important thing is that we don't hit the todo!() unreachable code.
    match result {
        Ok(items) => assert_eq!(items.len(), 0, "atomic values have no children: {items:?}"),
        Err(_) => {} // Acceptable — the expression may error for another reason.
    }
}
