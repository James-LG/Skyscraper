use skyscraper::xpath::grammar::data_model::{AnyAtomicType, XpathItem};
use skyscraper::xpath::grammar::XpathItemTreeNode;
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

// ============================================================================
// Regression: fn:substring must not panic when start > end (negative length
// or positive-infinity start). Previously chars[start..end] panicked.
// ============================================================================

#[test]
fn substring_negative_length_returns_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("substring('hello', 10, -5)").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new())),
        "substring with start beyond string and negative length should return empty"
    );
}

#[test]
fn substring_large_start_negative_length() {
    // Another case where start > end after clamping.
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("substring('hello', 100, -50)").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new())),
        "substring with large start and negative length should return empty"
    );
}

// ============================================================================
// Regression: fn:apply must not panic on empty sequence second argument.
// ============================================================================

#[test]
fn fn_apply_empty_second_arg_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // The second argument to fn:apply should be an array; an empty sequence should error.
    let xpath = xpath::parse("apply(boolean#1, ())").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "fn:apply with empty second argument should return an error, not panic"
    );
}

// ============================================================================
// Regression: fn:QName must not panic on empty second argument.
// ============================================================================

#[test]
fn fn_qname_empty_second_arg_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("QName('http://example.com', ())").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "fn:QName with empty lexical QName should return an error, not panic"
    );
}

// ============================================================================
// Regression: fn:sum should preserve integer precision for large values.
// Previously all integers went through f64, losing precision above 2^53.
// ============================================================================

#[test]
fn fn_sum_preserves_integer_type() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("sum((1, 2, 3))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(6)),
        "sum of integers should return an integer, not a double"
    );
}

// ============================================================================
// Regression: fn:min/fn:max with mixed numeric and string types should error,
// not silently discard one type.
// ============================================================================

#[test]
fn fn_min_mixed_types_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("min((1, 'hello'))").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "fn:min with mixed numeric and string values should error (FORG0006)"
    );
}

#[test]
fn fn_max_mixed_types_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("max((1, 'hello'))").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "fn:max with mixed numeric and string values should error (FORG0006)"
    );
}

#[test]
fn fn_min_all_strings_works() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("min(('banana', 'apple', 'cherry'))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("apple".to_string())),
        "fn:min on all-string sequence should return the lexicographic minimum"
    );
}

#[test]
fn fn_max_all_integers_preserves_type() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("max((3, 7, 2))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(7)),
        "fn:max on all-integer sequence should return an integer"
    );
}

// ============================================================================
// Regression: CR-2 - fn:replace must convert XPath backreference syntax (\1)
// to regex crate syntax ($1).
// ============================================================================

#[test]
fn fn_replace_backreference_syntax() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"replace("abcd", "(ab)", "\1X")"#).unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("abXcd".to_string())),
        "fn:replace should convert XPath \\1 backreferences to work correctly"
    );
}

#[test]
fn fn_replace_literal_dollar_sign() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"replace("abc", "b", "$")"#).unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("a$c".to_string())),
        "Literal $ in replacement string should not be interpreted as backreference"
    );
}

// ============================================================================
// Regression: CR-4 - fn:min/fn:max must return NaN when any value is NaN.
// ============================================================================

#[test]
fn fn_min_with_nan_returns_nan() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("min((1.0, number('NaN'), 3.0))").unwrap();
    let result = xpath.apply(&document).unwrap();
    match &result[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => {
            assert!(d.0.is_nan(), "fn:min with NaN in sequence should return NaN");
        }
        other => panic!("Expected Double(NaN), got: {:?}", other),
    }
}

#[test]
fn fn_max_with_nan_returns_nan() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("max((1.0, number('NaN'), 3.0))").unwrap();
    let result = xpath.apply(&document).unwrap();
    match &result[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => {
            assert!(d.0.is_nan(), "fn:max with NaN in sequence should return NaN");
        }
        other => panic!("Expected Double(NaN), got: {:?}", other),
    }
}

// ============================================================================
// Regression: CR-6 - fn:index-of must atomize values before comparison.
// ============================================================================

