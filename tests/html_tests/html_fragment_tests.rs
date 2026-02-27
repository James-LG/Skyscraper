use skyscraper::html;
use skyscraper::xpath;

#[test]
fn fragment_empty_input() {
    let tree = html::parse_fragment("div", "").unwrap();

    // Should produce a valid tree with just the html root
    let xp = xpath::parse("/html").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn fragment_div_context_parses_body_content() {
    let tree = html::parse_fragment("div", "<p>Hello</p>").unwrap();

    let xp = xpath::parse("//p").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 1);

    let element = items[0].extract_as_node().extract_as_element_node();
    assert_eq!(element.name, "p");
    assert_eq!(element.text_content(&tree), "Hello");
}

#[test]
fn fragment_body_context_parses_in_body_mode() {
    let tree = html::parse_fragment("body", "<div>Content</div>").unwrap();

    let xp = xpath::parse("//div").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 1);

    let element = items[0].extract_as_node().extract_as_element_node();
    assert_eq!(element.name, "div");
    assert_eq!(element.text_content(&tree), "Content");
}

#[test]
fn fragment_table_context_parses_in_table_mode() {
    let tree = html::parse_fragment("table", "<tr><td>Cell</td></tr>").unwrap();

    let xp = xpath::parse("//td").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 1);

    let element = items[0].extract_as_node().extract_as_element_node();
    assert_eq!(element.text_content(&tree), "Cell");
}

#[test]
fn fragment_select_context_parses_in_select_mode() {
    let tree = html::parse_fragment("select", "<option>A</option><option>B</option>").unwrap();

    let xp = xpath::parse("//option").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn fragment_title_context_uses_rcdata() {
    // In RCDATA mode, tags are treated as text
    let tree = html::parse_fragment("title", "<p>Not a tag</p>").unwrap();

    // There should be no <p> element — the markup is treated as text
    let xp = xpath::parse("//p").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn fragment_style_context_uses_rawtext() {
    // In RAWTEXT mode, content is treated as raw text
    let tree = html::parse_fragment("style", "body { color: red; }").unwrap();

    // There should be no elements parsed from the CSS — it's raw text
    let xp = xpath::parse("//body").unwrap();
    let items = xp.apply(&tree).unwrap();
    // The "body" text in the CSS should not create a body element
    assert_eq!(items.len(), 0);
}

#[test]
fn fragment_template_context_pushes_in_template_mode() {
    let tree = html::parse_fragment("template", "<div>Template content</div>").unwrap();

    let xp = xpath::parse("//div").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 1);

    let element = items[0].extract_as_node().extract_as_element_node();
    assert_eq!(element.text_content(&tree), "Template content");
}

#[test]
fn fragment_xpath_works_on_fragment_tree() {
    let tree = html::parse_fragment(
        "div",
        r#"<ul><li class="a">First</li><li class="b">Second</li></ul>"#,
    )
    .unwrap();

    // Test descendant axis
    let xp = xpath::parse("//li").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 2);

    // Test attribute predicate
    let xp = xpath::parse("//li[@class='b']").unwrap();
    let items = xp.apply(&tree).unwrap();
    assert_eq!(items.len(), 1);
    let element = items[0].extract_as_node().extract_as_element_node();
    assert_eq!(element.text_content(&tree), "Second");
}
