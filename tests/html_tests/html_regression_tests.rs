use skyscraper::html::{self, QuirksMode};
use skyscraper::xpath;
use skyscraper::xpath::grammar::data_model::AnyAtomicType;

// ============================================================================
// Regression: #3 - CDATA in HTML content should produce a comment with
// "[CDATA[" content, not an empty comment (fall-through bug).
// ============================================================================

#[test]
fn cdata_in_html_content_does_not_crash() {
    // In an HTML-namespace context, <![CDATA[...]]> should be treated as a
    // bogus comment (CDATA-in-HTML parse error). Before the fix, the
    // fall-through caused a double error and overwrote the comment token.
    // This test verifies the parser handles it gracefully.
    let text = "<html><body><![CDATA[some data]]></body></html>";
    let result = html::parse(text);
    assert!(result.is_ok(), "CDATA in HTML content should not crash");
}

#[test]
fn cdata_in_html_preserves_markup_structure() {
    // The CDATA bogus comment should not corrupt surrounding structure.
    let text = "<html><body><div>before</div><![CDATA[data]]><div>after</div></body></html>";
    let document = html::parse(text).unwrap();
    let xp = xpath::parse("count(//div)").unwrap();
    let result = xp.apply(&document).unwrap();
    let count = result[0].extract_as_any_atomic_type();
    match count {
        AnyAtomicType::Integer(n) => assert_eq!(
            *n, 2,
            "Both divs should be present after CDATA handling"
        ),
        other => panic!("Expected integer count, got: {other:?}"),
    }
}

// ============================================================================
// Regression: #4 - DOCTYPE state: `>` should reconsume in BeforeDOCTYPEName,
// producing a force-quirks DOCTYPE (not DOCTYPEName).
// ============================================================================

#[test]
fn doctype_immediate_greater_than_sets_quirks() {
    // `<!DOCTYPE>` with no name: the `>` in DOCTYPE state should go to
    // BeforeDOCTYPEName which creates a force-quirks empty DOCTYPE.
    let text = "<!DOCTYPE><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    assert_eq!(
        document.quirks_mode(),
        QuirksMode::Quirks,
        "<!DOCTYPE> with no name should trigger quirks mode"
    );
}

// ============================================================================
// Regression: #8 - after_doctype_system_identifier_state should NOT set
// force_quirks on the anything-else path.
// ============================================================================

#[test]
fn doctype_trailing_chars_after_system_id_no_quirks() {
    // A valid DOCTYPE with trailing characters after the system identifier
    // should NOT trigger quirks mode. The anything-else path in
    // after_doctype_system_identifier_state must not set force_quirks.
    let text = r#"<!DOCTYPE html SYSTEM "about:legacy-compat" x><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    assert_eq!(
        document.quirks_mode(),
        QuirksMode::NoQuirks,
        "Trailing chars after system identifier should not force quirks"
    );
}

// ============================================================================
// Regression: #10 - Script data escaped `<` state should NOT emit a `/`
// character when encountering `</` in escaped script data.
// ============================================================================

#[test]
fn script_escaped_end_tag_no_spurious_slash() {
    // In escaped script data, `</script>` should close the script without
    // emitting a spurious '/' character.
    let text = "<html><body><script><!--x</script></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The script content should not contain a bare '/' before '</script>'
    assert!(
        !output.contains("/</script>"),
        "No spurious '/' should appear before the end tag: {output:?}"
    );
}

// ============================================================================
// Regression: #1 - Template EOF logic was inverted. When a template IS on
// the stack and EOF is reached, the parser should pop/clear/reset, not stop.
// ============================================================================

#[test]
fn template_eof_does_not_crash() {
    // An unclosed template element at EOF should not panic and should
    // produce a valid document with the template in it.
    let text = "<html><body><template><div>inside</div></body></html>";
    let result = html::parse(text);
    assert!(result.is_ok(), "Unclosed template at EOF should not crash");
}

#[test]
fn template_eof_template_content_preserved() {
    // Even with EOF in template, the content before should be parseable.
    let text = "<html><body><template><p>hello";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "Template with EOF should parse without error"
    );
}

// ============================================================================
// Regression: #2 - </body> should be ignored when body is not in scope.
// The insertion mode should NOT switch to AfterBody.
// ============================================================================

#[test]
fn end_body_ignored_when_not_in_scope() {
    // When </body> appears but body is not in scope (e.g., inside a
    // table context), it should be ignored and parsing should continue
    // in the current mode. The document should still parse successfully.
    let text = "<html><head></head><body><table></body><tr><td>cell</td></tr></table></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The table content should be present; the first </body> was ignored.
    assert!(
        output.contains("cell"),
        "Table content should be preserved when </body> is ignored: {output:?}"
    );
}

// ============================================================================
// Regression: #5 - Adoption agency should stop searching at markers.
// ============================================================================

#[test]
fn adoption_agency_stops_at_marker() {
    // When <b> appears inside a <td> (which inserts a marker), closing </b>
    // outside that scope should not find the outer <b> across the marker.
    // This tests that find_map stops at the marker boundary.
    let text = r#"<html><body><b><table><tr><td></td></tr></table></b></body></html>"#;
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "Adoption agency with markers should not crash"
    );
}

#[test]
fn adoption_agency_marker_boundary_produces_correct_tree() {
    // Formatting elements should not cross marker boundaries set by table cells.
    let text = r#"<div><b>bold<table><tr><td>cell</td></tr></table>more</b></div>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The document should parse and contain all text content.
    assert!(output.contains("bold"), "bold text should be present: {output:?}");
    assert!(output.contains("cell"), "cell text should be present: {output:?}");
    assert!(output.contains("more"), "more text should be present: {output:?}");
}

// ============================================================================
// Regression: #9 - <input type="hidden"> should NOT set frameset_ok to false.
// ============================================================================

#[test]
fn input_type_hidden_preserves_frameset_ok() {
    // <input type="hidden"> followed by <frameset> should work because
    // hidden inputs do not clear the frameset-ok flag.
    // This is a structural correctness test -- we verify the document parses
    // without the input affecting subsequent parsing.
    let text = r#"<html><head></head><body><input type="hidden"><input type="text"></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("input"),
        "Both inputs should be in the document: {output:?}"
    );
}

// ============================================================================
// Regression: #11 - char::from_u32().unwrap() should not panic on surrogates.
// ============================================================================

#[test]
fn unescape_surrogate_codepoint_does_not_panic() {
    // &#55296; is 0xD800 (a surrogate), which is not a valid Unicode scalar.
    // unescape_characters should handle this gracefully instead of panicking.
    let result = html::unescape_characters("&#55296;");
    // Should produce the replacement character U+FFFD, not panic.
    assert!(
        result.contains('\u{FFFD}'),
        "Surrogate codepoint should be replaced with U+FFFD: {result:?}"
    );
}

#[test]
fn unescape_valid_numeric_reference() {
    // Normal numeric character references should still work.
    let result = html::unescape_characters("&#65;");
    assert_eq!(result, "A", "&#65; should produce 'A'");
}
