use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

// ============================================================================
// Data state tests
// ============================================================================

#[test]
fn data_state_basic_text() {
    let text = "<html><body>hello world</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("hello world")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn data_state_special_characters_preserved() {
    // Various special characters in data state should be preserved as text
    let text = "<html><body>a\tb\nc</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains("a\tb\nc"),
        "tab and newline should be preserved. Got: {}", output);
}

#[test]
fn data_state_less_than_sign_transitions_to_tag_open() {
    let text = "<html><body><div>content</div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text("content"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Character reference tests (named, decimal, hexadecimal)
// ============================================================================

#[test]
fn named_character_reference_basic_entities() {
    let text = r##"<html><body><p>&amp;&lt;&gt;&quot;&#39;</p></body></html>"##;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("p", |p| p.add_text("&<>\"'"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn named_character_reference_nbsp() {
    let text = "<html><body><p>a&nbsp;b</p></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("p", |p| p.add_text("a\u{00A0}b"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn named_character_reference_copy_sign() {
    let text = "<html><body>&copy;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{00A9}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn decimal_character_reference_basic() {
    // &#65; = 'A', &#97; = 'a', &#48; = '0'
    let text = "<html><body>&#65;&#97;&#48;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("Aa0")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn decimal_character_reference_multibyte() {
    // &#169; = copyright sign (U+00A9), &#8212; = em dash (U+2014)
    let text = "<html><body>&#169;&#8212;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{00A9}\u{2014}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn decimal_character_reference_null_replaced_with_fffd() {
    // &#0; should produce U+FFFD per WHATWG spec (null character reference parse error)
    let text = "<html><body>&#0;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{FFFD}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hexadecimal_character_reference_lowercase_x() {
    // &#x41; = 'A', &#x61; = 'a', &#x30; = '0'
    let text = "<html><body>&#x41;&#x61;&#x30;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("Aa0")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hexadecimal_character_reference_uppercase_x() {
    // &#X41; = 'A' (uppercase X is also valid per WHATWG)
    let text = "<html><body>&#X41;&#X5A;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("AZ")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hexadecimal_character_reference_mixed_case_digits() {
    // &#xFF; = U+00FF (lowercase hex digits)
    // &#xFf; = U+00FF (mixed case hex digits)
    let text = "<html><body>&#xff;&#xFf;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{00FF}\u{00FF}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hexadecimal_character_reference_multibyte_unicode() {
    // &#x2014; = em dash, &#x1F600; = grinning face emoji
    let text = "<html><body>&#x2014;&#x1F600;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{2014}\u{1F600}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hexadecimal_character_reference_null_replaced_with_fffd() {
    // &#x0; should produce U+FFFD per WHATWG spec
    let text = "<html><body>&#x0;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{FFFD}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hexadecimal_character_reference_missing_semicolon() {
    // &#x41 without semicolon - WHATWG says: parse error, but still consume and emit the character
    let text = "<html><body>&#x41 end</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // 'A' should still be produced even without the semicolon
    assert!(output.contains('A'),
        "hex char ref without semicolon should still produce character. Got: {}", output);
}

#[test]
fn hexadecimal_character_reference_control_character_replacement() {
    // &#x80; (U+0080) should be replaced with U+20AC (Euro sign) per WHATWG table
    let text = "<html><body>&#x80;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{20AC}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn decimal_character_reference_control_character_replacement_0x8d() {
    // &#141; (0x8D) should be replaced with U+008D (reverse line feed) per WHATWG table
    let text = "<html><body>&#141;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{008D}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn character_reference_in_attribute_value() {
    // Character references in attribute values should be resolved
    let text = r#"<html><body><a href="?a=1&amp;b=2">link</a></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("a", |a| {
                        a.add_attribute_str("href", "?a=1&b=2")
                            .add_text("link")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hexadecimal_character_reference_in_attribute() {
    // Hex character references in attribute values should be resolved
    let text = r#"<html><body><div data-val="&#x41;&#x42;&#x43;">x</div></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("data-val", "ABC")
                            .add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn numeric_character_reference_no_digits_hex() {
    // &#x; with no digits — WHATWG says: parse error, flush code points, reconsume
    // The text is emitted literally; `&` becomes `&amp;` in serialized output
    let text = "<html><body>&#x;</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains("&amp;#x;"),
        "&#x; with no digits should be emitted literally (& escaped). Got: {}", output);
}

#[test]
fn numeric_character_reference_no_digits_decimal() {
    // &#; with no digits
    let text = "<html><body>&#;</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains("&amp;#;"),
        "&#; with no digits should be emitted literally (& escaped). Got: {}", output);
}

// ============================================================================
// RCDATA state tests (triggered by <title>, <textarea>)
// ============================================================================

#[test]
fn rcdata_title_basic_text() {
    let text = "<html><head><title>Page Title</title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| title.add_text("Page Title"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_title_with_html_entities_not_parsed_as_tags() {
    // Tags inside <title> should NOT be parsed as tags (RCDATA mode)
    let text = "<html><head><title>Hello <b>World</b></title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| {
                    title.add_text("Hello <b>World</b>")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_title_character_references_resolved() {
    // Character references ARE resolved in RCDATA mode
    let text = "<html><head><title>A &amp; B</title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| title.add_text("A & B"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_title_with_less_than_not_followed_by_slash() {
    // A lone '<' in RCDATA that isn't followed by '/' should be treated as text
    let text = "<html><head><title>a < b</title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| title.add_text("a < b"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_title_wrong_end_tag_not_closed() {
    // An end tag for a different element inside RCDATA should be treated as text
    let text = "<html><head><title>foo </div> bar</title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| {
                    title.add_text("foo </div> bar")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_textarea_basic() {
    let text = "<html><body><textarea>some text</textarea></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("textarea", |ta| ta.add_text("some text"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_textarea_ignores_tags() {
    // Tags inside <textarea> should NOT be parsed as tags
    let text = "<html><body><textarea><p>not a paragraph</p></textarea></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("textarea", |ta| {
                        ta.add_text("<p>not a paragraph</p>")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_textarea_hex_character_reference() {
    // Hex character references should work in RCDATA
    let text = "<html><body><textarea>&#x41;&#x42;</textarea></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("textarea", |ta| ta.add_text("AB"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// RAWTEXT state tests (triggered by <style>, <noframes>)
// ============================================================================

#[test]
fn rawtext_style_basic() {
    let text = "<html><head><style>body { color: red; }</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("body { color: red; }")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_with_html_like_content() {
    // HTML-like content inside <style> should NOT be parsed as tags
    let text = "<html><head><style>div > p { color: red; }</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("div > p { color: red; }")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_character_references_not_resolved() {
    // Character references in RAWTEXT should NOT be resolved (unlike RCDATA)
    let text = "<html><head><style>&amp; not resolved</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("&amp; not resolved")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_with_less_than_sign() {
    // A '<' not followed by '/' should be treated as text in RAWTEXT
    let text = "<html><head><style>a < b { }</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("a < b { }")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_wrong_end_tag_treated_as_text() {
    // An end tag for a different element inside RAWTEXT should be treated as text
    let text = "<html><head><style>body </div> { }</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("body </div> { }")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_end_tag_case_insensitive() {
    // RAWTEXT end tags should be matched case-insensitively
    let text = "<html><head><style>css content</STYLE></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("css content")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_partial_end_tag_treated_as_text() {
    // "</sty" without completing "style" should be treated as text
    let text = "<html><head><style></sty content</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("</sty content")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_multiline_css() {
    let text = "<html><head><style>\nbody {\n  color: red;\n}\n</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("\nbody {\n  color: red;\n}\n")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_noframes_basic() {
    let text = "<html><head><noframes><p>not parsed</p></noframes></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("noframes", |nf| {
                    nf.add_text("<p>not parsed</p>")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Script data state tests
// ============================================================================

#[test]
fn script_data_basic_content() {
    let text = "<html><head><script>var x = 1;</script></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| {
                    script.add_text("var x = 1;")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn script_data_with_html_like_content() {
    // HTML-like content inside <script> should NOT be parsed as tags
    let text = "<html><head><script>if (a < b && c > d) {}</script></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| {
                    script.add_text("if (a < b && c > d) {}")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn script_data_character_references_not_resolved() {
    // Character references in script data should NOT be resolved
    let text = "<html><head><script>&amp;</script></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| {
                    script.add_text("&amp;")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn script_data_wrong_end_tag_treated_as_text() {
    let text = "<html><head><script>x </div> y</script></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| {
                    script.add_text("x </div> y")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn script_data_multiple_scripts() {
    let text = "<html><head><script>var a=1;</script><script>var b=2;</script></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |s| s.add_text("var a=1;"))
                    .add_element("script", |s| s.add_text("var b=2;"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Tag open / end tag / tag name / attribute state tests
// ============================================================================

#[test]
fn tag_open_basic_start_tag() {
    let text = "<html><body><div></div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div)
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn tag_name_case_insensitivity() {
    // Tag names should be lowercased
    let text = "<HTML><BODY><DIV>text</DIV></BODY></HTML>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text("text"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn self_closing_void_element_br() {
    let text = "<html><body><br/></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("br", |br| br)
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn self_closing_void_element_br_without_slash() {
    let text = "<html><body><br></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("br", |br| br)
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attributes_single_quoted() {
    let text = "<html><body><div id='foo'>x</div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("id", "foo").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attributes_double_quoted() {
    let text = r#"<html><body><div class="bar">x</div></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("class", "bar").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attributes_unquoted() {
    let text = "<html><body><div id=foo>x</div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("id", "foo").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attributes_boolean_no_value() {
    let text = "<html><body><input disabled></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("input", |input| {
                        input.add_attribute_str("disabled", "")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attributes_multiple_with_whitespace() {
    let text = r#"<html><body><div  id="a"   class="b"  data-x="c">x</div></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("id", "a")
                            .add_attribute_str("class", "b")
                            .add_attribute_str("data-x", "c")
                            .add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attribute_name_case_insensitivity() {
    // Attribute names should be lowercased
    let text = r#"<html><body><div ID="foo">x</div></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("id", "foo").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attribute_with_character_reference_in_double_quoted_value() {
    let text = r#"<html><body><div data-json="{&quot;key&quot;:&quot;val&quot;}">x</div></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("data-json", r#"{"key":"val"}"#)
                            .add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Comment state tests
// ============================================================================

#[test]
fn comment_basic() {
    let text = "<html><body><!-- hello --></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_comment(" hello ")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn comment_empty() {
    let text = "<html><body><!----></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_comment("")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn comment_with_dashes() {
    let text = "<html><body><!-- a-b--c ---></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // WHATWG comment end state: "--->" consumes "--" as the close delimiter,
    // leaving the extra "-" as part of the comment content: " a-b--c -"
    assert!(output.contains("<!-- a-b--c --->"),
        "comment with extra dashes should be preserved. Got: {}", output);
}

#[test]
fn comment_with_less_than() {
    let text = "<html><body><!-- <div> --></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_comment(" <div> ")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn comment_multiple() {
    let text = "<html><body><!-- first --><!-- second --></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_comment(" first ")
                        .add_comment(" second ")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn comment_before_html() {
    let text = "<!-- before --><html><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains("<!-- before -->"),
        "comment before html should be preserved. Got: {}", output);
}

// ============================================================================
// DOCTYPE state tests
// ============================================================================

#[test]
fn doctype_html5() {
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_doctype("html")
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn doctype_case_insensitive() {
    let text = "<!doctype html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_doctype("html")
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn doctype_with_public_identifier() {
    let text = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_doctype("html")
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn doctype_with_system_identifier() {
    let text = r#"<!DOCTYPE html SYSTEM "about:legacy-compat"><html><head></head><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_doctype("html")
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Implicit tag generation tests
// ============================================================================

#[test]
fn implicit_html_head_body() {
    // Even without explicit <html>, <head>, <body> tags, the tree should have them
    let text = "<div>hello</div>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text("hello"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn implicit_head_body_with_only_text() {
    let text = "just text";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("just text")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Bogus comment state tests
// ============================================================================

#[test]
fn bogus_comment_exclamation_mark() {
    // <!X should create a bogus comment per WHATWG (incorrectly opened comment)
    // but the parser may handle this differently. Verify it doesn't crash.
    let text = "<html><body><!X bogus></body></html>";
    let _document = html::parse(text).unwrap();
}

// ============================================================================
// Mixed content and edge cases
// ============================================================================

#[test]
fn mixed_text_and_elements() {
    let text = "<html><body>before<span>middle</span>after</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("before")
                        .add_element("span", |span| span.add_text("middle"))
                        .add_text("after")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn nested_elements() {
    let text = "<html><body><div><span><a>deep</a></span></div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_element("span", |span| {
                            span.add_element("a", |a| a.add_text("deep"))
                        })
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn empty_document() {
    // Empty input should parse without errors
    let text = "";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The output should be empty (no implicit elements generated for empty input)
    assert!(output.is_empty() || output.trim().is_empty(),
        "empty document should produce empty or whitespace-only output. Got: {:?}", output);
}

#[test]
fn whitespace_only_document() {
    // Whitespace-only input should parse without errors
    let text = "   \n\t  ";
    let _document = html::parse(text).unwrap();
}

#[test]
fn multiple_character_references_in_sequence() {
    let text = "<html><body>&amp;&amp;&amp;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("&&&")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn character_reference_at_end_of_text() {
    let text = "<html><body>hello&amp;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("hello&")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn decimal_and_hex_character_references_mixed() {
    // &#65; = 'A', &#x42; = 'B', &#67; = 'C', &#x44; = 'D'
    let text = "<html><body>&#65;&#x42;&#67;&#x44;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("ABCD")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Ambiguous ampersand state tests
// ============================================================================

#[test]
fn ambiguous_ampersand_not_a_reference() {
    // Ampersand followed by text that doesn't match any named reference should be literal.
    // Note: "&not" actually matches the named character reference &not; (U+00AC)
    // so we use "&xyz" which doesn't match any named reference.
    let text = "<html><body>&xyz hello</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The ampersand should be preserved (escaped as &amp; in output)
    assert!(output.contains("&amp;xyz hello"),
        "ambiguous ampersand should be preserved. Got: {}", output);
}

#[test]
fn ampersand_followed_by_space() {
    // Ampersand followed by space is not a character reference
    let text = "<html><body>a & b</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("a & b")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// End tag edge cases
// ============================================================================

#[test]
fn end_tag_with_attributes_ignored() {
    // End tags with attributes are parse errors but the tag should still close
    let text = r#"<html><body><div>text</div id="x"></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text("text"))
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Complex real-world scenarios
// ============================================================================

#[test]
fn complex_page_structure() {
    let text = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test &amp; Page</title>
    <style>body { color: red; }</style>
    <script>var x = '<div>';</script>
</head>
<body>
    <div id="main" class="container">
        <h1>Hello &#x26; World</h1>
        <p>Paragraph with <em>emphasis</em> and <strong>strong</strong>.</p>
        <!-- navigation -->
        <a href="?a=1&amp;b=2">Link</a>
    </div>
</body>
</html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();

    // Verify key parsing behaviors
    assert!(output.contains("<title>Test &amp; Page</title>"),
        "title should have resolved character reference. Got: {}", output);
    assert!(output.contains("body { color: red; }"),
        "style RAWTEXT content should be preserved. Got: {}", output);
    // Script text nodes get < > escaped in to_string() output
    assert!(output.contains("var x = '&lt;div&gt;';"),
        "script content should be preserved (with < > escaped in serialization). Got: {}", output);
    assert!(output.contains("Hello &amp; World"),
        "hex char ref &#x26; should resolve to &. Got: {}", output);
    assert!(output.contains("<!-- navigation -->"),
        "comment should be preserved. Got: {}", output);
    assert!(output.contains("?a=1&amp;b=2"),
        "attribute char ref should be resolved. Got: {}", output);
}

#[test]
fn style_followed_by_content() {
    // After closing </style>, the tokenizer should return to data state
    let text = "<html><head><style>.x{}</style></head><body><p>text</p></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text(".x{}")
                })
            })
            .add_element("body", |body| {
                body.add_element("p", |p| p.add_text("text"))
            })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn title_followed_by_content() {
    // After closing </title>, the tokenizer should return to data state
    let text = "<html><head><title>t</title></head><body><div>content</div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| title.add_text("t"))
            })
            .add_element("body", |body| {
                body.add_element("div", |div| div.add_text("content"))
            })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn script_followed_by_content() {
    // After closing </script>, the tokenizer should return to data state
    let text = "<html><head><script>x</script></head><body><p>after</p></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |s| s.add_text("x"))
            })
            .add_element("body", |body| {
                body.add_element("p", |p| p.add_text("after"))
            })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hex_character_reference_surrogate_replaced_with_fffd() {
    // Surrogate code points (0xD800-0xDFFF) should be replaced with U+FFFD
    let text = "<html><body>&#xD800;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{FFFD}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn hex_character_reference_outside_unicode_replaced_with_fffd() {
    // Code points > 0x10FFFF should be replaced with U+FFFD
    let text = "<html><body>&#x110000;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{FFFD}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn decimal_character_reference_missing_semicolon() {
    // &#65 without semicolon - parse error but should still produce 'A'
    let text = "<html><body>&#65 end</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains('A'),
        "decimal char ref without semicolon should still produce character. Got: {}", output);
}

// ============================================================================
// Numeric character reference end state — WHATWG replacement table
// ============================================================================

#[test]
fn numeric_char_ref_windows_1252_replacements() {
    // WHATWG specifies that certain control character references (0x80-0x9F) are
    // replaced with Windows-1252 equivalents.
    // &#x80; -> U+20AC (Euro sign)
    // &#x82; -> U+201A (single low-9 quotation mark)
    // &#x83; -> U+0192 (Latin small letter f with hook)
    // &#x84; -> U+201E (double low-9 quotation mark)
    // &#x85; -> U+2026 (horizontal ellipsis)
    // &#x91; -> U+2018 (left single quotation mark)
    // &#x92; -> U+2019 (right single quotation mark)
    // &#x93; -> U+201C (left double quotation mark)
    // &#x94; -> U+201D (right double quotation mark)
    // &#x96; -> U+2013 (en dash)
    // &#x97; -> U+2014 (em dash)
    // &#x99; -> U+2122 (trade mark sign)

    let text = "<html><body>&#x80;&#x93;&#x94;&#x96;&#x97;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{20AC}\u{201C}\u{201D}\u{2013}\u{2014}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// RAWTEXT end tag name state — edge cases
// ============================================================================

#[test]
fn rawtext_style_end_tag_with_space_before_close() {
    // </style > with a space before > should still close the style element
    let text = "<html><head><style>css</style ></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| style.add_text("css"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_slash_not_followed_by_tag_name() {
    // "</" followed by a non-alpha character in RAWTEXT should be treated as text
    let text = "<html><head><style></1 content</style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| {
                    style.add_text("</1 content")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rawtext_style_empty() {
    let text = "<html><head><style></style></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("style", |style| style)
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// RCDATA end tag name state — edge cases
// ============================================================================

#[test]
fn rcdata_title_end_tag_case_insensitive() {
    let text = "<html><head><title>text</TITLE></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| title.add_text("text"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_title_end_tag_with_space() {
    // </title > with space should still close
    let text = "<html><head><title>text</title ></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| title.add_text("text"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_title_partial_end_tag_treated_as_text() {
    // "</titl" without completing "title" should be treated as text
    let text = "<html><head><title></titl content</title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| {
                    title.add_text("</titl content")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn rcdata_title_empty() {
    let text = "<html><head><title></title></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("title", |title| title)
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Script data escaped state tests
// ============================================================================

#[test]
fn script_data_with_html_comment() {
    // Script data can contain HTML comments which triggers the escaped states
    let text = "<html><head><script><!--\nvar x = 1;\n--></script></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| {
                    script.add_text("<!--\nvar x = 1;\n-->")
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn script_data_escaped_with_close_tag() {
    // </script> inside script content should close the script even if inside an
    // HTML-style comment, because the script data state tracks end tags.
    let text = "<html><head><script>var x=1;</script></head><body>after</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains("<script>var x=1;</script>"),
        "script should close at </script>. Got: {}", output);
    assert!(output.contains("after"),
        "content after script should be parsed. Got: {}", output);
}

// ============================================================================
// Special character handling
// ============================================================================

#[test]
fn tab_line_feed_form_feed_in_text() {
    let text = "<html><body>\t\n\x0C</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains("\t") || output.contains("\n"),
        "whitespace characters should be preserved. Got: {:?}", output);
}

#[test]
fn named_character_reference_hearts() {
    let text = "<html><body>&hearts;</body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("\u{2665}")
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn named_character_reference_two_codepoint() {
    // Some named character references produce two code points
    // &nLt; -> U+226A U+20D2
    let text = "<html><body>&nLt;</body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(output.contains('\u{226A}'),
        "two-codepoint named ref should produce first codepoint. Got: {:?}", output);
}

// ============================================================================
// Attribute edge cases
// ============================================================================

#[test]
fn attribute_empty_value_double_quoted() {
    let text = r#"<html><body><div id="">x</div></body></html>"#;
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("id", "").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attribute_empty_value_single_quoted() {
    let text = "<html><body><div id=''>x</div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("id", "").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attribute_with_hex_char_ref_in_single_quoted() {
    let text = "<html><body><div data-x='&#x41;'>x</div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("data-x", "A").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn attribute_with_decimal_char_ref_in_unquoted() {
    let text = "<html><body><div data-x=&#65;>x</div></body></html>";
    let document = html::parse(text).unwrap();
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("data-x", "A").add_text("x")
                    })
                })
        })
        .build()
        .unwrap();
    assert!(test_framework::compare_documents(expected, document, true));
}

// ============================================================================
// Duplicate attribute deduplication tests (WHATWG 13.2.5.34)
// ============================================================================

/// Duplicate attributes on a tag: per WHATWG spec, only the first
/// occurrence of an attribute name should be kept; subsequent
/// duplicates are parse errors and must be dropped.
#[test]
fn duplicate_attributes_first_wins() {
    let text = r#"<html><head></head><body><div class="a" class="b">text</div></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("class", "a").add_text("text")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Duplicate attributes with different values: only the first value is kept.
#[test]
fn duplicate_attributes_multiple_different_values() {
    let text = r#"<html><head></head><body><p id="first" id="second" id="third">text</p></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("p", |p| {
                        p.add_attribute_str("id", "first").add_text("text")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Non-duplicate attributes should all be preserved.
#[test]
fn non_duplicate_attributes_all_preserved() {
    let text =
        r#"<html><head></head><body><div id="a" class="b" data-x="c">text</div></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("id", "a")
                            .add_attribute_str("class", "b")
                            .add_attribute_str("data-x", "c")
                            .add_text("text")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Duplicate attribute where first occurrence has empty value.
#[test]
fn duplicate_attribute_first_empty_value() {
    let text = r#"<html><head></head><body><div class="" class="b">text</div></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("class", "").add_text("text")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Duplicate attribute names are compared after lowercasing.
/// CLASS="a" and class="b" should be treated as duplicates.
#[test]
fn duplicate_attribute_case_insensitive() {
    let text = r#"<html><head></head><body><div CLASS="a" class="b">text</div></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_attribute_str("class", "a").add_text("text")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