#[test]
fn fn_index_of_atomized_comparison() {
    let document = html::parse("<html><body><div>hello</div></body></html>").unwrap();
    // index-of with integer values (basic case)
    let xpath = xpath::parse("index-of((10, 20, 30, 20), 20)").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 2, "fn:index-of should find two matches");
    assert_eq!(result[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(result[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(4)));
}

// ============================================================================
// Regression: CR-7 - Node comparison (is/<</>>) must reject multi-item operands.
// ============================================================================

#[test]
fn node_comparison_rejects_multi_item_operands() {
    let document =
        html::parse("<html><body><div>a</div><div>b</div></body></html>").unwrap();
    let xpath = xpath::parse("//div is //div").unwrap();
    let result = xpath.apply(&document);
    assert!(
        result.is_err(),
        "Node comparison with multi-item operands should raise XPTY0004"
    );
}

// ============================================================================
// Regression: CR-9 - fn:distinct-values must treat Integer(1) and Double(1.0)
// as equal for deduplication.
// ============================================================================

#[test]
fn fn_distinct_values_cross_type_numeric() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("count(distinct-values((1, 1.0, 2)))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)),
        "fn:distinct-values should consider integer 1 and double 1.0 as equal"
    );
}

// ============================================================================
// Regression: CR-12 - fn:number(true()) should return 1.0, not NaN.
// ============================================================================

#[test]
fn fn_number_true_returns_one() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("number(true())").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(ordered_float::OrderedFloat(1.0))),
        "fn:number(true()) should return 1.0 per XPath spec"
    );
}

#[test]
fn fn_number_false_returns_zero() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("number(false())").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(ordered_float::OrderedFloat(0.0))),
        "fn:number(false()) should return 0.0 per XPath spec"
    );
}

// ============================================================================
// Regression: CR-14 - fn:substring(-INF, INF) should return the full string.
// ============================================================================

#[test]
fn fn_substring_neg_inf_pos_inf() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse(r#"substring("motor car", -1 div 0e0, 1 div 0e0)"#).unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("motor car".to_string())),
        "fn:substring with start=-INF and length=INF should return the full string"
    );
}

// ======================== Fix 2: fn:subsequence NaN/Infinity ========================

#[test]
fn fn_subsequence_nan_start_returns_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("subsequence((1,2,3), number('NaN'))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 0, "fn:subsequence with NaN start should return empty");
}

#[test]
fn fn_subsequence_pos_inf_start_returns_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("subsequence((1,2,3), 1 div 0e0)").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 0, "fn:subsequence with +INF start should return empty");
}

#[test]
fn fn_subsequence_neg_inf_start_2arg_returns_full() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("subsequence((1,2,3), -1 div 0e0)").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 3, "fn:subsequence with -INF start (2-arg) should return full sequence");
}

#[test]
fn fn_subsequence_nan_length_returns_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("subsequence((1,2,3), 1, number('NaN'))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 0, "fn:subsequence with NaN length should return empty");
}

#[test]
fn fn_subsequence_neg_inf_pos_inf_combo() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("subsequence((1,2,3), -1 div 0e0, 1 div 0e0)").unwrap();
    let result = xpath.apply(&document).unwrap();
    // -INF + INF = NaN for the end position, so this should return empty.
    assert_eq!(result.len(), 0, "fn:subsequence with -INF start and +INF length: -INF+INF=NaN end");
}

// ======================== Fix 3: fn:substring 2-arg -Infinity ========================

#[test]
fn fn_substring_neg_inf_2arg_returns_full() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"substring("hello", -1 div 0e0)"#).unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("hello".to_string())),
        "fn:substring with -INF start (2-arg) should return the full string"
    );
}

// ======================== Fix 4: boolean() EBV errors ========================

#[test]
fn boolean_function_item_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("boolean(true#0)").unwrap();
    let result = xpath.apply(&document);
    assert!(result.is_err(), "boolean(function-item) should error with FORG0006");
}

#[test]
fn boolean_multi_item_non_node_errors() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("boolean((1, 2))").unwrap();
    let result = xpath.apply(&document);
    assert!(result.is_err(), "boolean((1, 2)) should error with FORG0006");
}

