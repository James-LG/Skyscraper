use skyscraper::{html, xpath};

/// `//div | //span` should return all div and span elements.
#[test]
fn union_bar_combines_results() {
    let text = r#"<html><body><div>a</div><span>b</span><p>c</p></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div | //span").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return div and span: {items:?}");
}

/// `//div union //span` should work like `|`.
#[test]
fn union_keyword_combines_results() {
    let text = r#"<html><body><div>a</div><span>b</span><p>c</p></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div union //span").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return div and span: {items:?}");
}

/// Union of overlapping sets removes duplicates.
#[test]
fn union_deduplicates() {
    let text = r#"<html><body><div class="a">x</div><div class="b">y</div></body></html>"#;

    let document = html::parse(text).unwrap();
    // Both sides select the same divs — result should still be 2, not 4.
    let xpath = xpath::parse("//div | //div").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should deduplicate: {items:?}");
}

/// Union of three sets: `//div | //span | //p`.
#[test]
fn union_three_sets() {
    let text = r#"<html><body><div>a</div><span>b</span><p>c</p></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div | //span | //p").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3, "should return all three elements: {items:?}");
}

/// Union results should be in document order.
#[test]
fn union_document_order() {
    let text = r#"<html><body><span>first</span><div>second</div></body></html>"#;

    let document = html::parse(text).unwrap();
    // Select div first, then span — result should still be span before div (document order).
    let xpath = xpath::parse("//div | //span").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    let first = items[0].extract_as_node().extract_as_element_node();
    assert_eq!(first.name, "span", "span should come first in document order");
}

/// `//div intersect //div[@class]` keeps only divs that have a class attribute.
#[test]
fn intersect_filters_common() {
    let text =
        r#"<html><body><div class="a">x</div><div>y</div><div class="b">z</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div intersect //div[@class]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items.len(),
        2,
        "should return only divs with class attr: {items:?}"
    );
}

/// `//div except //div[@class]` removes divs that have a class attribute.
#[test]
fn except_removes_matching() {
    let text =
        r#"<html><body><div class="a">x</div><div>y</div><div class="b">z</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div except //div[@class]").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items.len(),
        1,
        "should return only the div without class: {items:?}"
    );
}

/// Except with disjoint sets returns the LHS unchanged.
#[test]
fn except_disjoint_returns_lhs() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div except //span").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return div: {items:?}");
}

/// Intersect with disjoint sets returns empty.
#[test]
fn intersect_disjoint_returns_empty() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div intersect //span").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should return empty: {items:?}");
}
