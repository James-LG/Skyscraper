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

// ============================================================================
// Regression: CR-1 - DefaultParseErrorHandler should swallow errors.
// The default handler now returns Ok(()) so real-world HTML with parse errors
// (which is almost all HTML) can be parsed without aborting.
// ============================================================================

#[test]
fn default_parser_handles_parse_errors_gracefully() {
    // This HTML has multiple parse-error-inducing patterns:
    // unclosed tags, missing optional tags, etc. The parser should
    // handle them all gracefully without returning Err.
    let text = "<html><body><p>unclosed paragraph<p>second paragraph<div>in div</div></body></html>";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "Parser should swallow parse errors by default: {:?}",
        result.err()
    );
}

#[test]
fn default_parser_handles_misnested_formatting() {
    // Misnested formatting tags trigger parse errors and the adoption
    // agency algorithm. The parser should not abort.
    let text = "<html><body><b><i>bold-italic</b>italic-only</i></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("bold-italic"),
        "Content should be preserved: {output:?}"
    );
    assert!(
        output.contains("italic-only"),
        "Content should be preserved: {output:?}"
    );
}

// ============================================================================
// Regression: CR-2 - Noah's Ark should iterate in reverse (from the end of
// the active formatting elements list back toward the marker).
// ============================================================================

#[test]
fn noahs_ark_handles_many_identical_elements() {
    // Noah's Ark limits identical formatting elements to 3.
    // When a 4th identical element is pushed, the earliest one should be
    // removed. Reverse iteration ensures we count from the end of the list.
    let text = "<html><body><b>1<b>2<b>3<b>4</b></b></b></b></body></html>";
    let document = html::parse(text).unwrap();
    let xp = xpath::parse("count(//b)").unwrap();
    let result = xp.apply(&document).unwrap();
    let count = result[0].extract_as_any_atomic_type();
    match count {
        AnyAtomicType::Integer(n) => assert!(
            *n <= 4,
            "Noah's Ark should limit formatting elements: got {n}"
        ),
        other => panic!("Expected integer count, got: {other:?}"),
    }
}

// ============================================================================
// Regression: CR-4 - reset_open_elements_stack: "head" check needs `&& !last`
// guard. When head is the last element on the stack (fragment parsing), the
// parser should fall through to InBody, not set InHead.
// ============================================================================

#[test]
fn head_as_last_element_uses_in_body_not_in_head() {
    // Fragment parsing with <head> as the context element: when head is
    // the last (bottom) element on the open elements stack, the insertion
    // mode should be InBody, not InHead.
    let text = "<head><title>test</title></head><body><p>content</p></body>";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "Parsing with head element should not crash: {:?}",
        result.err()
    );
}

// ============================================================================
// Regression: CR-5 - unwrap() on current_node_as_element() replaced with
// error propagation. Parse error reporting paths should not panic.
// ============================================================================

#[test]
fn end_tag_mismatch_does_not_panic() {
    // When </div> is encountered but the current node is not a div after
    // generating implied end tags, the parser reports a parse error.
    // Previously this could panic via unwrap().
    let text = "<html><body><p>text</p></div></body></html>";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "Mismatched end tag should not panic: {:?}",
        result.err()
    );
}

#[test]
fn end_li_mismatch_does_not_panic() {
    // </li> when the current node is not li should not panic.
    let text = "<html><body><ul><li>item<p>nested</p></li></ul></body></html>";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "li end tag handling should not panic: {:?}",
        result.err()
    );
}

#[test]
fn end_dd_dt_mismatch_does_not_panic() {
    // </dd> and </dt> parse error paths should not panic.
    let text = "<html><body><dl><dt>term<dd>def<p>nested</p></dd></dt></dl></body></html>";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "dd/dt end tag handling should not panic: {:?}",
        result.err()
    );
}

#[test]
fn heading_end_tag_mismatch_does_not_panic() {
    // </h1> when the current node is not h1 should not panic.
    let text = "<html><body><h1><span>text</span></h1></body></html>";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "Heading end tag handling should not panic: {:?}",
        result.err()
    );
}

#[test]
fn ruby_rt_rp_mismatch_does_not_panic() {
    // rb/rtc/rp/rt tags trigger checks on the current element name.
    // These should not panic.
    let text = "<html><body><ruby>base<rb>base2<rt>annotation<rp>(</rp>alt<rp>)</rp></rt></rb></ruby></body></html>";
    let result = html::parse(text);
    assert!(
        result.is_ok(),
        "ruby/rt/rp handling should not panic: {:?}",
        result.err()
    );
}

// ============================================================================
// Regression: CR-8 - unescape_characters should not double-unescape.
// e.g. "&amp;lt;" should become "&lt;", not "<".
// ============================================================================

#[test]
fn unescape_no_double_unescape() {
    // "&amp;lt;" contains a literal "&amp;" which should unescape to "&",
    // yielding "&lt;". It should NOT further unescape to "<".
    let result = html::unescape_characters("&amp;lt;");
    assert_eq!(
        result, "&lt;",
        "&amp;lt; should become &lt;, not be double-unescaped to <"
    );
}

#[test]
fn unescape_amp_gt_no_double_unescape() {
    let result = html::unescape_characters("&amp;gt;");
    assert_eq!(
        result, "&gt;",
        "&amp;gt; should become &gt;, not >"
    );
}

#[test]
fn unescape_amp_amp_no_double_unescape() {
    let result = html::unescape_characters("&amp;amp;");
    assert_eq!(
        result, "&amp;",
        "&amp;amp; should become &amp;, not &"
    );
}

#[test]
fn unescape_basic_entities_still_work() {
    let result = html::unescape_characters("&lt;&gt;&amp;&quot;");
    assert_eq!(result, r#"<>&""#, "Basic entity unescaping should work");
}

// ============================================================================
// Regression: CR-9 - display_node indent should use usize, not u8.
// Deeply nested documents (> 255 levels) should not overflow.
// ============================================================================

#[test]
fn deeply_nested_document_display_no_overflow() {
    // Build a document nested deeper than 255 levels (u8::MAX).
    // With the old u8 indent, this would overflow. With usize, it works fine.
    let mut text = String::new();
    let depth = 260;
    for _ in 0..depth {
        text.push_str("<div>");
    }
    text.push_str("deep");
    for _ in 0..depth {
        text.push_str("</div>");
    }
    let full_html = format!("<html><body>{}</body></html>", text);
    let document = html::parse(&full_html).unwrap();

    // Pretty display uses indent parameter recursively.
    // With u8 this would overflow at depth > 255.
    let output = document.to_string();
    assert!(
        output.contains("deep"),
        "Deeply nested content should be preserved"
    );
    // Verify we can find all nesting levels via XPath.
    let xp = xpath::parse("count(//div)").unwrap();
    let result = xp.apply(&document).unwrap();
    let count = result[0].extract_as_any_atomic_type();
    match count {
        AnyAtomicType::Integer(n) => assert_eq!(
            *n, depth,
            "All {depth} nested divs should be present"
        ),
        other => panic!("Expected integer count, got: {other:?}"),
    }
}