// ======================== Fix 5: deep-equal cross-type ========================

#[test]
fn deep_equal_cross_type_numeric() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("deep-equal(1, 1.0e0)").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)),
        "deep-equal(1, 1.0e0) should be true (cross-type numeric)"
    );
}

// ======================== Fix 6: fn:sum overflow ========================

#[test]
fn fn_sum_integer_result_type() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("sum((1, 2, 3))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(6)),
        "sum((1,2,3)) should return Integer(6)"
    );
}

// ======================== Fix 7: fn:sort numeric ========================

#[test]
fn fn_sort_numeric_order() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("sort((2, 10, 1))").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
    assert_eq!(result[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(result[2], XpathItem::AnyAtomicType(AnyAtomicType::Integer(10)));
}

#[test]
fn fn_sort_string_still_works() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"sort(("banana", "apple", "cherry"))"#).unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("apple".to_string()))
    );
    assert_eq!(
        result[1],
        XpathItem::AnyAtomicType(AnyAtomicType::String("banana".to_string()))
    );
    assert_eq!(
        result[2],
        XpathItem::AnyAtomicType(AnyAtomicType::String("cherry".to_string()))
    );
}

// ======================== Fix 8: document node in union sort ========================

#[test]
fn union_with_document_node_sorts_doc_first() {
    let document = html::parse("<html><body><div>text</div></body></html>").unwrap();
    let xpath = xpath::parse("//div | /").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert!(result.len() >= 2, "union should have at least 2 items");
    match &result[0] {
        XpathItem::Node(XpathItemTreeNode::DocumentNode(_)) => {}
        other => panic!("First item in union should be document node, got: {:?}", other),
    }
}

// ======================== Fix 11: fn:function-name returns QName ========================

#[test]
fn fn_function_name_returns_qname() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("function-name(true#0)").unwrap();
    let result = xpath.apply(&document).unwrap();
    match &result[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::QName {
            local_name, prefix, ..
        }) => {
            assert_eq!(local_name, "true");
            assert_eq!(prefix.as_deref(), Some("fn"));
        }
        other => panic!("Expected QName, got: {:?}", other),
    }
}

// ======================== Fix 12: Regex XPath dialect ========================

#[test]
fn regex_xpath_initial_name_char() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"matches("hello", "^\i")"#).unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)),
        r#"matches("hello", "^\i") should be true (h is a letter)"#
    );
}

#[test]
fn regex_xpath_name_char_full() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"matches("a1", "^\c+$")"#).unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(
        result[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)),
        r#"matches("a1", "^\c+$") should be true (a and 1 are name chars)"#
    );
}

// ============================================================================
// Regression: SimpleMapExpr should set correct position/size context
// Per XPath 3.1 section 3.5.2, the ! operator should set position() and
// last() based on the LHS sequence.
// ============================================================================

#[test]
fn simple_map_position_returns_correct_positions() {
    let document = html::parse("<html><body><div>a</div><div>b</div><div>c</div></body></html>").unwrap();
    // (1 to 3) ! position() should return (1, 2, 3)
    let xpath = xpath::parse("(1 to 3) ! position()").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
    assert_eq!(result[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(result[2], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn simple_map_last_returns_correct_size() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // (1 to 4) ! last() should return (4, 4, 4, 4)
    let xpath = xpath::parse("(1 to 4) ! last()").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 4);
    for item in &result {
        assert_eq!(
            *item,
            XpathItem::AnyAtomicType(AnyAtomicType::Integer(4)),
            "last() in simple map should reflect LHS size"
        );
    }
}

// ============================================================================
// Regression: fn:round precision should be clamped to avoid overflow
// ============================================================================

#[test]
fn round_with_normal_precision() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("round(3.14159, 2)").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => {
            assert!(
                (d.0 - 3.14).abs() < 0.001,
                "round(3.14159, 2) should be approximately 3.14, got {}",
                d.0
            );
        }
        XpathItem::AnyAtomicType(AnyAtomicType::Float(f)) => {
            assert!(
                (f.0 - 3.14).abs() < 0.01,
                "round(3.14159, 2) should be approximately 3.14, got {}",
                f.0
            );
        }
        other => panic!("expected Double or Float, got {:?}", other),
    }
}

