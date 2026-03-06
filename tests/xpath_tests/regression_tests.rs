use skyscraper::xpath::grammar::data_model::{AnyAtomicType, XpathItem};
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

// ============================================================================
// Regression: CR-3 - dedup should use node identity (NodeId), not structural
// equality. Two text nodes with the same content at different positions are
// distinct nodes and should not be collapsed.
// ============================================================================

#[test]
fn xpath_dedup_preserves_same_content_different_nodes() {
    // Two <span> elements both containing "x" are distinct nodes.
    // dedup should NOT collapse them since they have different NodeIds.
    let text = r#"<html><body>
        <div><span>x</span></div>
        <div><span>x</span></div>
    </body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//span").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result.len(),
        2,
        "Two distinct span nodes with same content should both be preserved: got {} items",
        result.len()
    );
}

#[test]
fn xpath_dedup_identity_based_for_text_nodes() {
    // Multiple text nodes with identical content "hello" at different
    // positions should all be preserved after dedup.
    let text = "<html><body><p>hello</p><p>hello</p><p>hello</p></body></html>";
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//p/text()").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result.len(),
        3,
        "Three distinct text nodes with same content should all be preserved: got {} items",
        result.len()
    );
}

// ============================================================================
// Regression: CR-6 - Lazy-init step_expr expansions. Path expression
// expansions using `/` and `//` should still produce correct results.
// ============================================================================

#[test]
fn leading_slash_expansion_with_lazy_statics() {
    let text = "<html><body><div><p>found</p></div></body></html>";
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("/html/body/div/p").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1, "Leading slash path should find 1 element");
}

#[test]
fn leading_double_slash_expansion_with_lazy_statics() {
    let text = "<html><body><div><p>a</p><p>b</p></div></body></html>";
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//p").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result.len(),
        2,
        "Leading double-slash path should find 2 elements"
    );
}

#[test]
fn mid_path_double_slash_with_lazy_statics() {
    let text = r#"<html><body>
        <div><span><a>deep</a></span></div>
        <div><a>shallow</a></div>
    </body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("/html//a").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result.len(),
        2,
        "Mid-path double-slash should find all descendant <a> elements"
    );
}

#[test]
fn bare_slash_returns_document_node() {
    let text = "<html><body></body></html>";
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("/").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result.len(),
        1,
        "Bare '/' should return the document node"
    );
}

// ============================================================================
// Regression: CR-7 - fn:substring NaN/infinity handling.
// Per XPath spec, NaN arguments should produce empty string.
// ============================================================================

#[test]
fn fn_substring_nan_start_returns_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // number("x") produces NaN; substring with NaN start should return "".
    let xpath = xpath::parse(r#"substring("hello", number("x"))"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new())),
        "substring with NaN start should return empty string"
    );
}

#[test]
fn fn_substring_nan_length_returns_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // NaN length should return empty string.
    let xpath = xpath::parse(r#"substring("hello", 1, number("x"))"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new())),
        "substring with NaN length should return empty string"
    );
}

#[test]
fn fn_substring_normal_cases_still_work() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let cases = vec![
        (r#"substring("hello", 2, 3)"#, "ell"),
        (r#"substring("hello", 2)"#, "ello"),
        (r#"substring("12345", 0, 3)"#, "12"),
        (r#"substring("12345", -1, 5)"#, "123"),
        (r#"substring("hello", 1, 5)"#, "hello"),
    ];
    for (expr, expected) in cases {
        let xpath = xpath::parse(expr).unwrap();
        let items = xpath.apply(&document).unwrap();
        assert_eq!(
            items[0],
            XpathItem::AnyAtomicType(AnyAtomicType::String(String::from(expected))),
            "Expression '{expr}' should return \"{expected}\""
        );
    }
}

// ============================================================================
// Regression: CR-10 - fn:apply must unpack array members as individual
// arguments, not treat the sequence items as arguments.
// ============================================================================

#[test]
fn fn_apply_unpacks_array_members() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"apply(fn:concat#2, ["hello ", "world"])"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello world"))),
        "fn:apply should unpack array members as function arguments"
    );
}

#[test]
fn fn_apply_non_array_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"apply(fn:concat#2, ("a", "b"))"#).unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "fn:apply with non-array second argument should error"
    );
}

