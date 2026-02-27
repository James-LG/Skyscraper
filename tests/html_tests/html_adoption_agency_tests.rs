use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

/// Basic adoption agency: nested `<b>` tags inside a `<p>` should be handled.
/// Per WHATWG, `<p>hello<b>world</p>more</b>` should close the `<b>` at the `</p>`
/// and reopen it after.
#[test]
fn adoption_agency_basic_formatting_in_paragraph() {
    // <b> is a formatting element that should be handled by the adoption agency
    let text = r#"<div><b>bold text</b></div>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_element("b", |b| b.add_text("bold text"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Test that misnested formatting tags are handled correctly.
/// `<b><i>text</b>more</i>` should produce:
///   <b><i>text</i></b><i>more</i>
/// per the adoption agency algorithm.
#[test]
fn adoption_agency_misnested_formatting_tags() {
    let text = r#"<div><b><i>text</b>more</i></div>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_element("b", |b| {
                            b.add_element("i", |i| i.add_text("text"))
                        })
                        .add_element("i", |i| i.add_text("more"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Test adoption agency with anchor tags - nested `<a>` tags.
/// Per WHATWG, a new `<a>` while one is already in active formatting
/// should close the old one via adoption agency before opening the new one.
#[test]
fn adoption_agency_nested_anchor_tags() {
    let text = r#"<div><a href="1">link1<a href="2">link2</a></div>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_element("a", |a| {
                            a.add_text("link1").add_attribute_str("href", "1")
                        })
                        .add_element("a", |a| {
                            a.add_text("link2").add_attribute_str("href", "2")
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Per WHATWG, block-level elements like `<div>` are simply inserted as children
/// of the current node (formatting element `<b>`), without closing the formatting.
/// `<b>bold<div>block</div>still bold</b>` nests div inside b.
#[test]
fn adoption_agency_formatting_across_block_element() {
    let text = r#"<body><b>bold<div>block</div>still bold</b></body>"#;

    let document = html::parse(text).unwrap();

    // Per WHATWG spec, <div> is inserted as child of <b> without closing <b>.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("b", |b| {
                        b.add_text("bold")
                            .add_element("div", |div| div.add_text("block"))
                            .add_text("still bold")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Test that `<em>` and `<strong>` formatting elements work properly.
#[test]
fn adoption_agency_em_and_strong() {
    let text = r#"<div><em>emphasized</em> and <strong>strong</strong></div>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_element("em", |em| em.add_text("emphasized"))
                            .add_text(" and ")
                            .add_element("strong", |s| s.add_text("strong"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// WHATWG adoption agency Step 4.6: if the formatting element is in the stack
/// of open elements but NOT in scope, the end tag should be ignored.
/// Here `<table>` is a scope barrier (in ELEMENT_IN_SCOPE_TYPES) that does NOT
/// push a marker onto active formatting, so `<b>` remains findable in step 4.3
/// but is unreachable via the scope walk.
#[test]
fn adoption_agency_formatting_element_not_in_scope_table_barrier() {
    let text = r#"<body><b><table></b></table></body>"#;

    let document = html::parse(text).unwrap();

    // `</b>` is encountered inside InTable, processed via InBody.
    // `b` is in active formatting and in the stack, but `table` is a scope
    // barrier so `b` is not in scope.  Per spec step 4.6 the end tag is
    // ignored, leaving `b` open with `table` as its child.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("b", |b| {
                        b.add_element("table", |table| table)
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Same as above but with text before and after the table inside the `<b>`.
/// The text before the table should be a child of `<b>`, the table should also
/// be a child of `<b>`, and text after `</table>` (while `<b>` is still open)
/// should also be a child of `<b>`.
#[test]
fn adoption_agency_formatting_not_in_scope_with_surrounding_text() {
    let text = r#"<body><b>before<table></b></table>after</b></body>"#;

    let document = html::parse(text).unwrap();

    // "before" is added to `b` before the table is opened.
    // `</b>` inside the table context is ignored (not in scope).
    // `</table>` closes the table, returning to InBody with `b` still open.
    // "after" is added to `b`.
    // The second `</b>` closes `b` normally.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("b", |b| {
                        b.add_text("before")
                            .add_element("table", |table| table)
                            .add_text("after")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Test that the GitHub sample HTML can be parsed without panicking.
/// The full round-trip check is in html_github_tests; this confirms no panic.
#[test]
fn github_sample_should_parse_without_panic() {
    let text: String = include_str!("../samples/James-LG_Skyscraper.html")
        .parse()
        .unwrap();
    let document = html::parse(&text).unwrap();

    // Just verify we can serialize it without panicking
    let _output = document.to_string();
}
