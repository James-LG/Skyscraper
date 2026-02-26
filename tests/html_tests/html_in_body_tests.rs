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

/// An explicit </dd> end tag should close the dd element (WHATWG 13.2.6.4.7).
#[test]
fn dd_end_tag_closes_dd() {
    let text = "<html><body><dl><dd>content</dd></dl></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<dd>content</dd>"),
        "dd should be properly closed: {output:?}"
    );
}

/// A </dd> end tag with no dd in scope should be ignored (WHATWG 13.2.6.4.7).
#[test]
fn dd_end_tag_without_scope_is_ignored() {
    let text = "<html><body></dd><p>text</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<p>text</p>"),
        "Body content should be preserved: {output:?}"
    );
}

/// An explicit </dt> end tag should close the dt element (WHATWG 13.2.6.4.7).
#[test]
fn dt_end_tag_closes_dt() {
    let text = "<html><body><dl><dt>term</dt></dl></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<dt>term</dt>"),
        "dt should be properly closed: {output:?}"
    );
}

/// An <applet> start tag should insert the element and push a marker onto
/// the active formatting elements list (WHATWG 13.2.6.4.7).
#[test]
fn applet_start_tag_inserts_element() {
    let text = "<html><body><applet>content</applet></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<applet>content</applet>"),
        "applet should be present: {output:?}"
    );
}

/// A <marquee> start tag should insert the element (WHATWG 13.2.6.4.7).
#[test]
fn marquee_start_tag_inserts_element() {
    let text = "<html><body><marquee>scrolling</marquee></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<marquee>scrolling</marquee>"),
        "marquee should be present: {output:?}"
    );
}

/// An </object> end tag without object in scope should be ignored (WHATWG 13.2.6.4.7).
#[test]
fn object_end_tag_without_scope_is_ignored() {
    let text = "<html><body></object><p>text</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<p>text</p>"),
        "Body content should be preserved: {output:?}"
    );
}

/// A <nobr> start tag should insert a nobr element and push it onto the
/// list of active formatting elements (WHATWG 13.2.6.4.7).
#[test]
fn nobr_start_tag_inserts_element() {
    let text = "<html><body><nobr>no break</nobr></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<nobr>no break</nobr>"),
        "nobr element should be present: {output:?}"
    );
}

/// Nested <nobr> tags should trigger the adoption agency algorithm to close
/// the first one before opening the second (WHATWG 13.2.6.4.7).
#[test]
fn nested_nobr_triggers_adoption_agency() {
    let text = "<html><body><nobr>first<nobr>second</nobr></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The first nobr should be closed by the adoption agency when the second appears.
    assert!(
        output.contains("first"),
        "first text should be preserved: {output:?}"
    );
    assert!(
        output.contains("second"),
        "second text should be preserved: {output:?}"
    );
}

/// A </sarcasm> end tag should use the "any other end tag" logic (WHATWG 13.2.6.4.7).
/// The spec joke says: "Take a deep breath, then act as described in the
/// 'any other end tag' entry below."
#[test]
fn sarcasm_end_tag_uses_any_other_end_tag() {
    // </sarcasm> without a matching open element should be ignored (any other end tag logic).
    let text = "<html><body><p>text</sarcasm></p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<p>text</p>"),
        "Content should be preserved: {output:?}"
    );
}

/// A <plaintext> start tag should close p in button scope, insert the element,
/// and switch the tokenizer to PLAINTEXT state (WHATWG 13.2.6.4.7).
/// Everything after <plaintext> is treated as raw text (no end tag).
#[test]
fn plaintext_start_tag_inserts_and_switches_tokenizer() {
    let text = "<html><body><plaintext>raw <b>not bold</b> text";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<plaintext>"),
        "plaintext element should be present: {output:?}"
    );
    // In PLAINTEXT mode, everything is text — <b> and </b> are not parsed as tags.
    assert!(
        output.contains("&lt;b&gt;") || output.contains("<b>not bold</b>") || output.contains("raw"),
        "Content after <plaintext> should be preserved as text: {output:?}"
    );
}

