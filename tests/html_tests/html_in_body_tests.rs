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

/// A <frameset> start tag in the body when frameset-ok is "not ok" (i.e. after
/// any non-whitespace character or certain tags that set frameset_ok = false)
/// should be ignored (WHATWG 13.2.6.4.7).
///
/// Any regular content in <body> sets frameset_ok to false, so a subsequent
/// <frameset> must be dropped.
#[test]
fn frameset_in_body_ignored_when_frameset_not_ok() {
    let text = "<html><body>text<frameset></frameset></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The frameset should not appear — frameset_ok was set to false by "text".
    assert!(
        !output.contains("<frameset"),
        "frameset should be ignored when frameset-ok is false: {output:?}"
    );
    assert!(
        output.contains("text"),
        "Body content should be preserved: {output:?}"
    );
}

/// A <frameset> start tag in the body when frameset-ok is still "ok" should
/// replace the body element and switch to InFrameset mode (WHATWG 13.2.6.4.7).
///
/// To reach InBody with frameset_ok=true, we use a tag like <div> that triggers
/// AfterHead's "anything else" (creating an implicit body without setting
/// frameset_ok to false). The <div> itself doesn't modify frameset_ok either.
/// Then <frameset> in InBody should detach body, pop the stack, insert
/// the frameset element, and switch to InFrameset.
#[test]
fn frameset_in_body_replaces_body_when_frameset_ok() {
    // <div> triggers AfterHead -> anything_else (implicit body, InBody, reprocess).
    // <div> in InBody doesn't set frameset_ok = false.
    // <frameset> in InBody with frameset_ok = true should succeed.
    let text = "<html><head></head><div><frameset></frameset></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<frameset>"),
        "frameset should be present: {output:?}"
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
