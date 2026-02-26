use skyscraper::html;

/// A DOCTYPE token encountered in the "before head" insertion mode should be
/// treated as a parse error and ignored (WHATWG 13.2.6.4.3).
///
/// Normal flow: DOCTYPE consumed in Initial mode, then <html> in BeforeHtml,
/// then BeforeHead. A second DOCTYPE here should be silently dropped.
#[test]
fn doctype_in_before_head_mode_is_ignored() {
    // The first DOCTYPE is consumed by Initial mode.
    // <html> transitions through BeforeHtml into BeforeHead.
    // The second DOCTYPE hits the BeforeHead handler and should be ignored.
    let text = "<!DOCTYPE html><html><!DOCTYPE html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only one DOCTYPE should be present in the output.
    assert_eq!(
        output.matches("<!DOCTYPE").count(),
        1,
        "Only one DOCTYPE should survive; got: {output:?}"
    );
    // The document structure should still be correct.
    assert!(
        output.contains("<head></head>"),
        "Should contain <head></head>: {output:?}"
    );
    assert!(
        output.contains("<body></body>"),
        "Should contain <body></body>: {output:?}"
    );
}

/// An <html> start tag encountered in the "before head" insertion mode should be
/// processed using the rules for "in body" (WHATWG 13.2.6.4.3).
///
/// The InBody rules for a duplicate <html> tag merge new attributes onto the
/// existing <html> element (without creating a second one).
#[test]
fn html_start_tag_in_before_head_merges_attributes() {
    // The first <html> creates the element, transitioning to BeforeHead.
    // The second <html lang="en"> hits the BeforeHead handler, which forwards
    // to InBody. InBody should merge the `lang` attribute onto the existing <html>.
    let text = r#"<html><html lang="en"><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only one <html> element should exist.
    assert_eq!(
        output.matches("<html").count(),
        1,
        "Only one <html> should exist; got: {output:?}"
    );
    // The merged attribute should be present.
    assert!(
        output.contains(r#"lang="en""#),
        "Should contain lang attribute: {output:?}"
    );
}

/// When a second <html> with no new attributes appears in "before head" mode,
/// it should be silently ignored (no duplicate element, no crash).
#[test]
fn duplicate_html_start_tag_in_before_head_no_new_attrs() {
    let text = "<html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(
        output.matches("<html").count(),
        1,
        "Only one <html> should exist; got: {output:?}"
    );
    assert!(
        output.contains("<head></head>"),
        "Should contain <head>: {output:?}"
    );
}

/// An unexpected end tag (not head/body/html/br) in "before head" mode should
/// be treated as a parse error and ignored (WHATWG 13.2.6.4.3).
#[test]
fn unexpected_end_tag_in_before_head_mode_is_ignored() {
    // After <html>, we are in BeforeHead. A stray </div> should be ignored.
    let text = "<html></div><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains("</div>"),
        "Stray </div> should not appear: {output:?}"
    );
    assert!(
        output.contains("<head></head>"),
        "Should contain <head></head>: {output:?}"
    );
}

/// Multiple unexpected end tags in "before head" mode should all be ignored.
#[test]
fn multiple_unexpected_end_tags_in_before_head_mode_are_ignored() {
    let text = "<html></span></footer></section><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains("</span>")
            && !output.contains("</footer>")
            && !output.contains("</section>"),
        "No stray end tags should appear: {output:?}"
    );
    assert!(
        output.contains("<head></head>"),
        "Should contain <head></head>: {output:?}"
    );
}
