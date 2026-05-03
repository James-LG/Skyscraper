use skyscraper::{html, xpath};

/// `//li ! .` should return the same nodes as `//li`.
#[test]
fn simple_map_identity() {
    let text = r#"<html><body><ul><li>a</li><li>b</li><li>c</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath_map = xpath::parse("//li ! .").unwrap();
    let xpath_plain = xpath::parse("//li").unwrap();

    let items_map = xpath_map.apply(&document).unwrap();
    let items_plain = xpath_plain.apply(&document).unwrap();
    assert_eq!(items_map.len(), items_plain.len());
    assert_eq!(items_map.len(), 3);
}

/// `//li ! text()` should return the text node of each li.
#[test]
fn simple_map_select_child_text() {
    let text = r#"<html><body><ul><li>alpha</li><li>beta</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//li ! text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 text nodes: {items:?}");
}

/// Chained map: `//ul ! li ! text()`.
/// First selects <ul>, then for each <ul> selects child <li>, then for each <li> selects text().
#[test]
fn simple_map_chained() {
    let text =
        r#"<html><body><ul><li>x</li><li>y</li></ul><ul><li>z</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//ul ! li ! text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3, "should return 3 text nodes: {items:?}");
}

/// Map with no items (no `!`) should just return the base expression.
#[test]
fn simple_map_no_items_returns_base() {
    let text = r#"<html><body><div>hello</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
}

/// Map over attributes: `//div ! @class` selects the class attribute of each div.
#[test]
fn simple_map_attributes() {
    let text = r#"<html><body><div class="a">1</div><div class="b">2</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div ! @class").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 attribute nodes: {items:?}");
}

/// Map distributes over multiple <ul> elements independently.
#[test]
fn simple_map_multiple_sources() {
    let text = r#"<html><body>
        <ul><li>a</li><li>b</li></ul>
        <ol><li>c</li></ol>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//ul ! li").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 li from ul only: {items:?}");
}
