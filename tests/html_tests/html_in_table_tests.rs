use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

/// Basic table structure: <table> with <tbody>, <tr>, <td>.
/// The InTable mode receives <tr> and implicitly creates <tbody>.
#[test]
fn table_basic_structure_with_implicit_tbody() {
    let text = r#"<html><head></head><body><table><tr><td>cell</td></tr></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("tbody", |tbody| {
                            tbody.add_element("tr", |tr| {
                                tr.add_element("td", |td| td.add_text("cell"))
                            })
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Table with explicit <tbody>.
#[test]
fn table_with_explicit_tbody() {
    let text = r#"<html><head></head><body><table><tbody></tbody></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| table.add_element("tbody", |tbody| tbody))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Table with explicit <thead> and <tfoot>.
#[test]
fn table_with_thead_tfoot() {
    let text =
        r#"<html><head></head><body><table><thead></thead><tfoot></tfoot></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table
                            .add_element("thead", |thead| thead)
                            .add_element("tfoot", |tfoot| tfoot)
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Comment inside a table is inserted as a comment node.
#[test]
fn table_comment() {
    let text =
        r#"<html><head></head><body><table><!-- table comment --></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_comment(" table comment ")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// </table> end tag closes the table and resets the insertion mode.
#[test]
fn table_end_tag_closes_table() {
    let text = r#"<html><head></head><body><table></table><p>after</p></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| table)
                        .add_element("p", |p| p.add_text("after"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Whitespace characters inside a table are handled by InTableText.
/// Pure whitespace should be inserted as text.
#[test]
fn table_whitespace_preserved() {
    let text = r#"<html><head></head><body><table> </table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| table.add_text(" "))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Non-whitespace text in a table triggers foster parenting.
/// The text should be foster-parented before the table element.
#[test]
fn table_text_foster_parented() {
    let text = r#"<html><head></head><body><table>hello</table></body></html>"#;

    let document = html::parse(text).unwrap();

    // "hello" is non-whitespace in table text, so it gets foster-parented
    // before the <table> element (into <body>, before <table>).
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_text("hello")
                        .add_element("table", |table| table)
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// End tags that don't belong in table (e.g. </body>) are parse errors and ignored.
#[test]
fn table_ignores_unexpected_end_tags() {
    let text = r#"<html><head></head><body><table></body></table></body></html>"#;

    let document = html::parse(text).unwrap();

    // </body> inside table is a parse error and ignored.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| table)
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <style> inside a table is processed using InHead rules.
#[test]
fn table_style_uses_in_head_rules() {
    let text =
        r#"<html><head></head><body><table><style>td{color:red}</style></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("style", |style| style.add_text("td{color:red}"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <script> inside a table is processed using InHead rules.
#[test]
fn table_script_uses_in_head_rules() {
    let text =
        r#"<html><head></head><body><table><script>var x=1;</script></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("script", |script| script.add_text("var x=1;"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Empty table.
#[test]
fn table_empty() {
    let text = r#"<html><head></head><body><table></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body.add_element("table", |table| table))
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <caption> start tag in table.
#[test]
fn table_caption() {
    let text =
        r#"<html><head></head><body><table><caption>Title</caption></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("caption", |caption| caption.add_text("Title"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <colgroup> start tag in table.
#[test]
fn table_colgroup() {
    let text =
        r#"<html><head></head><body><table><colgroup></colgroup></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("colgroup", |cg| cg)
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <td> and <th> in table implicitly create <tbody>.
#[test]
fn table_td_implicitly_creates_tbody() {
    let text = r#"<html><head></head><body><table><td>data</td></table></body></html>"#;

    let document = html::parse(text).unwrap();

    // When InTable sees <td>, it creates an implicit <tbody> and switches to InTableBody.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("tbody", |tbody| {
                            tbody.add_element("tr", |tr| {
                                tr.add_element("td", |td| td.add_text("data"))
                            })
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// EOF inside table uses InBody rules for EOF.
#[test]
fn table_eof() {
    let text = r#"<html><head></head><body><table>"#;

    let document = html::parse(text).unwrap();

    // EOF in InTable delegates to InBody which calls stop_parsing.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body.add_element("table", |table| table))
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Mixed whitespace and newlines in table text (all whitespace) should be preserved.
#[test]
fn table_whitespace_newlines_preserved() {
    let text = "<html><head></head><body><table>\n\t</table></body></html>";

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| table.add_text("\n\t"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <input type="hidden"> in table is handled specially — inserted and immediately popped.
#[test]
fn table_hidden_input() {
    let text = r#"<html><head></head><body><table><input type="hidden" name="token" value="abc"></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("input", |input| {
                            input
                                .add_attribute_str("type", "hidden")
                                .add_attribute_str("name", "token")
                                .add_attribute_str("value", "abc")
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <input> without type="hidden" in table triggers foster parenting.
#[test]
fn table_non_hidden_input_foster_parented() {
    let text = r#"<html><head></head><body><table><input type="text"></table></body></html>"#;

    let document = html::parse(text).unwrap();

    // <input type="text"> in table triggers "anything else" (foster parenting).
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("input", |input| {
                        input.add_attribute_str("type", "text")
                    })
                    .add_element("table", |table| table)
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Nested <table> start tag: the first table is closed, then the second is opened.
#[test]
fn table_nested_table_closes_first() {
    let text = r#"<html><head></head><body><table><table></table></table></body></html>"#;

    let document = html::parse(text).unwrap();

    // A <table> start tag inside InTable is a parse error.
    // It pops until the first <table> is closed, resets insertion mode,
    // then reprocesses the token. This results in two sibling tables.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| table)
                        .add_element("table", |table| table)
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
