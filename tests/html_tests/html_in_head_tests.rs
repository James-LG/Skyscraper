use skyscraper::html;

/// A DOCTYPE token encountered in the "in head" insertion mode should be
/// treated as a parse error and ignored (WHATWG 13.2.6.4.4).
///
/// The first DOCTYPE is consumed by Initial mode. After <html><head>, we are
/// in InHead. A second DOCTYPE here should be silently dropped.
#[test]
fn doctype_in_head_mode_is_ignored() {
    let text = "<!DOCTYPE html><html><head><!DOCTYPE html></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only one DOCTYPE should be present in the output.
    assert_eq!(
        output.matches("<!DOCTYPE").count(),
        1,
        "Only one DOCTYPE should survive; got: {output:?}"
    );
    assert!(
        output.contains("<head></head>"),
        "Should contain <head></head>: {output:?}"
    );
    assert!(
        output.contains("<body></body>"),
        "Should contain <body></body>: {output:?}"
    );
}

/// An <html> start tag encountered in the "in head" insertion mode should be
/// processed using the rules for "in body" (WHATWG 13.2.6.4.4).
///
/// The InBody rules for a duplicate <html> tag merge new attributes onto the
/// existing <html> element (without creating a second one).
#[test]
fn html_start_tag_in_head_merges_attributes() {
    let text = r#"<html><head><html lang="en"></head><body></body></html>"#;
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

/// When a second <html> with no new attributes appears in "in head" mode,
/// it should be silently ignored (no duplicate element, no crash).
#[test]
fn duplicate_html_start_tag_in_head_no_new_attrs() {
    let text = "<html><head><html></head><body></body></html>";
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

/// A duplicate <head> start tag in the "in head" insertion mode should be
/// treated as a parse error and ignored (WHATWG 13.2.6.4.4).
#[test]
fn duplicate_head_start_tag_in_head_is_ignored() {
    let text = "<html><head><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only one <head> element should exist (the original).
    assert_eq!(
        output.matches("<head").count(),
        1,
        "Only one <head> should exist; got: {output:?}"
    );
    assert!(
        output.contains("<body></body>"),
        "Should contain <body></body>: {output:?}"
    );
}

/// A duplicate <head> with attributes should still be ignored — the attributes
/// should NOT be merged (unlike <html> which uses InBody rules).
#[test]
fn duplicate_head_with_attributes_is_ignored() {
    let text = r#"<html><head><head class="extra"></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(
        output.matches("<head").count(),
        1,
        "Only one <head> should exist; got: {output:?}"
    );
    // The duplicate's attributes should NOT appear.
    assert!(
        !output.contains("extra"),
        "Duplicate <head> attributes should not appear: {output:?}"
    );
}
