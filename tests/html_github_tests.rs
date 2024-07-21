use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

mod test_framework;

// #[test]
// fn text_should_unescape_characters() {
//     // arrange
//     let text = r##"<div>&amp;&quot;&#39;&lt;&gt;&#96;</div>"##;

//     // act
//     let document = html::parse(text).unwrap();

//     // assert
//     let root_node = document.root_node;
//     let mut children = root_node.children(&document);

//     let child = children.next().unwrap();
//     let html_text = document.get_html_node(&child).unwrap().extract_as_text();
//     assert_eq!(html_text.value, r##"&"'<>`"##);
// }

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
