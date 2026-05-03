use skyscraper::html;

/// An unclosed <title> element should trigger the text insertion mode EOF
/// handling: parse error, pop the element, switch to the original insertion
/// mode, and reprocess the EOF token (WHATWG 13.2.6.4.8).
///
/// The parser should not panic and the title element (with its partial text
/// content) should appear in the output tree.
#[test]
fn unclosed_title_eof_in_text_mode() {
    let text = "<html><head><title>hello";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The title should be present with its text content.
    assert!(
        output.contains("<title>"),
        "Should contain <title>: {output:?}"
    );
    assert!(
        output.contains("hello"),
        "Should contain the title text: {output:?}"
    );
}

/// An unclosed <style> element should also recover via the text insertion mode
/// EOF path, preserving whatever style text was consumed before EOF.
#[test]
fn unclosed_style_eof_in_text_mode() {
    let text = "<html><head><style>body { color: red }";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<style>"),
        "Should contain <style>: {output:?}"
    );
    assert!(
        output.contains("body { color: red }"),
        "Should contain the style text: {output:?}"
    );
}

/// An unclosed <textarea> in body should recover via the text insertion mode
/// EOF path.
#[test]
fn unclosed_textarea_eof_in_text_mode() {
    let text = "<html><body><textarea>some text";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<textarea>"),
        "Should contain <textarea>: {output:?}"
    );
    assert!(
        output.contains("some text"),
        "Should contain the textarea content: {output:?}"
    );
}

/// An unclosed <script> element should recover via the text insertion mode EOF
/// path. The spec has a distinct step for script elements (setting the "already
/// started" flag), but since scripting is not supported, it should behave like
/// any other raw-text element.
#[test]
fn unclosed_script_eof_in_text_mode() {
    let text = "<html><head><script>var x = 1;";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<script>"),
        "Should contain <script>: {output:?}"
    );
}

/// A completely empty <title> that is not closed should still recover cleanly.
#[test]
fn unclosed_empty_title_eof_in_text_mode() {
    let text = "<html><head><title>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<title>"),
        "Should contain <title>: {output:?}"
    );
}

/// Content after a properly closed title should parse normally — this is a
/// baseline sanity check that the fix didn't break the normal path.
#[test]
fn closed_title_normal_parsing() {
    let text = "<html><head><title>hi</title></head><body><p>body</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<title>hi</title>"),
        "Should contain properly closed title: {output:?}"
    );
    assert!(
        output.contains("<p>body</p>"),
        "Should contain body paragraph: {output:?}"
    );
}
