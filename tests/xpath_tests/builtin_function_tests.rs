use ordered_float::OrderedFloat;
use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

// ── Boolean functions ────────────────────────────────────────────────

#[test]
fn fn_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("true()").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("false()").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_not_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("not(true())").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_not_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("not(false())").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_not_empty_string() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"not("")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_boolean_of_nonempty_string() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"boolean("hello")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_boolean_of_zero() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("boolean(0)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

// ── Numeric functions ────────────────────────────────────────────────

#[test]
fn fn_number_from_string() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"number("42")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(OrderedFloat(42.0)))
    );
}

#[test]
fn fn_number_nan() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"number("abc")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    match &items[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => assert!(d.is_nan()),
        other => panic!("expected NaN double, got {:?}", other),
    }
}

#[test]
fn fn_abs_positive() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("abs(-5)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(5)));
}

#[test]
fn fn_ceiling() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("ceiling(1.5)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Float(OrderedFloat(2.0f32)))
    );
}

#[test]
fn fn_floor() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("floor(1.9)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Float(OrderedFloat(1.0f32)))
    );
}

#[test]
fn fn_round() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("round(1.5)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Float(OrderedFloat(2.0f32)))
    );
}

// ── String functions ─────────────────────────────────────────────────

#[test]
fn fn_concat() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"concat("hello", " ", "world")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello world")))
    );
}

#[test]
fn fn_string_join() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"string-join(("a", "b", "c"), "-")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("a-b-c")))
    );
}

#[test]
fn fn_string_length() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"string-length("hello")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(5)));
}

#[test]
fn fn_normalize_space() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"normalize-space("  hello   world  ")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello world")))
    );
}

#[test]
fn fn_upper_case() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"upper-case("hello")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("HELLO")))
    );
}

#[test]
fn fn_lower_case() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"lower-case("HELLO")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello")))
    );
}

#[test]
fn fn_starts_with_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"starts-with("hello world", "hello")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_starts_with_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"starts-with("hello world", "world")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_ends_with_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"ends-with("hello world", "world")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_substring() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"substring("hello", 2, 3)"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("ell")))
    );
}

#[test]
fn fn_substring_no_length() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"substring("hello", 2)"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("ello")))
    );
}

/// `substring("12345", 0, 3)` → `"12"` per spec (start before position 1).
#[test]
fn fn_substring_zero_start() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"substring("12345", 0, 3)"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("12")))
    );
}

/// `substring("12345", -1, 5)` → `"123"` per spec (negative start position).
#[test]
fn fn_substring_negative_start() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"substring("12345", -1, 5)"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("123")))
    );
}

#[test]
fn fn_substring_before() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"substring-before("hello-world", "-")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello")))
    );
}

#[test]
fn fn_substring_after() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"substring-after("hello-world", "-")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("world")))
    );
}

#[test]
fn fn_translate() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"translate("abc", "abc", "ABC")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("ABC")))
    );
}

// ── Sequence functions ───────────────────────────────────────────────

#[test]
fn fn_empty_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("empty(//nonexistent)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_empty_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("empty(//body)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_exists_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("exists(//body)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_count() {
    let document = html::parse("<html><body><p>1</p><p>2</p><p>3</p></body></html>").unwrap();
    let xpath = xpath::parse("count(//p)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn fn_head() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("head((1, 2, 3))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
}

#[test]
fn fn_tail() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("tail((1, 2, 3))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn fn_reverse() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("reverse((1, 2, 3))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(items[2], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
}

#[test]
fn fn_sum_integers() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("sum((1, 2, 3))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(6)));
}

// ── Node functions ───────────────────────────────────────────────────

#[test]
fn fn_name_of_element() {
    let document = html::parse("<html><body><div>hello</div></body></html>").unwrap();
    let xpath = xpath::parse("name(//div)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("div")))
    );
}

#[test]
fn fn_local_name_of_element() {
    let document = html::parse("<html><body><div>hello</div></body></html>").unwrap();
    let xpath = xpath::parse("local-name(//div)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("div")))
    );
}

// ── Context functions ────────────────────────────────────────────────

#[test]
fn fn_position_in_predicate() {
    let document =
        html::parse("<html><body><p>a</p><p>b</p><p>c</p></body></html>").unwrap();
    let xpath = xpath::parse("//p[position() = 2]").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.text_content(&document), "b");
}

#[test]
fn fn_last_in_predicate() {
    let document =
        html::parse("<html><body><p>a</p><p>b</p><p>c</p></body></html>").unwrap();
    let xpath = xpath::parse("//p[last()]").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.text_content(&document), "c");
}

// ── Combined usage ───────────────────────────────────────────────────

#[test]
fn fn_not_with_contains() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"not(contains("hello", "xyz"))"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_count_with_predicate() {
    let document = html::parse(
        r#"<html><body><div class="a">1</div><div class="b">2</div><div class="a">3</div></body></html>"#,
    )
    .unwrap();
    let xpath = xpath::parse(r#"count(//div[@class = "a"])"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

// ── Regex-based string functions ─────────────────────────────────────

#[test]
fn fn_matches_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"matches("hello123", "\d+")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_matches_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"matches("hello", "\d+")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_matches_case_insensitive() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"matches("Hello", "hello", "i")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_replace() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"replace("hello world", "world", "rust")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello rust")))
    );
}

#[test]
fn fn_replace_regex() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"replace("abc123def", "\d+", "NUM")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("abcNUMdef")))
    );
}

