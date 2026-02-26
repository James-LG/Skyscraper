use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

/// Basic noscript element in head is parsed as a tree node with its content visible
/// (scripting is disabled in Skyscraper).
#[test]
fn noscript_in_head_basic() {
    // arrange
    let text = r#"<html><head><noscript><style>body{color:red}</style></noscript></head><body></body></html>"#;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("noscript", |noscript| {
                    noscript.add_element("style", |style| style.add_text("body{color:red}"))
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// noscript with a link element inside (void element handled via InHead rules).
#[test]
fn noscript_in_head_with_link() {
    let text = r#"<html><head><noscript><link rel="stylesheet" href="style.css"></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("noscript", |noscript| {
                    noscript.add_element("link", |link| {
                        link.add_attribute_str("rel", "stylesheet")
                            .add_attribute_str("href", "style.css")
                    })
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// noscript with a meta element inside.
#[test]
fn noscript_in_head_with_meta() {
    let text = r#"<html><head><noscript><meta http-equiv="refresh" content="0;url=fallback.html"></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("noscript", |noscript| {
                    noscript.add_element("meta", |meta| {
                        meta.add_attribute_str("http-equiv", "refresh")
                            .add_attribute_str("content", "0;url=fallback.html")
                    })
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// An unexpected start tag like <div> inside noscript in head triggers
/// the "anything else" case: pop noscript, switch to InHead, and reprocess.
/// The <div> ends up in body since InHead's anything_else pops head and goes to AfterHead.
#[test]
fn noscript_in_head_unexpected_start_tag_causes_reprocess() {
    let text = r#"<html><head><noscript><div>hello</div></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    // The <div> is unexpected in InHeadNoscript, so noscript is popped,
    // mode goes to InHead, then InHead's anything_else pops head and goes to AfterHead,
    // which creates <body> and the <div> ends up there.
    // The </noscript>, </head> end tags are then ignored or handled by in body.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head.add_element("noscript", |ns| ns))
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text("hello"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// A </noscript> end tag properly closes the noscript element and returns to InHead.
#[test]
fn noscript_end_tag_returns_to_in_head() {
    let text =
        r#"<html><head><noscript></noscript><title>Test</title></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("noscript", |ns| ns)
                    .add_element("title", |title| title.add_text("Test"))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// A <head> start tag inside noscript in head is a parse error and should be ignored.
#[test]
fn noscript_in_head_ignores_head_start_tag() {
    let text = r#"<html><head><noscript><head></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    // <head> is ignored, noscript is empty
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head.add_element("noscript", |ns| ns))
                .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// A nested <noscript> start tag inside noscript in head is a parse error and should be ignored.
#[test]
fn noscript_in_head_ignores_nested_noscript_start_tag() {
    let text = r#"<html><head><noscript><noscript></noscript></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    // The inner <noscript> is ignored. The first </noscript> closes the outer noscript.
    // The second </noscript> is an unexpected end tag in InHead and is ignored (parse error).
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head.add_element("noscript", |ns| ns))
                .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Comment inside noscript in head is processed using InHead rules (inserted as comment).
#[test]
fn noscript_in_head_comment() {
    let text =
        r#"<html><head><noscript><!-- a comment --></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("noscript", |ns| ns.add_comment(" a comment "))
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Multiple elements inside noscript in head.
#[test]
fn noscript_in_head_multiple_elements() {
    let text = r#"<html><head><noscript><link rel="stylesheet" href="a.css"><style>body{}</style><link rel="stylesheet" href="b.css"></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("noscript", |ns| {
                    ns.add_element("link", |link| {
                        link.add_attribute_str("rel", "stylesheet")
                            .add_attribute_str("href", "a.css")
                    })
                    .add_element("style", |style| style.add_text("body{}"))
                    .add_element("link", |link| {
                        link.add_attribute_str("rel", "stylesheet")
                            .add_attribute_str("href", "b.css")
                    })
                })
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// EOF inside noscript in head triggers "anything else": pop noscript, switch to InHead,
/// reprocess EOF which cascades through InHead and AfterHead.
#[test]
fn noscript_in_head_eof() {
    let text = r#"<html><head><noscript>"#;

    let document = html::parse(text).unwrap();

    // EOF triggers anything_else: pop noscript, switch to InHead, reprocess.
    // InHead's anything_else pops head, switches to AfterHead, reprocesses.
    // AfterHead's anything_else creates body, switches to InBody, reprocesses.
    // InBody handles EOF with stop_parsing.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head.add_element("noscript", |ns| ns))
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// A </br> end tag inside noscript in head acts as "anything else" per spec.
#[test]
fn noscript_in_head_br_end_tag() {
    let text = r#"<html><head><noscript></br></noscript></head><body></body></html>"#;

    let document = html::parse(text).unwrap();

    // </br> triggers anything_else: pop noscript, switch to InHead, reprocess.
    // InHead sees </br> which matches the ["body", "html", "br"] branch,
    // which also calls anything_else: pop head, switch to AfterHead, reprocess.
    // AfterHead creates body and reprocesses </br> in InBody.
    // InBody converts </br> to a <br> element.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head.add_element("noscript", |ns| ns))
                .add_element("body", |body| body.add_element("br", |br| br))
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
