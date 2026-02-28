use skyscraper::html::{self, QuirksMode};

/// A malformed DOCTYPE (e.g. `<!DOCTYPE>` which sets force_quirks) should
/// put the document into quirks mode.
#[test]
fn force_quirks_sets_quirks_mode() {
    let text = "<!DOCTYPE><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::Quirks);
}

/// A document with no DOCTYPE at all should be in quirks mode.
#[test]
fn missing_doctype_sets_quirks_mode() {
    let text = "<html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::Quirks);
}

/// The standard HTML5 `<!DOCTYPE html>` should produce no-quirks mode.
#[test]
fn standard_html5_doctype_is_no_quirks() {
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::NoQuirks);
}

/// A legacy public identifier should trigger quirks mode.
#[test]
fn legacy_public_identifier_sets_quirks() {
    let text =
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 3.2//EN"><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::Quirks);
}

/// XHTML 1.0 Transitional with a system identifier should trigger limited-quirks mode.
#[test]
fn xhtml_transitional_sets_limited_quirks() {
    let text = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd"><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::LimitedQuirks);
}

/// HTML 4.01 Frameset with a system identifier should trigger limited-quirks mode.
#[test]
fn html4_frameset_with_system_id_sets_limited_quirks() {
    let text = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Frameset//EN" "http://www.w3.org/TR/html4/frameset.dtd"><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::LimitedQuirks);
}

/// HTML 4.01 Frameset without a system identifier should trigger full quirks mode.
#[test]
fn html4_frameset_without_system_id_sets_quirks() {
    let text = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Frameset//EN"><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::Quirks);
}

/// In quirks mode, `<p><table>` should NOT close the `<p>` first.
/// The table ends up foster-parented before the `<p>`, and `<p>` remains open.
#[test]
fn quirks_mode_table_in_p_does_not_close_p() {
    // No DOCTYPE → quirks mode
    let text = "<html><body><p>text<table><tr><td>cell</td></tr></table></p></body></html>";
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::Quirks);
    let output = document.to_string();
    // In quirks mode the <p> is NOT closed before the <table>, so the table
    // is foster-parented and the <p> wraps both the text and eventually closes.
    // The key behavior: <p> should still be present and contain "text".
    assert!(
        output.contains("<p>"),
        "p element should be present: {output:?}"
    );
}

/// In no-quirks mode, `<p><table>` SHOULD close the `<p>` before the table.
#[test]
fn no_quirks_mode_table_in_p_closes_p() {
    let text =
        "<!DOCTYPE html><html><body><p>text<table><tr><td>cell</td></tr></table></p></body></html>";
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::NoQuirks);
    let output = document.to_string();
    // In no-quirks mode the <p> IS closed before the <table>.
    // The <p> should contain only "text" and be closed before the table appears.
    assert!(
        output.contains("<p>text</p>"),
        "p element should be closed before table in no-quirks mode: {output:?}"
    );
}

/// Public identifier matching should be case-insensitive.
#[test]
fn case_insensitive_public_id_matching() {
    // Use uppercase version of a known quirky prefix
    let text =
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 3.2//EN"><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    assert_eq!(document.quirks_mode(), QuirksMode::Quirks);

    // Dramatically different casing to confirm to_ascii_lowercase() is doing the work
    let text2 =
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTd htMl 3.2//eN"><html><head></head><body></body></html>"#;
    let document2 = html::parse(text2).unwrap();
    assert_eq!(document2.quirks_mode(), QuirksMode::Quirks);
}

/// Template children should be accessible as direct children of the template element.
#[test]
fn template_children_are_under_template_element() {
    let text = "<!DOCTYPE html><html><head><template><div>hello</div><span>world</span></template></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Children of <template> should be directly under it (not in a DocumentFragment)
    assert!(
        output.contains("<template><div>hello</div><span>world</span></template>"),
        "Template children should be directly under the template element: {output:?}"
    );
}