#[test]
fn fn_tokenize() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"tokenize("a-b-c", "-")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("a"))));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("b"))));
    assert_eq!(items[2], XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("c"))));
}

#[test]
fn fn_tokenize_whitespace() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"tokenize("  hello  world  ")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello"))));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("world"))));
}

// ── Additional sequence functions ────────────────────────────────────

#[test]
fn fn_subsequence() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("subsequence((1, 2, 3, 4, 5), 2, 3)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
    assert_eq!(items[2], XpathItem::AnyAtomicType(AnyAtomicType::Integer(4)));
}

#[test]
fn fn_insert_before() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("insert-before((1, 2, 3), 2, 99)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 4);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(99)));
    assert_eq!(items[2], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(items[3], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn fn_remove() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("remove((1, 2, 3), 2)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn fn_index_of() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("index-of((10, 20, 30), 20)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_zero_or_one_valid() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("zero-or-one(42)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(42)));
}

#[test]
fn fn_one_or_more_error() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("one-or-more(//nonexistent)").unwrap();
    assert!(xpath.apply(&document).is_err());
}

#[test]
fn fn_exactly_one_valid() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("exactly-one(42)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(42)));
}

#[test]
fn fn_avg() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("avg((1, 2, 3, 4))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(OrderedFloat(2.5)))
    );
}

#[test]
fn fn_max_integers() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("max((3, 1, 4, 1, 5))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(5)));
}

#[test]
fn fn_min_integers() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("min((3, 1, 4, 1, 5))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
}

#[test]
fn fn_zero_or_one_error() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("zero-or-one((1, 2))").unwrap();
    assert!(xpath.apply(&document).is_err());
}

#[test]
fn fn_exactly_one_error_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("exactly-one(//nonexistent)").unwrap();
    assert!(xpath.apply(&document).is_err());
}

#[test]
fn fn_exactly_one_error_multiple() {
    let document = html::parse("<html><body><p>a</p><p>b</p></body></html>").unwrap();
    let xpath = xpath::parse("exactly-one(//p)").unwrap();
    assert!(xpath.apply(&document).is_err());
}

// ── Numeric functions (additional) ───────────────────────────────────

#[test]
fn fn_round_half_to_even_basic() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("round-half-to-even(2.5)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Float(OrderedFloat(2.0f32)))
    );
}

#[test]
fn fn_round_half_to_even_odd() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("round-half-to-even(3.5)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Float(OrderedFloat(4.0f32)))
    );
}

#[test]
fn fn_round_half_to_even_precision() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("round-half-to-even(1.125, 2)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Float(OrderedFloat(1.12f32)))
    );
}

#[test]
fn fn_format_integer_decimal() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"format-integer(42, "1")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("42")))
    );
}

#[test]
fn fn_format_integer_alpha_lower() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"format-integer(3, "a")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("c")))
    );
}

#[test]
fn fn_format_integer_roman() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"format-integer(14, "I")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("XIV")))
    );
}

// ── String functions (additional) ────────────────────────────────────

#[test]
fn fn_compare_less() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"compare("abc", "def")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(-1)));
}

#[test]
fn fn_compare_equal() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"compare("abc", "abc")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(0)));
}

#[test]
fn fn_codepoint_equal_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"codepoint-equal("abc", "abc")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_codepoints_to_string() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("codepoints-to-string((72, 105))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("Hi")))
    );
}

#[test]
fn fn_string_to_codepoints() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"string-to-codepoints("Hi")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(72)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(105)));
}

#[test]
fn fn_encode_for_uri() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"encode-for-uri("hello world")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello%20world")))
    );
}

#[test]
fn fn_iri_to_uri() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"iri-to-uri("http://example.com/path?q=hello world")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from(
            "http://example.com/path?q=hello%20world"
        )))
    );
}

#[test]
fn fn_escape_html_uri() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse(r#"escape-html-uri("http://example.com/test#fragment")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from(
            "http://example.com/test#fragment"
        )))
    );
}

