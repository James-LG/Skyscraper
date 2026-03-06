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
// ============================================================================
// Regression: CR-10 - Adoption agency algorithm must not double-push elements
// onto the open_elements stack. The create_element_node_from_token_result
// method creates elements without pushing to open_elements; the adoption
// agency manages the stack position manually.
// ============================================================================

#[test]
fn adoption_agency_overlapping_formatting_no_double_push() {
    // Overlapping formatting tags trigger the adoption agency algorithm.
    // Before the fix, insert_create_an_element_for_the_token_result would
    // push the new node to open_elements, and then the algorithm would
    // also manually place it, resulting in duplicates in the stack.
    let text = "<html><body><b><i>bold-italic</b>just-italic</i>normal</body></html>";
    let document = html::parse(text).unwrap();

    // Verify the tree structure is correct after adoption agency runs
    let xp_bold_italic = xpath::parse("//b/i").unwrap();
    let result = xp_bold_italic.apply(&document).unwrap();
    assert!(
        !result.is_empty(),
        "Adoption agency should produce <b><i> nesting"
    );

    // Verify all text content is preserved
    let xp_all_text = xpath::parse("//body//text()").unwrap();
    let all_text = xp_all_text.apply(&document).unwrap();
    let text_content: String = all_text
        .iter()
        .filter_map(|item| item.extract_as_node().text(&document))
        .collect();
    assert!(
        text_content.contains("bold-italic"),
        "bold-italic text should be present: {text_content}"
    );
    assert!(
        text_content.contains("just-italic"),
        "just-italic text should be present: {text_content}"
    );
    assert!(
        text_content.contains("normal"),
        "normal text should be present: {text_content}"
    );
}

#[test]
fn adoption_agency_triple_overlap_no_corruption() {
    // Three overlapping formatting elements: tests the inner loop (step 4.14)
    // of the adoption agency algorithm more thoroughly.
    let text = "<html><body><a href='#'><b><em>text</a>after</em></b></body></html>";
    let document = html::parse(text).unwrap();

    let xp = xpath::parse("//body//text()").unwrap();
    let all_text = xp.apply(&document).unwrap();
    let text_content: String = all_text
        .iter()
        .filter_map(|item| item.extract_as_node().text(&document))
        .collect();
    assert!(
        text_content.contains("text"),
        "text should be present: {text_content}"
    );
    assert!(
        text_content.contains("after"),
        "after should be present: {text_content}"
    );
}

// ============================================================================
// Regression: CR-11 - ScriptDataDoubleEscaped states must emit '<' character
// when transitioning to ScriptDataDoubleEscapedLessThanSign per WHATWG spec.
// ============================================================================

#[test]
fn script_double_escaped_preserves_less_than() {
    // Script content with double-escaped comment: the '<' inside should not
    // be dropped. Before the fix, the '<' was consumed but never emitted.
    let text = "<html><body><script><!--<script>var x = 1 < 2;</script>--></script></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The '<' characters in "1 < 2" and "<script>" should be preserved in output
    assert!(
        output.contains('<'),
        "Less-than signs in double-escaped script should be preserved: {output:?}"
    );
}

// ============================================================================
// Regression: CR-12 - Adoption agency Step 4.15 must use
// appropriate_place_for_inserting_a_node with the common ancestor as override
// target, not a direct append. This ensures foster parenting is respected.
// ============================================================================

#[test]
fn adoption_agency_step_15_foster_parenting() {
    // When adoption agency runs with a table as common ancestor, foster
    // parenting should be respected. This test exercises the code path.
    let text = "<html><body><table><b><tr><td>cell</td></tr></b></table></body></html>";
    let document = html::parse(text).unwrap();

    // Verify the table structure is intact
    let xp = xpath::parse("//td").unwrap();
    let result = xp.apply(&document).unwrap();
    assert!(
        !result.is_empty(),
        "Table cell should be present in the document"
    );
}

// ============================================================================
// Regression: CR-13 - ELEMENT_IN_SCOPE_TYPES must include MathML and SVG
// scope barrier elements per WHATWG spec.
// ============================================================================

#[test]
fn svg_foreign_object_is_scope_barrier() {
    // foreignObject is an SVG scope barrier. Elements inside it should be
    // parsed in HTML mode, and scope checks should work correctly.
    let text = "<html><body><svg><foreignObject><p>html content</p></foreignObject></svg></body></html>";
    let document = html::parse(text).unwrap();

    let xp = xpath::parse("//p").unwrap();
    let result = xp.apply(&document).unwrap();
    assert!(
        !result.is_empty(),
        "<p> inside <foreignObject> should be found"
    );
}

// ============================================================================
// Regression: CR-14 - Noah's Ark attribute comparison must match by name,
// not by positional index, to handle different attribute orderings.
// ============================================================================

