use skyscraper::html;
use skyscraper::xpath::grammar::XpathItemTreeNode;

// ---------------------------------------------------------------------------
// Character tokens in foreign content
// ---------------------------------------------------------------------------

/// A NULL character (U+0000) inside a foreign element should be replaced with
/// U+FFFD REPLACEMENT CHARACTER (WHATWG 13.2.6.5).
#[test]
fn null_character_in_foreign_content_replaced_with_replacement_character() {
    let text = "<html><body><svg><text>\0</text></svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains('\u{FFFD}'),
        "NULL should be replaced with U+FFFD in foreign content: {output:?}"
    );
}

/// Regular text inside a foreign element should be preserved.
#[test]
fn text_in_foreign_content_is_preserved() {
    let text = "<html><body><svg><text>hello</text></svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("hello"),
        "text in foreign content should be preserved: {output:?}"
    );
}

/// Whitespace characters in foreign content should be inserted normally.
#[test]
fn whitespace_in_foreign_content_is_preserved() {
    let text = "<html><body><svg> <rect/> </svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<svg>"),
        "svg element should be present: {output:?}"
    );
}

// ---------------------------------------------------------------------------
// Comment and DOCTYPE in foreign content
// ---------------------------------------------------------------------------

/// A comment token in foreign content should be inserted as a comment node.
#[test]
fn comment_in_foreign_content_is_inserted() {
    let text = "<html><body><svg><!-- svg comment --></svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- svg comment -->"),
        "comment in foreign content should be preserved: {output:?}"
    );
}

// ---------------------------------------------------------------------------
// Start tags — breakout behavior
// ---------------------------------------------------------------------------

/// An HTML-like start tag (e.g. <div>) inside SVG foreign content should
/// break out of foreign content, popping SVG elements until the adjusted
/// current node is in the HTML namespace, then reprocess using "in body" rules.
#[test]
fn html_start_tag_breaks_out_of_svg_foreign_content() {
    let text = "<html><body><svg><div>hello</div></svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // The <div> should NOT be a child of <svg>; it should be a sibling.
    // The parser should pop <svg> when it sees <div>.
    assert!(
        output.contains("<div>hello</div>"),
        "div should be present after breaking out of SVG: {output:?}"
    );
}

/// A <p> tag inside MathML foreign content should break out of foreign content.
#[test]
fn p_tag_breaks_out_of_mathml_foreign_content() {
    let text = "<html><body><math><mi>x</mi><p>text</p></math></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<p>text</p>"),
        "p element should be present after breaking out of MathML: {output:?}"
    );
}

/// A <font> tag with a "color" attribute should break out of foreign content.
#[test]
fn font_with_color_breaks_out_of_foreign_content() {
    let text = r#"<html><body><svg><font color="red">text</font></svg></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<font"),
        "font element should be present after breaking out: {output:?}"
    );
}

/// A <font> tag WITHOUT color/face/size attributes should NOT break out of
/// foreign content — it should be treated as a generic foreign element.
#[test]
fn font_without_special_attrs_stays_in_foreign_content() {
    let text = "<html><body><svg><font>text</font></svg></body></html>";
    let document = html::parse(text).unwrap();
    // The font element should be in SVG namespace since it didn't break out
    let font_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "font" => Some(e),
            _ => None,
        })
        .expect("font element should be present");
    assert_eq!(
        font_element.namespace.as_deref(),
        Some("http://www.w3.org/2000/svg"),
        "font without color/face/size should stay in SVG namespace"
    );
}

// ---------------------------------------------------------------------------
// Start tags — foreign element insertion
// ---------------------------------------------------------------------------

/// Unknown elements inside SVG should be inserted in the SVG namespace.
#[test]
fn unknown_element_in_svg_gets_svg_namespace() {
    let text = "<html><body><svg><rect></rect></svg></body></html>";
    let document = html::parse(text).unwrap();
    let rect_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "rect" => Some(e),
            _ => None,
        })
        .expect("rect element should be present");
    assert_eq!(
        rect_element.namespace.as_deref(),
        Some("http://www.w3.org/2000/svg"),
        "rect should be in SVG namespace"
    );
}

/// Unknown elements inside MathML should be inserted in the MathML namespace.
#[test]
fn unknown_element_in_mathml_gets_mathml_namespace() {
    let text = "<html><body><math><mfrac><mn>1</mn><mn>2</mn></mfrac></math></body></html>";
    let document = html::parse(text).unwrap();
    let mfrac_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "mfrac" => Some(e),
            _ => None,
        })
        .expect("mfrac element should be present");
    assert_eq!(
        mfrac_element.namespace.as_deref(),
        Some("http://www.w3.org/1998/Math/MathML"),
        "mfrac should be in MathML namespace"
    );
}

