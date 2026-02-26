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

/// A <pre> start tag should close an open <p> element, insert the <pre>,
/// set frameset-ok to "not ok", and skip a leading newline (WHATWG 13.2.6.4.7).
#[test]
fn pre_start_tag_closes_p_and_inserts() {
    let text = "<html><body><p>para<pre>code</pre></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<pre>"),
        "pre element should be present: {output:?}"
    );
    // The p should be closed before pre.
    assert!(
        !output.contains("<p><pre>"),
        "p should not contain pre: {output:?}"
    );
}

/// A <listing> start tag should behave identically to <pre> (WHATWG 13.2.6.4.7).
#[test]
fn listing_start_tag_closes_p_and_inserts() {
    let text = "<html><body><p>para<listing>code</listing></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<listing>"),
        "listing element should be present: {output:?}"
    );
    assert!(
        !output.contains("<p><listing>"),
        "p should not contain listing: {output:?}"
    );
}

/// A newline immediately following <pre> should be stripped (WHATWG 13.2.6.4.7).
#[test]
fn pre_strips_leading_newline() {
    let text = "<html><body><pre>\nhello</pre></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The leading newline after <pre> should be stripped, leaving just "hello".
    assert!(
        output.contains("<pre>hello</pre>"),
        "Leading newline should be stripped: {output:?}"
    );
}

/// A non-LF character immediately following <pre> should NOT be stripped.
#[test]
fn pre_does_not_strip_non_lf() {
    let text = "<html><body><pre>hello</pre></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<pre>hello</pre>"),
        "Non-LF content should be preserved: {output:?}"
    );
}

/// A <dd> start tag should close an existing open <dd> and insert a new one
/// (WHATWG 13.2.6.4.7).
#[test]
fn dd_start_tag_closes_previous_dd() {
    let text = "<html><body><dl><dd>first<dd>second</dl></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The first dd should be implicitly closed by the second dd.
    assert!(
        output.contains("<dd>first</dd>"),
        "first dd should be closed: {output:?}"
    );
    assert!(
        output.contains("<dd>second</dd>"),
        "second dd should be present: {output:?}"
    );
}

/// A <dt> start tag should close an existing open <dd> element
/// (WHATWG 13.2.6.4.7).
#[test]
fn dt_start_tag_closes_previous_dd() {
    let text = "<html><body><dl><dd>desc<dt>term</dl></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<dd>desc</dd>"),
        "dd should be closed by dt: {output:?}"
    );
    assert!(
        output.contains("<dt>term</dt>"),
        "dt should be present: {output:?}"
    );
}

/// A <dd> start tag should close an existing open <dt> element
/// (WHATWG 13.2.6.4.7).
#[test]
fn dd_start_tag_closes_previous_dt() {
    let text = "<html><body><dl><dt>term<dd>desc</dl></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<dt>term</dt>"),
        "dt should be closed by dd: {output:?}"
    );
    assert!(
        output.contains("<dd>desc</dd>"),
        "dd should be present: {output:?}"
    );
}

/// A <dd>/<dt> start tag should close a <p> element in button scope
/// (WHATWG 13.2.6.4.7).
#[test]
fn dd_start_tag_closes_p_element() {
    let text = "<html><body><p>text<dd>desc</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains("<p><dd>"),
        "p should not contain dd: {output:?}"
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