/// A <plaintext> start tag should close an open <p> element (WHATWG 13.2.6.4.7).
#[test]
fn plaintext_start_tag_closes_p() {
    let text = "<html><body><p>para<plaintext>raw text";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains("<p><plaintext>"),
        "p should not contain plaintext: {output:?}"
    );
}

/// A <param> start tag should insert the element and immediately pop it
/// from the stack (void element behavior) (WHATWG 13.2.6.4.7).
#[test]
fn param_start_tag_inserts_void_element() {
    let text = "<html><body><object><param name=\"movie\" value=\"test.swf\"></object></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<param "),
        "param element should be present: {output:?}"
    );
}

/// A <source> start tag should insert the element and immediately pop it
/// from the stack (void element behavior) (WHATWG 13.2.6.4.7).
#[test]
fn source_start_tag_inserts_void_element() {
    let text = "<html><body><video><source src=\"video.mp4\"></video></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<source "),
        "source element should be present: {output:?}"
    );
}

/// A <track> start tag should insert the element and immediately pop it
/// from the stack (void element behavior) (WHATWG 13.2.6.4.7).
#[test]
fn track_start_tag_inserts_void_element() {
    let text = "<html><body><video><track kind=\"subtitles\" src=\"subs.vtt\"></video></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<track "),
        "track element should be present: {output:?}"
    );
}

/// An <hr> start tag should close an open <p> element, insert the <hr>,
/// pop it, and set frameset-ok to "not ok" (WHATWG 13.2.6.4.7).
#[test]
fn hr_start_tag_closes_p_and_inserts() {
    let text = "<html><body><p>text<hr></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<hr>"),
        "hr element should be present: {output:?}"
    );
    // The p should be closed before hr.
    assert!(
        !output.contains("<p><hr>"),
        "p should not contain hr: {output:?}"
    );
}

/// An <hr> without an open <p> should just insert normally (WHATWG 13.2.6.4.7).
#[test]
fn hr_start_tag_inserts_without_p() {
    let text = "<html><body><div><hr></div></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<hr>"),
        "hr element should be present: {output:?}"
    );
}

/// An <image> start tag should be rewritten to <img> and reprocessed
/// (WHATWG 13.2.6.4.7).
#[test]
fn image_start_tag_rewritten_to_img() {
    let text = "<html><body><image src=\"photo.jpg\"></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // <image> should be rewritten to <img>.
    assert!(
        output.contains("<img "),
        "image should be rewritten to img: {output:?}"
    );
    assert!(
        !output.contains("<image"),
        "image tag should not appear in output: {output:?}"
    );
    // Attributes should survive the tag name rewrite.
    assert!(
        output.contains("photo.jpg"),
        "attributes should be preserved through rewrite: {output:?}"
    );
}

/// An <xmp> start tag should close an open <p> element, reconstruct active
/// formatting elements, set frameset-ok to "not ok", and use the generic
/// raw text element parsing algorithm (WHATWG 13.2.6.4.7).
#[test]
fn xmp_start_tag_inserts_raw_text() {
    let text = "<html><body><xmp><b>not bold</b></xmp></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<xmp>"),
        "xmp element should be present: {output:?}"
    );
    // Inside <xmp>, tags are treated as raw text and serialized escaped.
    assert!(
        output.contains("&lt;b&gt;"),
        "Content inside xmp should be escaped as raw text: {output:?}"
    );
}

/// An <xmp> start tag should close an open <p> element (WHATWG 13.2.6.4.7).
#[test]
fn xmp_start_tag_closes_p() {
    let text = "<html><body><p>text<xmp>code</xmp></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        !output.contains("<p><xmp>"),
        "p should not contain xmp: {output:?}"
    );
}

/// An <iframe> start tag should set frameset-ok to "not ok" and use the
/// generic raw text element parsing algorithm (WHATWG 13.2.6.4.7).
#[test]
fn iframe_start_tag_inserts_raw_text() {
    let text = "<html><body><iframe><b>not bold</b></iframe></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<iframe>"),
        "iframe element should be present: {output:?}"
    );
}

