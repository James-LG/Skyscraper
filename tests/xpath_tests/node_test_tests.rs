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

// --- prefix:* wildcard tests ---

/// `svg:*` matches only SVG-namespace elements, not HTML elements.
#[test]
fn wildcard_svg_prefix_matches_svg_elements() {
    let text = r#"<html><body><svg><rect/><circle/></svg><div>not svg</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//svg:*").unwrap();

    let items = xpath.apply(&document).unwrap();
    // Should match: svg, rect, circle (all in SVG namespace)
    assert_eq!(items.len(), 3, "should match svg, rect, circle: {items:?}");
    for item in &items {
        let el = item.extract_as_node().extract_as_element_node();
        assert!(
            ["svg", "rect", "circle"].contains(&el.name.as_str()),
            "unexpected element: {}",
            el.name
        );
    }
}

/// `svg:*` does not match HTML elements.
#[test]
fn wildcard_svg_prefix_excludes_html_elements() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//svg:*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should not match any HTML elements: {items:?}");
}

/// `html:*` matches HTML-namespace elements.
#[test]
fn wildcard_html_prefix_matches_html_elements() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/html:*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match div and span: {items:?}");
}

/// `html:*` does not match SVG-namespace elements.
#[test]
fn wildcard_html_prefix_excludes_svg_elements() {
    let text = r#"<html><body><svg><rect/></svg></body></html>"#;

    let document = html::parse(text).unwrap();
    // Look only at children of svg; those are in SVG namespace.
    let xpath = xpath::parse("//svg/html:*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should not match SVG elements: {items:?}");
}

/// `mathml:*` matches MathML-namespace elements.
#[test]
fn wildcard_mathml_prefix_matches_mathml_elements() {
    let text = r#"<html><body><math><mi>x</mi><mo>+</mo><mn>1</mn></math></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//mathml:*").unwrap();

    let items = xpath.apply(&document).unwrap();
    // Should match: math, mi, mo, mn
    assert_eq!(items.len(), 4, "should match math, mi, mo, mn: {items:?}");
}

/// Unknown prefix raises XPST0081 error.
#[test]
fn wildcard_unknown_prefix_errors() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//foo:*").unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "unknown prefix should produce an error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("XPST0081"),
        "error should reference XPST0081: {err}"
    );
}

/// `Q{http://www.w3.org/2000/svg}*` matches SVG elements (braced URI wildcard).
#[test]
fn wildcard_braced_uri_svg_matches() {
    let text = r#"<html><body><svg><rect/></svg></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//Q{http://www.w3.org/2000/svg}*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match svg and rect: {items:?}");
}

/// `Q{http://www.w3.org/1999/xhtml}*` matches HTML elements (braced URI wildcard).
#[test]
fn wildcard_braced_uri_html_matches() {
    let text = r#"<html><body><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/Q{http://www.w3.org/1999/xhtml}*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match div: {items:?}");
}

/// `Q{http://www.w3.org/2000/svg}*` does not match HTML elements.
#[test]
fn wildcard_braced_uri_svg_excludes_html() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//Q{http://www.w3.org/2000/svg}*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should not match HTML elements: {items:?}");
}
