use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

/// Basic <select> with <option> children.
#[test]
fn select_basic_options() {
    let text =
        r#"<html><head></head><body><select><option>A</option><option>B</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select
                            .add_element("option", |opt| opt.add_text("A"))
                            .add_element("option", |opt| opt.add_text("B"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Select with optgroup containing options.
#[test]
fn select_optgroup_with_options() {
    let text = r#"<html><head></head><body><select><optgroup label="fruits"><option>Apple</option><option>Banana</option></optgroup></select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("optgroup", |og| {
                            og.add_attribute_str("label", "fruits")
                                .add_element("option", |opt| opt.add_text("Apple"))
                                .add_element("option", |opt| opt.add_text("Banana"))
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Multiple optgroups in select.
#[test]
fn select_multiple_optgroups() {
    let text = r#"<html><head></head><body><select><optgroup label="a"><option>1</option></optgroup><optgroup label="b"><option>2</option></optgroup></select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select
                            .add_element("optgroup", |og| {
                                og.add_attribute_str("label", "a")
                                    .add_element("option", |opt| opt.add_text("1"))
                            })
                            .add_element("optgroup", |og| {
                                og.add_attribute_str("label", "b")
                                    .add_element("option", |opt| opt.add_text("2"))
                            })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <option> start tag implicitly closes a previous <option>.
#[test]
fn select_option_closes_previous_option() {
    let text = r#"<html><head></head><body><select><option>A<option>B</select></body></html>"#;

    let document = html::parse(text).unwrap();

    // Per the spec, a new <option> pops the current <option> if open.
    // Both options become direct children of select.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select
                            .add_element("option", |opt| opt.add_text("A"))
                            .add_element("option", |opt| opt.add_text("B"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <optgroup> start tag implicitly closes a previous <option> and <optgroup>.
#[test]
fn select_optgroup_closes_option_and_optgroup() {
    let text = r#"<html><head></head><body><select><optgroup><option>A<optgroup><option>B</optgroup></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // Per the spec:
    // 1. First <optgroup> is opened.
    // 2. <option>A is opened inside the first optgroup.
    // 3. Second <optgroup> pops <option>A (current is option), then pops the first <optgroup>.
    // 4. <option>B is opened inside the second optgroup.
    // 5. </optgroup> closes the second optgroup.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select
                            .add_element("optgroup", |og| {
                                og.add_element("option", |opt| opt.add_text("A"))
                            })
                            .add_element("optgroup", |og| {
                                og.add_element("option", |opt| opt.add_text("B"))
                            })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// </optgroup> end tag when current node is option and previous is optgroup:
/// pops both option and optgroup.
#[test]
fn select_end_optgroup_pops_option_then_optgroup() {
    let text = r#"<html><head></head><body><select><optgroup><option>A</optgroup></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // Per the spec for </optgroup>:
    // 1. Current node is <option>, node before is <optgroup> → pop option.
    // 2. Now current node is <optgroup> → pop it.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("optgroup", |og| {
                            og.add_element("option", |opt| opt.add_text("A"))
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Nested <select> start tag closes the current select (parse error).
/// Per the spec, <select> in InSelect closes the select and resets the mode,
/// but does NOT reprocess the token (unlike input/textarea).
#[test]
fn select_nested_select_closes_first() {
    let text = r#"<html><head></head><body><select><option>A</option><select><option>B</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // The second <select> closes the first select (parse error).
    // It does NOT open a new select — the token is consumed.
    // Then <option>B is processed in InBody, which creates an option child of body.
    // </option> and </select> are ignored (parse error: nothing to close).
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                    .add_element("option", |opt| opt.add_text("B"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Character text inside select is preserved.
#[test]
fn select_text_content() {
    let text = r#"<html><head></head><body><select>some text</select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| select.add_text("some text"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Comment inside select is preserved.
#[test]
fn select_comment() {
    let text =
        r#"<html><head></head><body><select><!-- comment --></select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| select.add_comment(" comment "))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <input> inside select closes the select and reprocesses the token.
#[test]
fn select_input_closes_select() {
    let text = r#"<html><head></head><body><select><option>A</option><input type="text"></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // <input> in InSelect pops until select is popped, resets insertion mode,
    // then reprocesses <input> in InBody which inserts it as a void element.
    // The trailing </select> is ignored (no select in scope).
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                    .add_element("input", |input| input.add_attribute_str("type", "text"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Empty select element.
#[test]
fn select_empty() {
    let text = r#"<html><head></head><body><select></select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body.add_element("select", |select| select))
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Select with <hr> element — hr is a void element that gets popped immediately.
#[test]
fn select_hr_element() {
    let text = r#"<html><head></head><body><select><option>A</option><hr><option>B</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select
                            .add_element("option", |opt| opt.add_text("A"))
                            .add_element("hr", |hr| hr)
                            .add_element("option", |opt| opt.add_text("B"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// EOF inside select delegates to InBody for EOF handling.
#[test]
fn select_eof() {
    let text = r#"<html><head></head><body><select><option>A"#;

    let document = html::parse(text).unwrap();

    // EOF in select is handled by InBody's EOF, which calls stop_parsing.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// <textarea> inside select closes the select and reprocesses.
#[test]
fn select_textarea_closes_select() {
    let text = r#"<html><head></head><body><select><option>A</option><textarea>text</textarea></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // <textarea> in InSelect: pop until select, reset mode, reprocess.
    // <textarea> in InBody creates a textarea element.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                    .add_element("textarea", |ta| ta.add_text("text"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Select inside a table cell enters InSelectInTable mode.
#[test]
fn select_in_table_cell() {
    let text = r#"<html><head></head><body><table><tr><td><select><option>A</option></select></td></tr></table></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("tbody", |tbody| {
                            tbody.add_element("tr", |tr| {
                                tr.add_element("td", |td| {
                                    td.add_element("select", |select| {
                                        select
                                            .add_element("option", |opt| opt.add_text("A"))
                                    })
                                })
                            })
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// InSelectInTable: table start tag inside select closes the select.
#[test]
fn select_in_table_table_start_closes_select() {
    let text = r#"<html><head></head><body><table><tr><td><select><option>A</option><table></table></select></td></tr></table></body></html>"#;

    let document = html::parse(text).unwrap();

    // <table> in InSelectInTable: pop until select, reset mode, reprocess <table>.
    // This creates a nested table structure.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("tbody", |tbody| {
                            tbody.add_element("tr", |tr| {
                                tr.add_element("td", |td| {
                                    td.add_element("select", |select| {
                                        select
                                            .add_element("option", |opt| opt.add_text("A"))
                                    })
                                    .add_element("table", |t| t)
                                })
                            })
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Select with attributes.
#[test]
fn select_with_attributes() {
    let text = r#"<html><head></head><body><select name="color" id="sel"><option value="r">Red</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select
                            .add_attribute_str("name", "color")
                            .add_attribute_str("id", "sel")
                            .add_element("option", |opt| {
                                opt.add_attribute_str("value", "r").add_text("Red")
                            })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Anything else in InSelect is a parse error and ignored.
/// E.g. <div> inside select is ignored.
#[test]
fn select_ignores_unexpected_tags() {
    let text = r#"<html><head></head><body><select><div>text</div><option>A</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // <div> is an "anything else" case in InSelect — parse error, token ignored.
    // But the text "text" inside will be inserted as character tokens.
    // </div> is also ignored.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select
                            .add_text("text")
                            .add_element("option", |opt| opt.add_text("A"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Option and optgroup in body (outside select) — InBody handles these.
#[test]
fn option_in_body_outside_select() {
    let text = r#"<html><head></head><body><option>A</option><option>B</option></body></html>"#;

    let document = html::parse(text).unwrap();

    // InBody's optgroup/option handler: if current node is option, pop it first.
    // Then reconstruct formatting elements and insert.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("option", |opt| opt.add_text("A"))
                        .add_element("option", |opt| opt.add_text("B"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// </optgroup> when current node is not option or optgroup — parse error, ignored.
#[test]
fn select_end_optgroup_parse_error_when_not_optgroup() {
    let text = r#"<html><head></head><body><select></optgroup><option>A</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // </optgroup> right after <select> — current node is select, not optgroup.
    // Parse error, token ignored. The option is still inserted normally.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// </option> when current node is not option — parse error, ignored.
#[test]
fn select_end_option_parse_error_when_not_option() {
    let text = r#"<html><head></head><body><select></option><option>A</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // </option> right after <select> — current node is select, not option.
    // Parse error, token ignored.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// NULL character inside select — parse error, ignored.
#[test]
fn select_null_character_ignored() {
    let text = "<html><head></head><body><select>\0<option>A</option></select></body></html>";

    let document = html::parse(text).unwrap();

    // NULL character is a parse error and ignored — not inserted as text.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// DOCTYPE inside select — parse error, ignored.
#[test]
fn select_doctype_ignored() {
    let text =
        r#"<html><head></head><body><select><!DOCTYPE html><option>A</option></select></body></html>"#;

    let document = html::parse(text).unwrap();

    // DOCTYPE in select is a parse error, ignored.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("select", |select| {
                        select.add_element("option", |opt| opt.add_text("A"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// InSelectInTable: end tag when element not in table scope — ignored.
#[test]
fn select_in_table_end_tag_not_in_scope_ignored() {
    let text = r#"<html><head></head><body><table><tr><td><select></caption><option>A</option></select></td></tr></table></body></html>"#;

    let document = html::parse(text).unwrap();

    // </caption> in InSelectInTable — parse error.
    // No caption in table scope, so token is ignored. Select continues normally.
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("table", |table| {
                        table.add_element("tbody", |tbody| {
                            tbody.add_element("tr", |tr| {
                                tr.add_element("td", |td| {
                                    td.add_element("select", |select| {
                                        select
                                            .add_element("option", |opt| opt.add_text("A"))
                                    })
                                })
                            })
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
