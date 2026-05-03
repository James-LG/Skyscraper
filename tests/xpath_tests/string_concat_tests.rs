use skyscraper::{html, xpath};

/// String concatenation with `||` should join two string literals
/// (XPath 3.1 section 3.10).
#[test]
fn string_concat_two_literals() {
    let text = r#"<html><body><div class="foobar">content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    // "foo" || "bar" = "foobar", which should match the class attribute.
    let xpath = xpath::parse(r#"//div[@class = "foo" || "bar"]"#).unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(
        nodes.len(),
        1,
        r#""foo" || "bar" should equal "foobar": {nodes:?}"#
    );
}

/// String concatenation with multiple `||` operators should chain
/// (XPath 3.1 section 3.10).
#[test]
fn string_concat_three_literals() {
    let text = r#"<html><body><div class="abcdef">content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"//div[@class = "ab" || "cd" || "ef"]"#).unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(
        nodes.len(),
        1,
        r#""ab" || "cd" || "ef" should equal "abcdef": {nodes:?}"#
    );
}

/// String concatenation should convert numeric values to strings
/// (XPath 3.1 section 3.10).
#[test]
fn string_concat_with_number() {
    let text = r#"<html><body><div class="item42">content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"//div[@class = "item" || 42]"#).unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(
        nodes.len(),
        1,
        r#""item" || 42 should equal "item42": {nodes:?}"#
    );
}

/// String concatenation of non-matching values should not match
/// (XPath 3.1 section 3.10).
#[test]
fn string_concat_no_match() {
    let text = r#"<html><body><div class="hello">content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"//div[@class = "good" || "bye"]"#).unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(
        nodes.len(),
        0,
        r#""good" || "bye" should not match "hello": {nodes:?}"#
    );
}
