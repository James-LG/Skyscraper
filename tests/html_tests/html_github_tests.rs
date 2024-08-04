use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

#[test]
fn text_should_unescape_characters() {
    // arrange
    let text = r##"<div>&amp;&quot;&#39;&lt;&gt;&#96;</div>"##;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .with_root("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text(r##"&"'<>`"##))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

// #[test]
// fn doctype_should_skip_regular_doctype() {
//     // arrange
//     let text = r##"
//         <!DOCTYPE html>
//         <div>hi</div>"##;

//     // act
//     let document = html::parse(text).unwrap();

//     // assert
//     let root_node = document.root_node;
//     let html_tag = document.get_html_node(&root_node).unwrap().extract_as_tag();
//     assert_eq!(html_tag.name, "div");
// }

// #[test]
// fn doctype_should_skip_verbose_doctype() {
//     // arrange
//     let text = r##"
//         <!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
//         <div>hi</div>"##;

//     // act
//     let document = html::parse(text).unwrap();

//     // assert
//     let root_node = document.root_node;
//     let html_tag = document.get_html_node(&root_node).unwrap().extract_as_tag();
//     assert_eq!(html_tag.name, "div");
// }
