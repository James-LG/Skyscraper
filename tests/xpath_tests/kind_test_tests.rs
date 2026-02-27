use skyscraper::{html, xpath};

/// `element()` with no arguments matches any element.
#[test]
fn element_test_no_args_matches_all_elements() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match div and span: {items:?}");
}

/// `element(div)` matches only div elements.
#[test]
fn element_test_named_matches_specific_element() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element(div)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match only div: {items:?}");
}

/// `element(*)` matches any element (same as no args).
#[test]
fn element_test_wildcard_matches_all() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element(*)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match div and span: {items:?}");
}

/// `element(nonexistent)` matches nothing.
#[test]
fn element_test_named_no_match() {
    let text = r#"<html><body><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element(section)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should match nothing: {items:?}");
}

/// `comment()` matches comment nodes.
#[test]
fn comment_test_matches_comments() {
    let text = r#"<html><body><!-- hello --><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/comment()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match 1 comment: {items:?}");
}

/// `namespace-node()` returns empty in HTML context.
#[test]
fn namespace_node_test_returns_empty() {
    let text = r#"<html><body><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div/namespace-node()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "HTML has no namespace nodes: {items:?}");
}

/// `attribute(id)` matches only the id attribute.
#[test]
fn attribute_test_named_matches_specific() {
    let text = r#"<html><body><div id="foo" class="bar">a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div/attribute(id)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match only id attribute: {items:?}");
}

/// `attribute(*)` matches all attributes.
#[test]
fn attribute_test_wildcard_matches_all() {
    let text = r#"<html><body><div id="foo" class="bar">a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div/attribute(*)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match both attributes: {items:?}");
}

/// `processing-instruction()` display formatting.
#[test]
fn pi_test_display() {
    let xpath = xpath::parse("//processing-instruction()").unwrap();
    assert_eq!(xpath.to_string(), "//processing-instruction()");
}

/// `processing-instruction(name)` display formatting.
#[test]
fn pi_test_named_display() {
    let xpath = xpath::parse("//processing-instruction(xml)").unwrap();
    assert_eq!(xpath.to_string(), "//processing-instruction(xml)");
}

/// `schema-attribute(name)` display formatting.
#[test]
fn schema_attribute_test_display() {
    let xpath = xpath::parse("//schema-attribute(price)").unwrap();
    assert_eq!(xpath.to_string(), "//schema-attribute(price)");
}