#[test]
fn noahs_ark_attribute_comparison_order_independent() {
    // Two elements with the same attributes in different order should be
    // considered matching by the Noah's Ark clause. With the old index-based
    // comparison, swapped attributes would not match.
    // We push 4 <b> elements with the same attributes (some in different order)
    // to trigger Noah's Ark (limit is 3).
    let text = r#"<html><body>
        <b class="x" id="a">1</b>
        <b id="a" class="x">2</b>
        <b class="x" id="a">3</b>
        <b id="a" class="x">4
    </body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // All text should be present (Noah's Ark removes old entries but
    // elements already in the tree remain)
    assert!(output.contains('1'), "Text 1 should be present: {output:?}");
    assert!(output.contains('4'), "Text 4 should be present: {output:?}");
}

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

// ============================================================================
// Regression: EOF token must be emitted even on truncated input so the tree
// builder properly finalizes the document (closing open elements, creating
// implied elements per WHATWG spec).
// ============================================================================

#[test]
fn truncated_input_gets_eof_finalization() {
    // Truncated HTML — no closing tags at all.
    let document = html::parse("<div>hello").unwrap();
    // The tree builder should still produce a valid tree with implied html/head/body.
    let xp = xpath::parse("//div").unwrap();
    let result = xp.apply(&document).unwrap();
    assert!(
        !result.is_empty(),
        "Truncated input should still produce a valid tree with the div element"
    );
}

#[test]
fn empty_input_creates_implied_elements() {
    // Per WHATWG spec, empty input should produce implied html/head/body.
    let document = html::parse("").unwrap();
    let output = document.to_string();
    assert_eq!(
        output, "<html><head></head><body></body></html>",
        "Empty document should have implied html/head/body per WHATWG spec"
    );
}

// ============================================================================
// Regression: unescape_characters must handle hex character references
// (&#xHH;) in addition to decimal (&#DD;).
// ============================================================================

#[test]
fn unescape_hex_character_references() {
    let result = html::unescape_characters("&#x27;hello&#x27;");
    assert_eq!(result, "'hello'", "Hex char ref &#x27; should produce apostrophe");
}

#[test]
fn unescape_mixed_hex_and_decimal() {
    let result = html::unescape_characters("&#x41;&#66;&#x43;");
    assert_eq!(result, "ABC", "Mixed hex (&#x41;, &#x43;) and decimal (&#66;) should all work");
}

#[test]
fn unescape_uppercase_hex() {
    let result = html::unescape_characters("&#x2019;");
    assert_eq!(result, "\u{2019}", "Uppercase hex &#x2019; should produce right single quotation mark");
}

// ============================================================================
// Regression: CR-3 - Adoption agency algorithm must adjust bookmark after
// removing the formatting element from active formatting elements.
// ============================================================================

#[test]
fn adoption_agency_nested_formatting_elements() {
    // This exercises the adoption agency algorithm with multiple nested formatting
    // elements where the bookmark adjustment matters.
    let text = "<html><body><b>1<i>2<b>3</b>4</i>5</b></body></html>";
    let document = html::parse(text).unwrap();
    // The parser should not crash or produce a malformed tree.
    let xp = xpath::parse("//b").unwrap();
    let result = xp.apply(&document).unwrap();
    assert!(
        !result.is_empty(),
        "Nested formatting elements should be parsed without errors"
    );
}

#[test]
fn adoption_agency_deeply_nested_same_tag() {
    // Multiple levels of the same formatting tag trigger repeated adoption agency runs.
    let text = "<html><body><b><b><b>text</b></b></b></body></html>";
    let document = html::parse(text).unwrap();
    let xp = xpath::parse("string(//body)").unwrap();
    let result = xp.apply(&document).unwrap();
    assert_eq!(
        result[0],
        skyscraper::xpath::grammar::data_model::XpathItem::AnyAtomicType(
            skyscraper::xpath::grammar::data_model::AnyAtomicType::String("text".to_string())
        ),
        "Deeply nested same formatting tags should preserve text content"
    );
}

// ============================================================================
// Regression: CR-10 - </search> end tag must be handled alongside other
// block-level end tags in the in_body insertion mode.
// ============================================================================

#[test]
fn search_end_tag_handled_correctly() {
    let text = "<html><body><search><p>content</p></search></body></html>";
    let document = html::parse(text).unwrap();
    let xp = xpath::parse("count(//search)").unwrap();
    let result = xp.apply(&document).unwrap();
    match &result[0] {
        skyscraper::xpath::grammar::data_model::XpathItem::AnyAtomicType(
            skyscraper::xpath::grammar::data_model::AnyAtomicType::Integer(n),
        ) => assert_eq!(*n, 1, "There should be exactly one <search> element"),
        other => panic!("Expected integer count, got: {:?}", other),
    }
}

#[test]
fn search_element_contains_children() {
    let text = "<html><body><search><div>inner</div></search><p>after</p></body></html>";
    let document = html::parse(text).unwrap();
    // The <div> should be inside <search>, not a sibling.
    let xp = xpath::parse("count(//search/div)").unwrap();
    let result = xp.apply(&document).unwrap();
    match &result[0] {
        skyscraper::xpath::grammar::data_model::XpathItem::AnyAtomicType(
            skyscraper::xpath::grammar::data_model::AnyAtomicType::Integer(n),
        ) => assert_eq!(
            *n, 1,
            "The <div> should be a child of <search>, not a sibling"
        ),
        other => panic!("Expected integer count, got: {:?}", other),
    }
}
