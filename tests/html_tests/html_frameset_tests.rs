use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

/// Basic frameset document with two frames.
#[test]
fn frameset_basic() {
    let text = r#"<html><head></head><frameset cols="50%,50%"><frame src="a.html"><frame src="b.html"></frameset></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| {
                    fs.add_attribute_str("cols", "50%,50%")
                        .add_element("frame", |f| f.add_attribute_str("src", "a.html"))
                        .add_element("frame", |f| f.add_attribute_str("src", "b.html"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Nested frameset elements.
#[test]
fn frameset_nested() {
    let text = r#"<html><head></head><frameset rows="50%,50%"><frameset cols="50%,50%"><frame src="a.html"><frame src="b.html"></frameset><frame src="c.html"></frameset></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| {
                    fs.add_attribute_str("rows", "50%,50%")
                        .add_element("frameset", |inner| {
                            inner
                                .add_attribute_str("cols", "50%,50%")
                                .add_element("frame", |f| f.add_attribute_str("src", "a.html"))
                                .add_element("frame", |f| f.add_attribute_str("src", "b.html"))
                        })
                        .add_element("frame", |f| f.add_attribute_str("src", "c.html"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Frameset with noframes fallback content.
#[test]
fn frameset_with_noframes() {
    let text = r#"<html><head></head><frameset><frame src="a.html"><noframes>No frames</noframes></frameset></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| {
                    fs.add_element("frame", |f| f.add_attribute_str("src", "a.html"))
                        .add_element("noframes", |nf| nf.add_text("No frames"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Comment after </html> in a frameset document is inserted at the Document level
/// (exercises AfterAfterFrameset insertion mode).
#[test]
fn frameset_comment_after_html() {
    let text =
        r#"<html><head></head><frameset><frame src="a.html"></frameset></html><!-- after -->"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| {
                    fs.add_element("frame", |f| f.add_attribute_str("src", "a.html"))
                })
        })
        .add_comment(" after ")
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Non-frame content inside frameset is ignored (parse error, anything-else branch).
#[test]
fn frameset_ignores_non_frame_content() {
    let text = r#"<html><head></head><frameset><div>ignored</div><frame src="a.html"></frameset></html>"#;

    let document = html::parse(text).unwrap();

    // The <div>ignored</div> tokens should be ignored as parse errors.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| {
                    fs.add_element("frame", |f| f.add_attribute_str("src", "a.html"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Frameset with whitespace between elements (whitespace should be preserved).
#[test]
fn frameset_with_whitespace() {
    let text = "<html><head></head><frameset>\n<frame src=\"a.html\">\n</frameset></html>";

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| {
                    fs.add_text("\n")
                        .add_element("frame", |f| f.add_attribute_str("src", "a.html"))
                        .add_text("\n")
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Frameset with comment nodes.
#[test]
fn frameset_with_comments() {
    let text =
        r#"<html><head></head><frameset><!-- frame list --><frame src="a.html"></frameset></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| {
                    fs.add_comment(" frame list ")
                        .add_element("frame", |f| f.add_attribute_str("src", "a.html"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Frameset with no frames at all (empty frameset).
#[test]
fn frameset_empty() {
    let text = r#"<html><head></head><frameset></frameset></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("frameset", |fs| fs)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