// ============================================================================
// Regression: CR-11 - fn:format-number must not panic when the picture
// argument is an empty sequence.
// ============================================================================

#[test]
fn fn_format_number_empty_picture_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // Use a subexpression that produces an empty sequence for the picture arg
    let xpath = xpath::parse(r#"format-number(123, //nonexistent)"#).unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "fn:format-number with empty picture should error, not panic"
    );
}

// ============================================================================
// Regression: CR-12 - idiv with NaN or Infinity operands must raise FOAR0002,
// not silently produce wrong integer results.
// ============================================================================

#[test]
fn idiv_nan_raises_error() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("number('NaN') idiv 1").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "NaN idiv 1 should raise FOAR0002 error"
    );
}

#[test]
fn idiv_infinity_raises_error() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("1.0e308 * 10 idiv 1").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "Infinity idiv 1 should raise FOAR0002 error"
    );
}

#[test]
fn idiv_normal_still_works() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("10 idiv 3").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)),
        "10 idiv 3 should return 3"
    );
}

// ============================================================================
// Regression: CR-13 - fn:node-name must return xs:QName, not xs:string.
// ============================================================================

#[test]
fn fn_node_name_returns_qname() {
    let document = html::parse("<html><body><div>test</div></body></html>").unwrap();
    let xpath = xpath::parse("node-name(//div)").unwrap();
    let items = xpath.apply(&document).unwrap();
    match &items[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::QName { local_name, .. }) => {
            assert_eq!(local_name, "div", "local name should be 'div'");
        }
        other => panic!(
            "fn:node-name should return QName, got: {:?}",
            other
        ),
    }
}

#[test]
fn fn_node_name_attribute_returns_qname() {
    let document =
        html::parse(r#"<html><body><div class="test">x</div></body></html>"#).unwrap();
    let xpath = xpath::parse("node-name(//div/@class)").unwrap();
    let items = xpath.apply(&document).unwrap();
    match &items[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::QName { local_name, .. }) => {
            assert_eq!(local_name, "class", "local name should be 'class'");
        }
        other => panic!(
            "fn:node-name on attribute should return QName, got: {:?}",
            other
        ),
    }
}

// ============================================================================
// Regression: CR-14 - fn:round must support the 2-argument form
// fn:round($arg, $precision).
// ============================================================================

#[test]
fn fn_round_two_args() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("round(3.456e0, 2)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(ordered_float::OrderedFloat(3.46))),
        "round(3.456, 2) should return 3.46"
    );
}

#[test]
fn fn_round_two_args_negative_precision() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("round(1234e0, -2)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(ordered_float::OrderedFloat(1200.0))),
        "round(1234, -2) should return 1200"
    );
}

// ============================================================================
// Regression: CR-15 - fn:tokenize 2-arg form must NOT filter empty strings.
// ============================================================================

#[test]
fn fn_tokenize_2arg_preserves_empty_strings() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"tokenize(",a,,b,", ",")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    // Expected: ("", "a", "", "b", "")
    assert_eq!(
        items.len(),
        5,
        "tokenize(',a,,b,', ',') should return 5 items including empty strings, got {}",
        items.len()
    );
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new())),
        "First item should be empty string"
    );
    assert_eq!(
        items[2],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new())),
        "Third item should be empty string"
    );
    assert_eq!(
        items[4],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new())),
        "Fifth item should be empty string"
    );
}

// ============================================================================
// Regression: CR-16 - fn:codepoints-to-string must reject negative integers
// instead of wrapping them via i64 -> u32 cast.
// ============================================================================

#[test]
fn fn_codepoints_to_string_negative_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("codepoints-to-string((-1))").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "codepoints-to-string(-1) should error, not wrap to valid char"
    );
}

#[test]
fn fn_codepoints_to_string_valid_still_works() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("codepoints-to-string((65, 66, 67))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("ABC"))),
        "codepoints-to-string(65, 66, 67) should return 'ABC'"
    );
}
