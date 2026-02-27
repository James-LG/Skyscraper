use skyscraper::{
    html,
    xpath::{self, grammar::data_model::XpathItem},
};

// ── ancestor:: axis ──────────────────────────────────────────────────

/// `ancestor::*` returns all ancestors of the context node.
#[test]
fn ancestor_returns_all_ancestors() {
    let text = r#"<html><body><div><p>hello</p></div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//p/ancestor::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    let names: Vec<&str> = items
        .iter()
        .filter_map(|i| match i {
            XpathItem::Node(n) => n.as_element_node().ok().map(|e| e.name.as_str()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"div"));
    assert!(names.contains(&"body"));
    assert!(names.contains(&"html"));
}

/// `ancestor::div` filters ancestors by name.
#[test]
fn ancestor_with_name_test() {
    let text = r#"<html><body><div><p>hello</p></div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//p/ancestor::div").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "div");
}

/// Root element has no ancestors (except the document node).
#[test]
fn ancestor_of_root_element() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    // html's ancestors that are elements: none (document node is not an element)
    let xpath = xpath::parse("/html/ancestor::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

// ── ancestor-or-self:: axis ──────────────────────────────────────────

/// `ancestor-or-self::*` includes the context node itself.
#[test]
fn ancestor_or_self_includes_self() {
    let text = r#"<html><body><div><p>hello</p></div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//p/ancestor-or-self::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    let names: Vec<&str> = items
        .iter()
        .filter_map(|i| match i {
            XpathItem::Node(n) => n.as_element_node().ok().map(|e| e.name.as_str()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"p"));
    assert!(names.contains(&"div"));
    assert!(names.contains(&"body"));
    assert!(names.contains(&"html"));
}

/// `ancestor-or-self::p` matches the context node itself.
#[test]
fn ancestor_or_self_matches_self_name() {
    let text = r#"<html><body><p>hello</p></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//p/ancestor-or-self::p").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "p");
}

// ── preceding-sibling:: axis ─────────────────────────────────────────

/// `preceding-sibling::*` returns all preceding siblings.
#[test]
fn preceding_sibling_returns_all_preceding() {
    let text = r#"<html><body><a>1</a><b>2</b><c>3</c><d>4</d></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//c/preceding-sibling::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    let names: Vec<&str> = items
        .iter()
        .map(|i| i.extract_as_node().extract_as_element_node().name.as_str())
        .collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
    assert_eq!(items.len(), 2);
}

/// `preceding-sibling::` with name test filters.
#[test]
fn preceding_sibling_with_name_test() {
    let text = r#"<html><body><a>1</a><b>2</b><a>3</a></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//a[2]/preceding-sibling::b").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "b");
}

/// First element has no preceding siblings.
#[test]
fn preceding_sibling_of_first_is_empty() {
    let text = r#"<html><body><a>1</a><b>2</b></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//a/preceding-sibling::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

// ── preceding:: axis ─────────────────────────────────────────────────

/// `preceding::*` returns all nodes before the context node (excluding ancestors).
#[test]
fn preceding_axis_returns_all_preceding_nodes() {
    let text = r#"<html><body><div><a>1</a></div><p>2</p><span>3</span></body></html>"#;

    let document = html::parse(text).unwrap();
    // preceding:: of <span> should include <p>, <a>, <div> and their text nodes.
    let xpath = xpath::parse("//span/preceding::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    let names: Vec<&str> = items
        .iter()
        .filter_map(|i| match i {
            XpathItem::Node(n) => n.as_element_node().ok().map(|e| e.name.as_str()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"div"));
    assert!(names.contains(&"a"));
    assert!(names.contains(&"p"));
    // <span> and ancestors (body, html) should not be in preceding.
    assert!(!names.contains(&"span"));
    assert!(!names.contains(&"body"));
    assert!(!names.contains(&"html"));
}

/// `preceding::` includes descendants of preceding siblings.
#[test]
fn preceding_axis_includes_descendants() {
    let text =
        r#"<html><body><div><p>inner</p></div><a>last</a></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//a/preceding::p").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "p");
}
