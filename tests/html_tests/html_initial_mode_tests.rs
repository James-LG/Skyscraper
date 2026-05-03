use skyscraper::html;

/// Test that DOCTYPE is preserved in the parsed tree and serialized.
#[test]
fn doctype_is_preserved_in_output() {
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.starts_with("<!DOCTYPE html>"),
        "Output should start with DOCTYPE, got: {:?}",
        &output[..std::cmp::min(50, output.len())]
    );
}

/// Test that DOCTYPE with just a name is round-tripped correctly.
#[test]
fn doctype_simple_roundtrip() {
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(output, text);
}

/// Test that comments before the html element are preserved.
#[test]
fn comment_before_html_is_preserved() {
    let text = "<!DOCTYPE html><!-- test comment --><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- test comment -->"),
        "Output should contain the comment, got: {:?}",
        &output[..std::cmp::min(100, output.len())]
    );
}

/// Test that comment before DOCTYPE (in initial mode) is preserved.
#[test]
fn comment_in_initial_mode_is_preserved() {
    let text = "<!-- initial comment --><!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.starts_with("<!-- initial comment -->"),
        "Output should start with the comment, got: {:?}",
        &output[..std::cmp::min(80, output.len())]
    );
}

/// Test that multiple comments before html are preserved in order.
#[test]
fn multiple_comments_before_html_preserved() {
    let text =
        "<!DOCTYPE html><!-- first --><!-- second --><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(output, text);
}

/// Test that no DOCTYPE produces output without DOCTYPE.
#[test]
fn no_doctype_no_doctype_in_output() {
    let text = "<html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(output, text);
}

/// A second DOCTYPE encountered in the "before html" insertion mode should be
/// treated as a parse error and ignored (WHATWG 13.2.6.4.2).
#[test]
fn duplicate_doctype_in_before_html_mode_is_ignored() {
    // First DOCTYPE is consumed in the Initial mode, switching to BeforeHtml.
    // The second DOCTYPE hits the BeforeHtml handler and should be ignored.
    let text = "<!DOCTYPE html><!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only one DOCTYPE should be present in the output.
    assert_eq!(
        output.matches("<!DOCTYPE").count(),
        1,
        "Only one DOCTYPE should survive; got: {output:?}"
    );
    // The rest of the document should still parse correctly.
    assert!(output.contains("<html>"), "Should contain <html>: {output:?}");
}

/// An unexpected end tag (not head/body/html/br) in "before html" mode should
/// be treated as a parse error and ignored (WHATWG 13.2.6.4.2).
#[test]
fn unexpected_end_tag_in_before_html_mode_is_ignored() {
    let text = "<!DOCTYPE html></div><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The stray </div> should be silently dropped.
    assert!(
        !output.contains("</div>"),
        "Stray </div> should not appear in output: {output:?}"
    );
    // Document structure should still be correct.
    assert!(output.contains("<html>"), "Should contain <html>: {output:?}");
    assert!(
        output.contains("<body></body>"),
        "Should contain <body></body>: {output:?}"
    );
}

/// Multiple unexpected end tags in "before html" mode should all be ignored.
#[test]
fn multiple_unexpected_end_tags_in_before_html_mode_are_ignored() {
    let text = "<!DOCTYPE html></span></p></footer><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains("</span>") && !output.contains("</p>") && !output.contains("</footer>"),
        "No stray end tags should appear: {output:?}"
    );
}
