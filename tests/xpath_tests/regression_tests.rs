use skyscraper::xpath::grammar::data_model::AnyAtomicType;
use skyscraper::{html, xpath};

// ============================================================================
// Regression: #6 - boolean(NaN) should return false, not true.
// Per XPath 3.1 Section 2.4.3: "fn:boolean returns false if $arg is +0, -0,
// or NaN".
// ============================================================================

#[test]
fn boolean_nan_double_returns_false() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    // number("not-a-number") produces NaN; boolean(NaN) should be false.
    // An if-expression lets us test the boolean conversion.
    let xpath = xpath::parse("if (number('not-a-number')) then 'yes' else 'no'").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::String("no".to_string()),
        "boolean(NaN) should be false, so the else branch should be taken"
    );
}

#[test]
fn boolean_zero_returns_false() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (0) then 'yes' else 'no'").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::String("no".to_string()),
        "boolean(0) should be false"
    );
}

#[test]
fn boolean_positive_number_returns_true() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("if (42) then 'yes' else 'no'").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::String("yes".to_string()),
        "boolean(42) should be true"
    );
}

// ============================================================================
// Regression: #7 - NaN comparison semantics. OrderedFloat makes NaN == NaN
// true, but XPath/IEEE 754 requires NaN eq NaN to be false.
// ============================================================================

#[test]
fn nan_eq_nan_is_false() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    // NaN eq NaN should be false per XPath spec.
    let xpath = xpath::parse("number('x') eq number('x')").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::Boolean(false),
        "NaN eq NaN should be false"
    );
}

#[test]
fn nan_ne_nan_is_true() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    // NaN ne NaN should be true per XPath spec.
    let xpath = xpath::parse("number('x') ne number('x')").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::Boolean(true),
        "NaN ne NaN should be true"
    );
}

#[test]
fn nan_lt_number_is_false() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    // NaN lt 0 should be false.
    let xpath = xpath::parse("number('x') lt 0").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::Boolean(false),
        "NaN lt 0 should be false"
    );
}

#[test]
fn nan_gt_number_is_false() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    // NaN gt 0 should be false.
    let xpath = xpath::parse("number('x') gt 0").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::Boolean(false),
        "NaN gt 0 should be false"
    );
}

#[test]
fn nan_le_nan_is_false() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    // NaN le NaN should be false.
    let xpath = xpath::parse("number('x') le number('x')").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::Boolean(false),
        "NaN le NaN should be false"
    );
}

#[test]
fn nan_ge_nan_is_false() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();
    // NaN ge NaN should be false.
    let xpath = xpath::parse("number('x') ge number('x')").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    let value = result[0].extract_as_any_atomic_type();
    assert_eq!(
        *value,
        AnyAtomicType::Boolean(false),
        "NaN ge NaN should be false"
    );
}

#[test]
fn normal_value_comparisons_still_work() {
    let text = "<html><body><div>x</div></body></html>";
    let document = html::parse(text).unwrap();

    let cases = vec![
        ("1 eq 1", true),
        ("1 ne 2", true),
        ("1 lt 2", true),
        ("2 gt 1", true),
        ("1 le 1", true),
        ("1 ge 1", true),
        ("1 eq 2", false),
        ("1 ne 1", false),
    ];

    for (expr, expected) in cases {
        let xpath = xpath::parse(expr).unwrap();
        let result = xpath.apply(&document).unwrap();
        assert_eq!(result.len(), 1, "expr: {expr}");
        let value = result[0].extract_as_any_atomic_type();
        assert_eq!(
            *value,
            AnyAtomicType::Boolean(expected),
            "Expression '{expr}' should be {expected}"
        );
    }
}

// ============================================================================
// Regression: #13 - dedup should work correctly (was O(n^2), now O(n)).
// Verify it still correctly deduplicates.
// ============================================================================

#[test]
fn xpath_dedup_removes_duplicate_nodes() {
    let text = r#"<html><body>
        <div class="a">first</div>
        <div class="b">second</div>
    </body></html>"#;
    let document = html::parse(text).unwrap();
    // Union of the same node set with itself should not produce duplicates.
    let xpath = xpath::parse("//div | //div").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result.len(),
        2,
        "Union of identical sets should deduplicate: got {} items",
        result.len()
    );
}

#[test]
fn xpath_dedup_preserves_distinct_nodes() {
    let text = r#"<html><body>
        <div>one</div>
        <span>two</span>
        <p>three</p>
    </body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div | //span | //p").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result.len(),
        3,
        "Union of distinct nodes should preserve all: got {} items",
        result.len()
    );
}