#[test]
fn round_with_extreme_precision_does_not_panic() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // Extreme precision that would previously overflow i32 or produce infinity.
    let xpath = xpath::parse("round(1.5, 1000000)").unwrap();
    let result = xpath.apply(&document);
    // Should not panic; the result may vary but must not crash.
    assert!(result.is_ok(), "round with extreme precision should not panic");
}

// ============================================================================
// Regression: fn:avg should handle string (xs:untypedAtomic) values
// ============================================================================

#[test]
fn avg_with_numeric_strings() {
    let document =
        html::parse("<html><body><span>10</span><span>20</span><span>30</span></body></html>")
            .unwrap();
    // //span/text() returns text nodes whose string values are "10", "20", "30".
    // fn:avg should promote these to doubles and compute the average.
    let xpath = xpath::parse("avg(//span/text())").unwrap();
    let result = xpath.apply(&document).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => {
            assert!(
                (d.0 - 20.0).abs() < 0.001,
                "avg of 10, 20, 30 should be 20.0, got {}",
                d.0
            );
        }
        other => panic!("expected Double, got {:?}", other),
    }
}

// ============================================================================
// Regression: QName PartialOrd should not include prefix
// ============================================================================

#[test]
fn qname_ordering_ignores_prefix() {
    use std::cmp::Ordering;
    // Two QNames with same namespace and local name but different prefix
    // should compare as equal.
    let qname1 = AnyAtomicType::QName {
        namespace_uri: "http://example.com".to_string(),
        local_name: "foo".to_string(),
        prefix: Some("a".to_string()),
    };
    let qname2 = AnyAtomicType::QName {
        namespace_uri: "http://example.com".to_string(),
        local_name: "foo".to_string(),
        prefix: Some("b".to_string()),
    };
    assert_eq!(
        qname1.partial_cmp(&qname2),
        Some(Ordering::Equal),
        "QNames with same namespace+localname but different prefix should be equal"
    );
}

// ============================================================================
// Regression: IfExpr Display should not include trailing newlines
// ============================================================================

#[test]
fn if_expr_display_no_trailing_newline() {
    let xpath = xpath::parse("if (true()) then 1 else 2").unwrap();
    let display = format!("{}", xpath);
    assert!(
        !display.ends_with('\n'),
        "IfExpr Display should not end with newline, got: {:?}",
        display
    );
}

// ============================================================================
// Regression: XpathItemSet::insert works after insertb removal
// Verify that insert (which replaced insertb) correctly adds items.
// ============================================================================

#[test]
fn xpath_item_set_insert_adds_items() {
    use skyscraper::xpath::xpath_item_set::XpathItemSet;

    let mut set = XpathItemSet::new();
    assert!(set.is_empty());

    set.insert(XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
    assert_eq!(set.len(), 1);

    set.insert(XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(set.len(), 2);

    // Duplicates are preserved (it's a sequence, not a set)
    set.insert(XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
    assert_eq!(set.len(), 3);
}

// ============================================================================
// Regression: XPath predicate evaluation uses 1-based positions correctly
// The new_with_variables method requires position >= 1. Verify that predicate
// filtering (which calls new_with_variables with i+1) works for all positions.
// ============================================================================

#[test]
fn predicate_position_one_based_first_child() {
    let text = "<ul><li>a</li><li>b</li><li>c</li></ul>";
    let tree = html::parse(text).unwrap();
    // /html/body/ul/li[1] selects the first li
    let xpath = xpath::parse("/html/body/ul/li[1]").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let el = result[0].extract_as_node().extract_as_element_node();
    assert_eq!(el.name, "li");
    let text = el.text_content(&tree);
    assert_eq!(text, "a");
}

#[test]
fn predicate_position_one_based_last_child() {
    let text = "<ul><li>a</li><li>b</li><li>c</li></ul>";
    let tree = html::parse(text).unwrap();
    // /html/body/ul/li[last()] selects the last li
    let xpath = xpath::parse("/html/body/ul/li[last()]").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let el = result[0].extract_as_node().extract_as_element_node();
    let text = el.text_content(&tree);
    assert_eq!(text, "c");
}

#[test]
fn predicate_position_one_based_all_positions() {
    let text = "<ul><li>a</li><li>b</li><li>c</li></ul>";
    let tree = html::parse(text).unwrap();
    // position() > 0 should match all elements (all positions are >= 1)
    let xpath = xpath::parse("/html/body/ul/li[position() > 0]").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 3);
}

// ============================================================================
// Regression: Unary negation of large values should use checked arithmetic.
// ============================================================================

#[test]
fn unary_negation_normal_values_work() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("-(42)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(-42));
}

#[test]
fn double_negation_returns_positive() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("-(-42)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(42));
}

// ============================================================================
// Regression: idiv should raise error for results outside i64 range.
// ============================================================================

#[test]
fn idiv_large_double_returns_error() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // 1e20 idiv 1 produces a result > i64::MAX; should return err:FOAR0002.
    let xpath = xpath::parse("1e20 idiv 1").unwrap();
    let result = xpath.apply(&tree);
    assert!(
        result.is_err(),
        "idiv result exceeding i64 range should return an error"
    );
}

#[test]
fn idiv_normal_values_work() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("7 idiv 2").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(3));
}

