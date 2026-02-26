use skyscraper::{html, xpath};

/// Basic for expression iterating over child elements.
/// `for $x in //li return $x` should return all matching elements.
#[test]
fn for_expr_identity() {
    let text = r#"<html><body><ul><li>a</li><li>b</li><li>c</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $x in //li return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3, "for $x in //li return $x should return 3 items: {items:?}");
}

/// For expression with a return expression that navigates from the bound variable.
/// `for $x in //li return $x/text()` should return text nodes of each li.
#[test]
fn for_expr_return_text() {
    let text = r#"<html><body><ul><li>alpha</li><li>beta</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $x in //li return $x/text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 text nodes: {items:?}");

    let texts: Vec<String> = items
        .iter()
        .map(|item| {
            item.extract_as_node()
                .extract_as_text_node()
                .content
                .to_string()
        })
        .collect();
    assert_eq!(texts, vec!["alpha", "beta"]);
}

/// For expression with literal sequence: `for $x in (1, 2, 3) return $x`.
#[test]
fn for_expr_literal_sequence() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $x in (1, 2, 3) return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3, "for over (1,2,3) should return 3 items: {items:?}");
}

/// For expression with multiple bindings (cartesian product).
/// `for $x in (1, 2), $y in (3, 4) return $x + $y`
/// Should produce: 1+3=4, 1+4=5, 2+3=5, 2+4=6
#[test]
fn for_expr_multiple_bindings() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $x in (1, 2), $y in (3, 4) return $x + $y").unwrap();

    let items = xpath.apply(&document).unwrap();
    // 4 combinations: (1,3), (1,4), (2,3), (2,4) → results 4, 5, 5, 6
    // Note: XpathItemSet is an IndexSet, so duplicates are removed.
    // 4, 5, 6 = 3 unique values
    assert_eq!(
        items.len(),
        3,
        "for $x in (1,2), $y in (3,4) return $x+$y should return 3 unique items: {items:?}"
    );
}

/// For expression where the inner binding references the outer variable.
/// `for $x in //ul, $y in $x/li return $y` should return all li elements.
#[test]
fn for_expr_dependent_bindings() {
    let text = r#"<html><body>
        <ul><li>a</li><li>b</li></ul>
        <ul><li>c</li></ul>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $x in //ul, $y in $x/li return $y").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items.len(),
        3,
        "should return all 3 li elements from both ul: {items:?}"
    );
}

/// For expression returning a computed string.
/// `for $x in //div return $x/@class` should return class attribute values.
#[test]
fn for_expr_return_attribute() {
    let text =
        r#"<html><body><div class="a">1</div><div class="b">2</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $x in //div return $x/@class").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 attribute nodes: {items:?}");
}

/// Empty sequence should produce empty result.
/// `for $x in //nonexistent return $x` should return nothing.
#[test]
fn for_expr_empty_sequence() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("for $x in //nonexistent return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "empty sequence should produce empty result: {items:?}");
}

/// Undefined variable should produce an error.
#[test]
fn var_ref_undefined_should_error() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("$undefined").unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "referencing undefined variable should error");
}