#[test]
fn fn_escape_html_uri_non_ascii() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // Tab character (0x09) is outside printable ASCII range and should be escaped.
    let xpath = xpath::parse(r#"escape-html-uri(concat("a", codepoints-to-string(9), "b"))"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("a%09b")))
    );
}

// ── Sequence functions (additional) ──────────────────────────────────

#[test]
fn fn_deep_equal_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("deep-equal((1, 2, 3), (1, 2, 3))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_deep_equal_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("deep-equal((1, 2, 3), (1, 2, 4))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_deep_equal_different_length() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("deep-equal((1, 2), (1, 2, 3))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_unordered() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("count(unordered((1, 2, 3)))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

// ── Node functions ───────────────────────────────────────────────────

#[test]
fn fn_has_children_true() {
    let document = html::parse("<html><body><p>text</p></body></html>").unwrap();
    let xpath = xpath::parse("//body/has-children()").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_has_children_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // head is auto-generated and empty.
    let xpath = xpath::parse("has-children(//head)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_path_element() {
    let document = html::parse("<html><body><p>hello</p></body></html>").unwrap();
    let xpath = xpath::parse("path(//p)").unwrap();
    let items = xpath.apply(&document).unwrap();
    let path_str = match &items[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::String(s)) => s.clone(),
        _ => panic!("Expected string"),
    };
    assert!(path_str.contains("html"), "path should contain 'html': {}", path_str);
    assert!(path_str.contains("body"), "path should contain 'body': {}", path_str);
    assert!(path_str.ends_with("p[1]"), "path should end with 'p[1]': {}", path_str);
}

#[test]
fn fn_namespace_uri_empty() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"namespace-uri(//body)"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new()))
    );
}

#[test]
fn fn_lang_match() {
    let document =
        html::parse(r#"<html lang="en-US"><body><p>hello</p></body></html>"#).unwrap();
    let xpath = xpath::parse(r#"//p/lang("en")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_lang_no_match() {
    let document =
        html::parse(r#"<html lang="en-US"><body><p>hello</p></body></html>"#).unwrap();
    let xpath = xpath::parse(r#"//p/lang("fr")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

// ── Accessor functions ───────────────────────────────────────────────

#[test]
fn fn_node_name_element() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("node-name(//body)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("body")))
    );
}

#[test]
fn fn_nilled_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("nilled(//body)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_generate_id() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("generate-id(//body)").unwrap();
    let items = xpath.apply(&document).unwrap();
    // Should return a non-empty string starting with N.
    match &items[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::String(s)) => {
            assert!(s.starts_with('N'), "generate-id should start with 'N': {}", s);
            assert!(s.len() > 1, "generate-id should not be empty");
        }
        _ => panic!("Expected string"),
    }
}

// ── Higher-order functions ───────────────────────────────────────────

#[test]
fn fn_for_each() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse("for-each((1, 2, 3), function($x) { $x * 2 })").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(4)));
    assert_eq!(items[2], XpathItem::AnyAtomicType(AnyAtomicType::Integer(6)));
}

#[test]
fn fn_filter() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse("filter((1, 2, 3, 4, 5), function($x) { $x > 3 })").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(4)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(5)));
}

#[test]
fn fn_fold_left() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(
        "fold-left((1, 2, 3), 0, function($acc, $x) { $acc + $x })",
    )
    .unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(6)));
}

#[test]
fn fn_fold_right() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(
        r#"fold-right(("a", "b", "c"), "", function($x, $acc) { concat($x, $acc) })"#,
    )
    .unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("abc")))
    );
}

#[test]
fn fn_for_each_pair() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(
        "for-each-pair((1, 2, 3), (4, 5, 6), function($a, $b) { $a + $b })",
    )
    .unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(5)));
    assert_eq!(items[1], XpathItem::AnyAtomicType(AnyAtomicType::Integer(7)));
    assert_eq!(items[2], XpathItem::AnyAtomicType(AnyAtomicType::Integer(9)));
}

#[test]
fn fn_sort_default() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"sort(("banana", "apple", "cherry"))"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("apple")))
    );
    assert_eq!(
        items[1],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("banana")))
    );
    assert_eq!(
        items[2],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("cherry")))
    );
}

#[test]
fn fn_function_name() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("function-name(fn:abs#1)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("fn:abs")))
    );
}

#[test]
fn fn_function_arity() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("function-arity(fn:contains#2)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_function_arity_inline() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse("function-arity(function($x, $y) { $x + $y })").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_apply() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("apply(fn:concat#2, (\"hello \", \"world\"))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello world")))
    );
}

// ── Map functions ────────────────────────────────────────────────────

