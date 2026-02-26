use skyscraper::html;

/// A NULL character (U+0000) inside the body should be treated as a parse error
/// and ignored per WHATWG 13.2.6.4.7.
///
/// The tokenizer (Data state) emits NULL as a character token; the tree
/// construction "in body" handler must drop it.
#[test]
fn null_character_in_body_is_ignored() {
    let text = "<html><body>hello\0world</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The NULL should be stripped — "hello" and "world" should appear without it.
    assert!(
        !output.contains('\0'),
        "NULL character should be stripped from output: {output:?}"
    );
    assert!(
        output.contains("helloworld"),
        "Text on either side of the NULL should be preserved: {output:?}"
    );
}

/// Multiple NULL characters interspersed with regular text should all be dropped.
#[test]
fn multiple_null_characters_in_body_are_ignored() {
    let text = "<html><body>\0a\0b\0c\0</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains('\0'),
        "No NULL characters should survive: {output:?}"
    );
    assert!(
        output.contains("abc"),
        "Non-NULL characters should be preserved: {output:?}"
    );
}

/// A NULL character inside an element nested in the body should also be ignored.
#[test]
fn null_character_in_nested_element_is_ignored() {
    let text = "<html><body><p>be\0fore</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains('\0'),
        "NULL should be stripped inside nested elements: {output:?}"
    );
    assert!(
        output.contains("before"),
        "Surrounding text should be intact: {output:?}"
    );
}

/// A DOCTYPE token encountered in the "in body" insertion mode should be
/// treated as a parse error and ignored (WHATWG 13.2.6.4.7).
///
/// The initial DOCTYPE is consumed by Initial mode. A second DOCTYPE
/// appearing inside <body> should be silently dropped.
#[test]
fn doctype_in_body_is_ignored() {
    let text = "<!DOCTYPE html><html><body><!DOCTYPE html><p>text</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only one DOCTYPE should be present in the output.
    assert_eq!(
        output.matches("<!DOCTYPE").count(),
        1,
        "Only one DOCTYPE should survive; got: {output:?}"
    );
    assert!(
        output.contains("<p>text</p>"),
        "Body content should be preserved: {output:?}"
    );
}
