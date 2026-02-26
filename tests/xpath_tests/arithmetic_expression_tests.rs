use skyscraper::{html, xpath};

/// Integer addition: 1 + 2 = 3 (XPath 3.1 section 3.4).
#[test]
fn additive_integer_plus() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[1 + 2 = 3]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "1 + 2 should equal 3: {nodes:?}");
}

/// Integer subtraction: 5 - 3 = 2 (XPath 3.1 section 3.4).
#[test]
fn additive_integer_minus() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[5 - 3 = 2]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "5 - 3 should equal 2: {nodes:?}");
}

/// Integer multiplication: 3 * 4 = 12 (XPath 3.1 section 3.4).
#[test]
fn multiplicative_integer_star() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[3 * 4 = 12]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "3 * 4 should equal 12: {nodes:?}");
}

/// The `div` operator always returns a double: 10 div 2 = 5e0 (XPath 3.1 section 3.4).
/// Note: `div` returns xs:double, so compare against a double literal (`5e0`).
#[test]
fn multiplicative_div() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[10 div 2 = 5e0]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "10 div 2 should equal 5e0: {nodes:?}");
}

/// The `idiv` operator truncates towards zero: 10 idiv 3 = 3 (XPath 3.1 section 3.4).
#[test]
fn multiplicative_idiv() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[10 idiv 3 = 3]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "10 idiv 3 should equal 3: {nodes:?}");
}

/// The `mod` operator returns the remainder: 10 mod 3 = 1 (XPath 3.1 section 3.4).
#[test]
fn multiplicative_mod() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[10 mod 3 = 1]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "10 mod 3 should equal 1: {nodes:?}");
}

/// Unary minus should negate a value (XPath 3.1 section 3.4).
#[test]
fn unary_minus_negation() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[-5 + 5 = 0]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "-5 + 5 should equal 0: {nodes:?}");
}

/// Double unary minus should cancel out (XPath 3.1 section 3.4).
#[test]
fn unary_double_minus_cancels() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[--5 = 5]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "--5 should equal 5: {nodes:?}");
}

/// Unary plus should be a no-op (XPath 3.1 section 3.4).
#[test]
fn unary_plus_noop() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[+5 = 5]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "+5 should equal 5: {nodes:?}");
}

/// Chained additive operations should evaluate left-to-right (XPath 3.1 section 3.4).
#[test]
fn additive_chained_operations() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[1 + 2 + 3 = 6]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "1 + 2 + 3 should equal 6: {nodes:?}");
}

/// Multiplication has higher precedence than addition (XPath 3.1 section 3.4).
#[test]
fn arithmetic_precedence() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    // 2 + 3 * 4 = 2 + 12 = 14 (not 20)
    let xpath = xpath::parse("//div[2 + 3 * 4 = 14]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(
        nodes.len(),
        1,
        "2 + 3 * 4 should equal 14 (not 20): {nodes:?}"
    );
}

/// A false arithmetic predicate should filter out nodes.
#[test]
fn arithmetic_false_predicate_filters() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[1 + 1 = 3]").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(
        nodes.len(),
        0,
        "1 + 1 != 3, so no nodes should match: {nodes:?}"
    );
}