// ============================================================================
// Regression: fn:round with precision should preserve large integer precision.
// ============================================================================

#[test]
fn round_integer_negative_precision() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // round(12345, -2) should round to nearest 100 → 12300
    let xpath = xpath::parse("round(12345, -2)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(12300));
}

#[test]
fn round_integer_positive_precision_unchanged() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // round(12345, 2) — positive precision on an integer has no effect.
    let xpath = xpath::parse("round(12345, 2)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(12345));
}

#[test]
fn round_integer_zero_precision_unchanged() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // round(99999, 0) should return the value unchanged.
    let xpath = xpath::parse("round(99999, 0)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(99999));
}

#[test]
fn round_half_to_even_integer() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // round-half-to-even(2550, -2) — ties to even → 2600 (26 is even)
    let xpath = xpath::parse("round-half-to-even(2550, -2)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(2600));
}

#[test]
fn round_half_to_even_integer_ties_down() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // round-half-to-even(2450, -2) — ties to even → 2400 (24 is even)
    let xpath = xpath::parse("round-half-to-even(2450, -2)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Integer(2400));
}

// ============================================================================
// Regression: map:find should return a single array of all found values.
// ============================================================================

#[test]
fn map_find_returns_single_array() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // map:find on a map with matching key should return array(*) containing the values.
    let xpath = xpath::parse(r#"map:find(map { "a": 1, "b": 2 }, "a")"#).unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1, "map:find should return exactly one item (an array)");
    // The result should be an array containing the found value(s).
    match &result[0] {
        XpathItem::Function(skyscraper::xpath::grammar::data_model::Function::Array { members }) => {
            assert_eq!(members.len(), 1, "array should have one member for one matching key");
        }
        other => panic!("expected Function::Array, got {:?}", other),
    }
}

// ============================================================================
// Regression: fn:deep-equal should raise error for function items.
// ============================================================================

#[test]
fn deep_equal_function_items_raises_error() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // deep-equal on two function items should raise err:FOTY0015.
    let xpath =
        xpath::parse("deep-equal(true#0, false#0)").unwrap();
    let result = xpath.apply(&tree);
    assert!(
        result.is_err(),
        "fn:deep-equal on function items should raise an error"
    );
}

#[test]
fn deep_equal_atomic_values_still_works() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("deep-equal((1, 2, 3), (1, 2, 3))").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::Boolean(true));
}

// ============================================================================
// Regression: fn:substring with extreme f64 values must not overflow during
// the 1-based to 0-based index conversion.
// Previously, `(very_large_f64 as i64 - 1)` could overflow i64.
// ============================================================================

/// substring with a very large start position should return empty string.
#[test]
fn substring_extreme_large_start_returns_empty() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // 1e19 is larger than i64::MAX (~9.2e18), previously caused overflow.
    let xpath = xpath::parse(r#"substring("hello", 1e19)"#).unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::String(String::new()));
}