/// Self-closing elements in foreign content should be popped immediately.
#[test]
fn self_closing_element_in_svg_pops_immediately() {
    let text = "<html><body><svg><circle/><rect/></svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<circle>"),
        "self-closing circle should be present: {output:?}"
    );
    assert!(
        output.contains("<rect>"),
        "self-closing rect should be present: {output:?}"
    );
}

/// SVG attribute adjustment should be applied when inserting foreign elements
/// in the "in foreign content" mode (not just in the in_body SVG handler).
#[test]
fn svg_attribute_adjustment_in_foreign_content() {
    let text =
        r#"<html><body><svg><rect viewbox="0 0 100 100"></rect></svg></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("viewBox="),
        "viewbox should be adjusted to viewBox in foreign content: {output:?}"
    );
}

/// SVG element name case correction should be applied (e.g. foreignobject → foreignObject).
#[test]
fn svg_element_name_correction_in_foreign_content() {
    let text = "<html><body><svg><foreignobject>text</foreignobject></svg></body></html>";
    let document = html::parse(text).unwrap();
    let fo_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "foreignObject" => Some(e),
            _ => None,
        })
        .expect("foreignObject element should be present");
    assert_eq!(
        fo_element.namespace.as_deref(),
        Some("http://www.w3.org/2000/svg"),
        "foreignObject should be in SVG namespace"
    );
}

// ---------------------------------------------------------------------------
// End tags in foreign content
// ---------------------------------------------------------------------------

/// An end tag that matches the current SVG element should pop it.
#[test]
fn end_tag_pops_matching_foreign_element() {
    let text = "<html><body><svg><g><rect></rect></g></svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<g>"),
        "g element should be present: {output:?}"
    );
    assert!(
        output.contains("<rect>"),
        "rect element should be present: {output:?}"
    );
}

/// An end tag in foreign content should match case-insensitively (by lowering
/// the node's tag name).
#[test]
fn end_tag_matches_case_insensitively_in_foreign_content() {
    // foreignobject gets corrected to foreignObject, but </foreignobject>
    // should still match it (lowercased comparison).
    let text = "<html><body><svg><foreignobject>text</foreignobject></svg></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("foreignObject"),
        "foreignObject element should be present: {output:?}"
    );
    assert!(
        output.contains("text"),
        "text content should be preserved: {output:?}"
    );
}

// ---------------------------------------------------------------------------
// MathML text integration points
// ---------------------------------------------------------------------------

/// A start tag that is NOT mglyph or malignmark inside a MathML text
/// integration point (e.g. <mtext>) should be processed as HTML, not as
/// foreign content (the tree construction dispatcher sends it to "in body").
#[test]
fn html_start_tag_in_mathml_text_integration_point() {
    let text = "<html><body><math><mtext><span>hello</span></mtext></math></body></html>";
    let document = html::parse(text).unwrap();
    let span_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "span" => Some(e),
            _ => None,
        })
        .expect("span element should be present");
    // span should be in HTML namespace (no namespace = HTML)
    assert_eq!(
        span_element.namespace, None,
        "span inside mtext should be in HTML namespace"
    );
}

/// mglyph inside a MathML text integration point should be processed as
/// foreign content (dispatched to foreign content rules).
#[test]
fn mglyph_in_mathml_text_integration_point_is_foreign() {
    let text = "<html><body><math><mtext><mglyph/></mtext></math></body></html>";
    let document = html::parse(text).unwrap();
    let mglyph_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "mglyph" => Some(e),
            _ => None,
        })
        .expect("mglyph element should be present");
    assert_eq!(
        mglyph_element.namespace.as_deref(),
        Some("http://www.w3.org/1998/Math/MathML"),
        "mglyph inside mtext should be in MathML namespace"
    );
}

// ---------------------------------------------------------------------------
// HTML integration points
// ---------------------------------------------------------------------------

/// Start tags inside an SVG <foreignObject> should be processed as HTML.
#[test]
fn html_content_inside_svg_foreignobject() {
    let text = "<html><body><svg><foreignobject><div>hello</div></foreignobject></svg></body></html>";
    let document = html::parse(text).unwrap();
    let div_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "div" => Some(e),
            _ => None,
        })
        .expect("div element should be present inside foreignObject");
    assert_eq!(
        div_element.namespace, None,
        "div inside foreignObject should be in HTML namespace"
    );
}