#[test]
fn fn_map_size() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"map:size(map { "a": 1, "b": 2 })"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_map_keys() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"map:keys(map { "x": 1, "y": 2 })"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn fn_map_contains_true() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"map:contains(map { "a": 1 }, "a")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)));
}

#[test]
fn fn_map_contains_false() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"map:contains(map { "a": 1 }, "b")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)));
}

#[test]
fn fn_map_get() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"map:get(map { "x": 42 }, "x")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(42)));
}

#[test]
fn fn_map_put() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse(r#"map:size(map:put(map { "a": 1 }, "b", 2))"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_map_remove() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse(r#"map:size(map:remove(map { "a": 1, "b": 2 }, "a"))"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
}

#[test]
fn fn_map_entry() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"map:get(map:entry("key", 99), "key")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(99)));
}

#[test]
fn fn_map_merge() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(
        r#"map:size(map:merge((map { "a": 1 }, map { "b": 2 })))"#,
    )
    .unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_map_for_each() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(
        r#"count(map:for-each(map { "a": 1, "b": 2 }, function($k, $v) { $v }))"#,
    )
    .unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_map_find() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(
        r#"count(map:find(map { "a": 1, "b": 2 }, "a"))"#,
    )
    .unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)));
}

// ── Array functions ──────────────────────────────────────────────────

#[test]
fn fn_array_size() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("array:size([1, 2, 3])").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn fn_array_get() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("array:get([10, 20, 30], 2)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(20)));
}

#[test]
fn fn_array_head() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("array:head([10, 20, 30])").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(10)));
}

#[test]
fn fn_array_tail() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("array:size(array:tail([1, 2, 3]))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(2)));
}

#[test]
fn fn_array_reverse() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("array:get(array:reverse([1, 2, 3]), 1)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn fn_array_append() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("array:size(array:append([1, 2], 3))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

#[test]
fn fn_array_join() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("array:size(array:join(([1, 2], [3, 4])))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(4)));
}

#[test]
fn fn_array_flatten() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("count(array:flatten([1, 2, 3]))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items[0], XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)));
}

// ── Math functions ───────────────────────────────────────────────────

#[test]
fn fn_math_pi() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("math:pi()").unwrap();
    let items = xpath.apply(&document).unwrap();
    match &items[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => {
            assert!((d.0 - std::f64::consts::PI).abs() < 1e-10);
        }
        _ => panic!("Expected double"),
    }
}

#[test]
fn fn_math_sqrt() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("math:sqrt(4)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(OrderedFloat(2.0)))
    );
}

#[test]
fn fn_math_pow() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("math:pow(2, 10)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Double(OrderedFloat(1024.0)))
    );
}

// ── Format/normalize/node functions ─────────────────────────────────

#[test]
fn fn_format_number_basic() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse(r##"format-number(12345.6, "#,###.00")"##).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("12,345.60")))
    );
}

#[test]
fn fn_format_number_leading_zeros() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("format-number(-6, \"000\")").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("-006")))
    );
}

#[test]
fn fn_format_number_percent() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse("format-number(0.14, \"01%\")").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("14%")))
    );
}

#[test]
fn fn_format_number_nan() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath =
        xpath::parse(r##"format-number(number("NaN"), "#")"##).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("NaN")))
    );
}

#[test]
fn fn_normalize_unicode_nfc() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"normalize-unicode("hello")"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("hello")))
    );
}

#[test]
fn fn_normalize_unicode_nfkd() {
    let document = html::parse("<html><body></body></html>").unwrap();
    // U+00C9 (É) decomposes under NFKD to E + combining acute accent (2 codepoints).
    let xpath =
        xpath::parse("string-length(normalize-unicode(\"\u{00C9}\", \"NFKD\")) = 2").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

#[test]
fn fn_innermost() {
    let document = html::parse("<html><body><div><p>text</p></div></body></html>").unwrap();
    // //div returns the outer div; //p returns the inner p.
    // innermost should exclude div because p is a descendant of div.
    let xpath = xpath::parse("count(innermost(//div | //p))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}

#[test]
fn fn_outermost() {
    let document = html::parse("<html><body><div><p>text</p></div></body></html>").unwrap();
    // outermost should exclude p because its ancestor div is in the set.
    let xpath = xpath::parse("count(outermost(//div | //p))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1))
    );
}

#[test]
fn fn_outermost_returns_div() {
    let document = html::parse("<html><body><div><p>text</p></div></body></html>").unwrap();
    let xpath = xpath::parse("name(outermost(//div | //p))").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("div")))
    );
}

#[test]
fn fn_base_uri() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"base-uri()"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new()))
    );
}

#[test]
fn fn_document_uri() {
    let document = html::parse("<html><body></body></html>").unwrap();
    let xpath = xpath::parse(r#"empty(document-uri())"#).unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}
