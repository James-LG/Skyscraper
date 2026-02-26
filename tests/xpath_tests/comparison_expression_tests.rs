use skyscraper::{html, xpath};

/// The `eq` value comparison should match equal atomic values
/// (XPath 3.1 section 3.5.1).
#[test]
fn value_comp_eq_matches_equal_strings() {
    let text = r#"<html><body>
        <div class="a">first</div>
        <div class="b">second</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class eq 'a']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "eq should match one div: {nodes:?}");
}

/// The `ne` value comparison should match non-equal atomic values
/// (XPath 3.1 section 3.5.1).
#[test]
fn value_comp_ne_matches_unequal_strings() {
    let text = r#"<html><body>
        <div class="a">first</div>
        <div class="b">second</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class ne 'a']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "ne should match the div with class='b': {nodes:?}");
}

/// The `is` node comparison should return true when both operands are the
/// same node (XPath 3.1 section 3.5.3).
#[test]
fn node_comp_is_matches_same_node() {
    let text = r#"<html><body>
        <div id="target">content</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    // A node is always identical to itself.
    let xpath = xpath::parse("//div[@id='target'][. is .]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "is should match the node with itself: {nodes:?}");
}