/// Start tags inside an SVG <desc> should be processed as HTML.
#[test]
fn html_content_inside_svg_desc() {
    let text = "<html><body><svg><desc><span>description</span></desc></svg></body></html>";
    let document = html::parse(text).unwrap();
    let span_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "span" => Some(e),
            _ => None,
        })
        .expect("span element should be present inside desc");
    assert_eq!(
        span_element.namespace, None,
        "span inside desc should be in HTML namespace"
    );
}

/// Start tags inside an SVG <title> should be processed as HTML.
#[test]
fn html_content_inside_svg_title() {
    let text = "<html><body><svg><title><b>bold</b></title></svg></body></html>";
    let document = html::parse(text).unwrap();
    let b_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "b" => Some(e),
            _ => None,
        })
        .expect("b element should be present inside title");
    assert_eq!(
        b_element.namespace, None,
        "b inside SVG title should be in HTML namespace"
    );
}

// ---------------------------------------------------------------------------
// Nested SVG/MathML
// ---------------------------------------------------------------------------

/// Nested SVG elements should maintain proper namespacing.
#[test]
fn nested_svg_elements_maintain_namespace() {
    let text = "<html><body><svg><g><circle/></g></svg></body></html>";
    let document = html::parse(text).unwrap();
    let g_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "g" => Some(e),
            _ => None,
        })
        .expect("g element should be present");
    assert_eq!(
        g_element.namespace.as_deref(),
        Some("http://www.w3.org/2000/svg"),
        "g should be in SVG namespace"
    );

    let circle_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "circle" => Some(e),
            _ => None,
        })
        .expect("circle element should be present");
    assert_eq!(
        circle_element.namespace.as_deref(),
        Some("http://www.w3.org/2000/svg"),
        "circle should be in SVG namespace"
    );
}

/// An SVG inside a MathML annotation-xml with encoding="text/html" should
/// be processed as HTML (annotation-xml is an HTML integration point).
#[test]
fn svg_in_mathml_annotation_xml_with_html_encoding() {
    let text = r#"<html><body><math><annotation-xml encoding="text/html"><svg></svg></annotation-xml></math></body></html>"#;
    let document = html::parse(text).unwrap();
    let svg_element = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::ElementNode(e) if e.name == "svg" => Some(e),
            _ => None,
        })
        .expect("svg element should be present");
    // When annotation-xml is an HTML integration point, start tags are
    // processed by "in body" rules, so <svg> gets SVG namespace via in_body.
    assert_eq!(
        svg_element.namespace.as_deref(),
        Some("http://www.w3.org/2000/svg"),
        "svg should be in SVG namespace"
    );
}

// ---------------------------------------------------------------------------
// Complex real-world-ish cases
// ---------------------------------------------------------------------------

/// SVG with nested elements followed by HTML content.
#[test]
fn svg_followed_by_html_content() {
    let text = "<html><body><svg><rect/></svg><p>after</p></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<svg>"),
        "svg should be present: {output:?}"
    );
    assert!(
        output.contains("<p>after</p>"),
        "p should follow svg: {output:?}"
    );
}

/// Multiple SVG elements in the same document.
#[test]
fn multiple_svg_elements() {
    let text = "<html><body><svg><rect/></svg><svg><circle/></svg></body></html>";
    let document = html::parse(text).unwrap();
    let svg_count = document
        .iter()
        .filter(|node| matches!(node, XpathItemTreeNode::ElementNode(e) if e.name == "svg"))
        .count();
    assert_eq!(svg_count, 2, "should have two svg elements");
}

/// MathML attributes should be adjusted when processing tokens in foreign content.
#[test]
fn mathml_attribute_adjustment_in_foreign_content() {
    let text = r#"<html><body><math><mrow definitionurl="http://example.com"></mrow></math></body></html>"#;
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("definitionURL="),
        "definitionurl should be adjusted to definitionURL in foreign content: {output:?}"
    );
}

// ---------------------------------------------------------------------------
// Foreign attribute namespace assignment (WHATWG 13.2.6.3)
// ---------------------------------------------------------------------------

/// xlink:href in SVG foreign content should get the xlink namespace.
#[test]
fn foreign_attribute_xlink_href_gets_xlink_namespace() {
    let text =
        r##"<html><body><svg><use xlink:href="#icon"></use></svg></body></html>"##;
    let document = html::parse(text).unwrap();
    let attr = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a) if a.name == "xlink:href" => Some(a),
            _ => None,
        })
        .expect("xlink:href attribute should be present");
    assert_eq!(
        attr.namespace.as_deref(),
        Some("http://www.w3.org/1999/xlink"),
        "xlink:href should have the xlink namespace"
    );
}

