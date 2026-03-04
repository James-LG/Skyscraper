use skyscraper::xpath::grammar::data_model::{AnyAtomicType, XpathItem};
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

// ============================================================================
// Regression tests for general comparison existential quantification (Fix #4)
// ============================================================================

/// Regression: general comparisons (`=`, `!=`, `<`, etc.) must use existential
/// quantification over sequences. `(1, 2) = (2, 3)` should be true because
/// there exists a pair (2, 2) where `=` holds.
#[test]
fn general_comp_existential_quantification() {
    let text = "<html><body></body></html>";
    let document = html::parse(text).unwrap();

    // (1, 2) = (2, 3) → true because 2 = 2
    let xpath = xpath::parse("(1, 2) = (2, 3)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)),
        "(1,2) = (2,3) should be true via existential quantification"
    );
}

/// Regression: `(1, 2) = (3, 4)` should be false — no pair satisfies `=`.
#[test]
fn general_comp_existential_no_match() {
    let text = "<html><body></body></html>";
    let document = html::parse(text).unwrap();

    let xpath = xpath::parse("(1, 2) = (3, 4)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)),
        "(1,2) = (3,4) should be false"
    );
}

/// Regression: general comparison with node sequences should compare atomized
/// values pairwise. `//div/@class = "b"` should match if any div has class="b".
#[test]
fn general_comp_node_sequence_existential() {
    let text = r#"<html><body>
        <div class="a">1</div>
        <div class="b">2</div>
        <div class="c">3</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class = ('a', 'c')]").unwrap();
    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(
        nodes.len(),
        2,
        "General comparison with sequence should use existential quantification: {nodes:?}"
    );
}
