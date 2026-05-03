use skyscraper::html;

/// A comment token encountered in the "after head" insertion mode should be
/// inserted as a comment node (WHATWG 13.2.6.4.6).
#[test]
fn comment_in_after_head_mode_is_inserted() {
    // After </head> we're in AfterHead. A comment here should be inserted.
    let text = "<html><head></head><!-- after head comment --><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- after head comment -->"),
        "Comment should be preserved: {output:?}"
    );
}

/// A DOCTYPE token encountered in the "after head" insertion mode should be
/// treated as a parse error and ignored (WHATWG 13.2.6.4.6).
#[test]
fn doctype_in_after_head_mode_is_ignored() {
    // The first DOCTYPE is consumed by Initial mode. After </head>, the parser
    // is in AfterHead mode. A second DOCTYPE here should be silently dropped.
    let text = "<!DOCTYPE html><html><head></head><!DOCTYPE html><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(
        output.matches("<!DOCTYPE").count(),
        1,
        "Only one DOCTYPE should survive; got: {output:?}"
    );
}

/// An <html> start tag in "after head" should be processed using InBody rules,
/// which merges new attributes onto the existing <html> element (WHATWG 13.2.6.4.6).
#[test]
fn html_start_tag_in_after_head_merges_attributes() {
    let text = r#"<html><head></head><html lang="en"><body></body></html>"#;
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

/// A duplicate <head> start tag in "after head" mode should be treated as a
/// parse error and ignored (WHATWG 13.2.6.4.6).
#[test]
fn duplicate_head_start_tag_in_after_head_is_ignored() {
    let text = "<html><head></head><head><title>oops</title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only one <head> element should exist.
    assert_eq!(
        output.matches("<head").count(),
        1,
        "Only one <head> should exist; got: {output:?}"
    );
}

/// An unexpected end tag (not body, html, or br) in "after head" mode should
/// be treated as a parse error and ignored (WHATWG 13.2.6.4.6).
#[test]
fn unexpected_end_tag_in_after_head_is_ignored() {
    let text = "<html><head></head></div><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The </div> should be ignored, no extra elements created.
    assert!(
        output.contains("<body></body>"),
        "Should still have <body>: {output:?}"
    );
}

/// End tags for body, html, or br in "after head" mode should trigger the
/// "anything else" path — inserting a <body> element and reprocessing
/// (WHATWG 13.2.6.4.6).
#[test]
fn body_end_tag_in_after_head_triggers_anything_else() {
    // </body> in after-head should create a <body> and reprocess.
    let text = "<html><head></head></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<body>"),
        "A <body> should be implicitly created: {output:?}"
    );
}

/// Head-level elements appearing after </head> should be processed using
/// "in head" rules with the head element pointer pushed/popped on the
/// stack of open elements (WHATWG 13.2.6.4.6).
///
/// A <meta> tag after </head> should end up inside the <head> element.
#[test]
fn meta_after_head_is_inserted_in_head() {
    let text = r#"<html><head></head><meta charset="utf-8"><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The <meta> should be inside <head>, not floating outside.
    assert!(
        output.contains("<head><meta"),
        "Meta should be inside head: {output:?}"
    );
}

/// A <title> tag appearing after </head> should be inserted into <head>
/// via the head element pointer mechanism (WHATWG 13.2.6.4.6).
#[test]
fn title_after_head_is_inserted_in_head() {
    let text = "<html><head></head><title>Late Title</title><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<head><title>"),
        "Title should be inside head: {output:?}"
    );
    assert!(
        output.contains("Late Title"),
        "Title content should be preserved: {output:?}"
    );
}

/// A <link> tag appearing after </head> should be inserted into <head>
/// via the head element pointer mechanism (WHATWG 13.2.6.4.6).
#[test]
fn link_after_head_is_inserted_in_head() {
    let text = r#"<html><head></head><link rel="stylesheet" href="a.css"><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<head><link"),
        "Link should be inside head: {output:?}"
    );
}

/// A <style> tag appearing after </head> should be inserted into <head>
/// via the head element pointer mechanism (WHATWG 13.2.6.4.6).
#[test]
fn style_after_head_is_inserted_in_head() {
    let text = "<html><head></head><style>body { color: red; }</style><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<head><style>"),
        "Style should be inside head: {output:?}"
    );
}

/// A <base> tag appearing after </head> should be inserted into <head>
/// via the head element pointer mechanism (WHATWG 13.2.6.4.6).
#[test]
fn base_after_head_is_inserted_in_head() {
    let text = r#"<html><head></head><base href="/"><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<head><base"),
        "Base should be inside head: {output:?}"
    );
}

/// A <script> tag appearing after </head> should be inserted into <head>
/// via the head element pointer mechanism (WHATWG 13.2.6.4.6).
/// This exercises the ScriptData tokenizer state path through Text mode.
#[test]
fn script_after_head_is_inserted_in_head() {
    let text = "<html><head></head><script>var x = 1;</script><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<head><script>"),
        "Script should be inside head: {output:?}"
    );
    assert!(
        output.contains("var x = 1;"),
        "Script content should be preserved: {output:?}"
    );
}

/// A <noframes> tag appearing after </head> should be inserted into <head>
/// via the head element pointer mechanism (WHATWG 13.2.6.4.6).
/// This exercises the generic raw text element parsing algorithm path.
#[test]
fn noframes_after_head_is_inserted_in_head() {
    let text = "<html><head></head><noframes>No frames</noframes><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<head><noframes>"),
        "Noframes should be inside head: {output:?}"
    );
}

/// Multiple head-level elements after </head> should all be placed inside
/// <head> via the head element pointer mechanism (WHATWG 13.2.6.4.6).
#[test]
fn multiple_head_elements_after_head_all_go_in_head() {
    let text = r#"<html><head></head><meta charset="utf-8"><title>T</title><link rel="icon" href="f"><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<head><meta"),
        "Meta should be inside head: {output:?}"
    );
    assert!(
        output.contains("<title>"),
        "Title should be inside head: {output:?}"
    );
    assert!(
        output.contains("<link"),
        "Link should be inside head: {output:?}"
    );
}