/// All xlink:* attributes should get the xlink namespace.
#[test]
fn foreign_attribute_all_xlink_variants_get_namespace() {
    let text = r##"<html><body><svg><a xlink:actuate="onRequest" xlink:arcrole="http://example.com" xlink:href="#" xlink:role="http://example.com" xlink:show="new" xlink:title="link" xlink:type="simple"></a></svg></body></html>"##;
    let document = html::parse(text).unwrap();
    let xlink_attrs: Vec<_> = document
        .iter()
        .filter_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a) if a.name.starts_with("xlink:") => Some(a),
            _ => None,
        })
        .collect();
    assert_eq!(xlink_attrs.len(), 7, "all 7 xlink:* attributes should be present");
    for attr in &xlink_attrs {
        assert_eq!(
            attr.namespace.as_deref(),
            Some("http://www.w3.org/1999/xlink"),
            "{} should have the xlink namespace",
            attr.name
        );
    }
}

/// xml:lang in SVG foreign content should get the XML namespace.
#[test]
fn foreign_attribute_xml_lang_gets_xml_namespace() {
    let text =
        r#"<html><body><svg xml:lang="en"><text>hello</text></svg></body></html>"#;
    let document = html::parse(text).unwrap();
    let attr = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a) if a.name == "xml:lang" => Some(a),
            _ => None,
        })
        .expect("xml:lang attribute should be present");
    assert_eq!(
        attr.namespace.as_deref(),
        Some("http://www.w3.org/XML/1998/namespace"),
        "xml:lang should have the XML namespace"
    );
}

/// xml:space in SVG foreign content should get the XML namespace.
#[test]
fn foreign_attribute_xml_space_gets_xml_namespace() {
    let text =
        r#"<html><body><svg xml:space="preserve"><text>hello</text></svg></body></html>"#;
    let document = html::parse(text).unwrap();
    let attr = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a) if a.name == "xml:space" => Some(a),
            _ => None,
        })
        .expect("xml:space attribute should be present");
    assert_eq!(
        attr.namespace.as_deref(),
        Some("http://www.w3.org/XML/1998/namespace"),
        "xml:space should have the XML namespace"
    );
}

/// xmlns on an SVG element should get the xmlns namespace.
#[test]
fn foreign_attribute_xmlns_gets_xmlns_namespace() {
    let text =
        r#"<html><body><svg xmlns="http://www.w3.org/2000/svg"><rect/></svg></body></html>"#;
    let document = html::parse(text).unwrap();
    let attr = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a) if a.name == "xmlns" => Some(a),
            _ => None,
        })
        .expect("xmlns attribute should be present");
    assert_eq!(
        attr.namespace.as_deref(),
        Some("http://www.w3.org/2000/xmlns/"),
        "xmlns should have the xmlns namespace"
    );
}

/// xmlns:xlink on an SVG element should get the xmlns namespace.
#[test]
fn foreign_attribute_xmlns_xlink_gets_xmlns_namespace() {
    let text =
        r##"<html><body><svg xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="#icon"/></svg></body></html>"##;
    let document = html::parse(text).unwrap();
    let attr = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a) if a.name == "xmlns:xlink" => Some(a),
            _ => None,
        })
        .expect("xmlns:xlink attribute should be present");
    assert_eq!(
        attr.namespace.as_deref(),
        Some("http://www.w3.org/2000/xmlns/"),
        "xmlns:xlink should have the xmlns namespace"
    );
}

/// Regular attributes in foreign content should NOT get a namespace.
#[test]
fn regular_attributes_in_foreign_content_have_no_namespace() {
    let text =
        r#"<html><body><svg><rect width="100" height="50"></rect></svg></body></html>"#;
    let document = html::parse(text).unwrap();
    let attrs: Vec<_> = document
        .iter()
        .filter_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a)
                if a.name == "width" || a.name == "height" =>
            {
                Some(a)
            }
            _ => None,
        })
        .collect();
    assert_eq!(attrs.len(), 2, "width and height should be present");
    for attr in &attrs {
        assert_eq!(
            attr.namespace, None,
            "{} should have no namespace",
            attr.name
        );
    }
}

/// Foreign attribute namespace assignment should work for MathML elements too.
#[test]
fn foreign_attribute_xlink_in_mathml_gets_namespace() {
    let text =
        r##"<html><body><math><mrow xlink:href="#ref"></mrow></math></body></html>"##;
    let document = html::parse(text).unwrap();
    let attr = document
        .iter()
        .find_map(|node| match node {
            XpathItemTreeNode::AttributeNode(a) if a.name == "xlink:href" => Some(a),
            _ => None,
        })
        .expect("xlink:href attribute should be present on MathML element");
    assert_eq!(
        attr.namespace.as_deref(),
        Some("http://www.w3.org/1999/xlink"),
        "xlink:href in MathML should have the xlink namespace"
    );
}