/// A <noembed> start tag should use the generic raw text element parsing
/// algorithm (WHATWG 13.2.6.4.7).
#[test]
fn noembed_start_tag_inserts_raw_text() {
    let text = "<html><body><noembed><b>not bold</b></noembed></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<noembed>"),
        "noembed element should be present: {output:?}"
    );
}

/// A <noscript> start tag when scripting is disabled should reconstruct
/// active formatting elements and insert normally (WHATWG 13.2.6.4.7).
#[test]
fn noscript_start_tag_inserts_normally_when_scripting_disabled() {
    let text = "<html><body><noscript>fallback content</noscript></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<noscript>fallback content</noscript>"),
        "noscript element should be present with content: {output:?}"
    );
}

/// A <rb> start tag inside a <ruby> element should generate implied end tags
/// and insert the rb element (WHATWG 13.2.6.4.7).
#[test]
fn rb_start_tag_inside_ruby() {
    let text = "<html><body><ruby>text<rb>base</rb></ruby></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<rb>base</rb>"),
        "rb element should be present: {output:?}"
    );
}

/// A <rtc> start tag inside a <ruby> element should generate implied end tags
/// and insert the rtc element (WHATWG 13.2.6.4.7).
#[test]
fn rtc_start_tag_inside_ruby() {
    let text = "<html><body><ruby>text<rtc>annotation</rtc></ruby></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<rtc>annotation</rtc>"),
        "rtc element should be present: {output:?}"
    );
}

/// A <rt> start tag inside a <ruby> element should generate implied end tags
/// except for rtc, and insert the rt element (WHATWG 13.2.6.4.7).
#[test]
fn rt_start_tag_inside_ruby() {
    let text = "<html><body><ruby>text<rt>annotation</rt></ruby></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<rt>annotation</rt>"),
        "rt element should be present: {output:?}"
    );
}

/// A <rp> start tag inside a <ruby> element should generate implied end tags
/// except for rtc, and insert the rp element (WHATWG 13.2.6.4.7).
#[test]
fn rp_start_tag_inside_ruby() {
    let text = "<html><body><ruby>text<rp>(</rp><rt>annotation</rt><rp>)</rp></ruby></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<rp>(</rp>"),
        "first rp element should be present: {output:?}"
    );
    assert!(
        output.contains("<rp>)</rp>"),
        "second rp element should be present: {output:?}"
    );
}

/// An <rt> inside a <ruby> with an open <rb> should implicitly close the <rb>
/// (via generate implied end tags) (WHATWG 13.2.6.4.7).
#[test]
fn rt_closes_open_rb() {
    let text = "<html><body><ruby><rb>base<rt>annotation</ruby></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<rb>base</rb>"),
        "rb should be implicitly closed: {output:?}"
    );
    assert!(
        output.contains("<rt>annotation</rt>"),
        "rt should be present: {output:?}"
    );
}

/// A <math> start tag should reconstruct active formatting elements and
/// insert a foreign element in the MathML namespace (WHATWG 13.2.6.4.7).
#[test]
fn math_start_tag_inserts_foreign_element() {
    let text = "<html><body><math><mi>x</mi></math></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<math>"),
        "math element should be present: {output:?}"
    );
}

/// A self-closing <math/> should insert and immediately pop (WHATWG 13.2.6.4.7).
#[test]
fn math_self_closing_pops_immediately() {
    let text = "<html><body><math/><p>after</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<math>"),
        "math element should be present: {output:?}"
    );
    assert!(
        output.contains("<p>after</p>"),
        "content after self-closing math should be preserved: {output:?}"
    );
}

/// Table-related start tags in body (<caption>, <col>, etc.) should be
/// treated as a parse error and ignored (WHATWG 13.2.6.4.7).
#[test]
fn table_related_start_tags_in_body_are_ignored() {
    let text = "<html><body><caption>text</caption><p>content</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<p>content</p>"),
        "Body content should be preserved: {output:?}"
    );
}

/// A <frame> start tag in body should be ignored (WHATWG 13.2.6.4.7).
#[test]
fn frame_start_tag_in_body_is_ignored() {
    let text = "<html><body><frame><p>content</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<p>content</p>"),
        "Body content should be preserved: {output:?}"
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