/// substring with a very large length should return from start to end.
#[test]
fn substring_extreme_large_length() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"substring("hello", 2, 1e19)"#).unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::String(String::from("ello")));
}

/// substring with a very negative start and large length should return full string.
#[test]
fn substring_extreme_negative_start_large_length() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"substring("hello", -1e19, 1e20)"#).unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::String(String::from("hello")));
}

/// substring with a very negative start and small length should return empty string.
#[test]
fn substring_extreme_negative_start_small_length() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"substring("hello", -1e19, 5)"#).unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1);
    let val = result[0].extract_as_any_atomic_type();
    assert_eq!(*val, AnyAtomicType::String(String::new()));
}

// ============================================================================
// Regression: idiv i64::MIN by -1 should return error, not panic from overflow.
// ============================================================================

#[test]
fn idiv_i64_min_by_neg_one_returns_error() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    // i64::MIN = -2147483648 * 2147483648 * 2 = -9223372036854775808.
    // Build it via multiplication of u32-range literals to reach i64::MIN,
    // then idiv by -1 which would overflow i64.
    let xpath =
        xpath::parse("(-2147483648 * 2147483648 * 2) idiv (-1)").unwrap();
    let result = xpath.apply(&tree);
    assert!(
        result.is_err(),
        "i64::MIN idiv -1 should return an error, not panic from overflow"
    );
}

// ============================================================================
// Regression: format-integer with NaN or Infinity should return error.
// ============================================================================

#[test]
fn format_integer_nan_returns_error() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("format-integer(number('NaN'), '1')").unwrap();
    let result = xpath.apply(&tree);
    assert!(
        result.is_err(),
        "format-integer(NaN, '1') should return an error"
    );
}

#[test]
fn format_integer_infinity_returns_error() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("format-integer(1 div 0, '1')").unwrap();
    let result = xpath.apply(&tree);
    assert!(
        result.is_err(),
        "format-integer(Infinity, '1') should return an error"
    );
}

// ============================================================================
// Regression: fn:root with too many arguments should return error.
// ============================================================================

#[test]
fn fn_root_too_many_args_returns_error() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("fn:root(1, 2)").unwrap();
    let result = xpath.apply(&tree);
    assert!(
        result.is_err(),
        "fn:root(1, 2) should return an error for too many arguments"
    );
}

// ============================================================================
// Regression: Range expression too large should return error.
// ============================================================================

#[test]
fn range_expr_too_large_returns_error() {
    let text = "<x/>";
    let tree = html::parse(text).unwrap();
    let xpath = xpath::parse("1 to 20000000").unwrap();
    let result = xpath.apply(&tree);
    assert!(
        result.is_err(),
        "1 to 20000000 should return an error (exceeds maximum range size)"
    );
}

// ============================================================================
// Regression: fn:innermost / fn:outermost correctness with HashSet optimization.
// ============================================================================

#[test]
fn fn_innermost_hashset_optimization_correct() {
    let text = "<html><body><div><p><span>deep</span></p></div></body></html>";
    let tree = html::parse(text).unwrap();
    // Select both div and span; innermost should return only span.
    let xpath = xpath::parse("innermost(//div | //span)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1, "innermost should return only the deepest node");
    let node = result[0].extract_as_node();
    match node {
        XpathItemTreeNode::ElementNode(e) => {
            assert_eq!(e.name, "span", "innermost should return span, not div");
        }
        other => panic!("Expected element node, got: {:?}", other),
    }
}

#[test]
fn fn_outermost_hashset_optimization_correct() {
    let text = "<html><body><div><p><span>deep</span></p></div></body></html>";
    let tree = html::parse(text).unwrap();
    // Select both div and span; outermost should return only div.
    let xpath = xpath::parse("outermost(//div | //span)").unwrap();
    let result = xpath.apply(&tree).unwrap();
    assert_eq!(result.len(), 1, "outermost should return only the shallowest node");
    let node = result[0].extract_as_node();
    match node {
        XpathItemTreeNode::ElementNode(e) => {
            assert_eq!(e.name, "div", "outermost should return div, not span");
        }
        other => panic!("Expected element node, got: {:?}", other),
    }
}
