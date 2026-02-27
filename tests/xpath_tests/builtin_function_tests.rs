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
