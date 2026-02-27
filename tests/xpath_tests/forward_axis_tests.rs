use skyscraper::{
    html,
    xpath::{self, grammar::data_model::XpathItem},
};

// ── self:: axis ──────────────────────────────────────────────────────

/// `self::node()` returns the context node itself.
#[test]
fn self_axis_returns_context_node() {
    let text = r#"<html><body><p>hello</p></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//p/self::node()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "p");
}

/// `self::p` matches when the context node is a `<p>`, filters otherwise.
#[test]
fn self_axis_with_name_test() {
    let text = r#"<html><body><p>one</p><div>two</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/*/self::p").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "p");
}

/// `.` is an abbreviation for `self::node()`.
#[test]
fn self_axis_dot_abbreviation() {
    let text = r#"<html><body><p>text</p></body></html>"#;

    let document = html::parse(text).unwrap();
    // Using . in a path that selects the <p> element
    let xpath = xpath::parse("//p/.").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "p");
}

// ── following-sibling:: axis ─────────────────────────────────────────

/// `following-sibling::*` returns all following siblings.
#[test]
fn following_sibling_returns_all_following() {
    let text = r#"<html><body><a>1</a><b>2</b><c>3</c><d>4</d></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//b/following-sibling::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2);
    let names: Vec<&str> = items
        .iter()
        .map(|i| i.extract_as_node().extract_as_element_node().name.as_str())
        .collect();
    assert_eq!(names, vec!["c", "d"]);
}

/// `following-sibling::` with a name test filters by element name.
#[test]
fn following_sibling_with_name_test() {
    let text = r#"<html><body><a>1</a><b>2</b><a>3</a><c>4</c></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//a[1]/following-sibling::a").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "a");
}

/// Last element has no following siblings.
#[test]
fn following_sibling_of_last_is_empty() {
    let text = r#"<html><body><a>1</a><b>2</b></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//b/following-sibling::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

// ── following:: axis ─────────────────────────────────────────────────

/// `following::*` returns all nodes after the context node in document order (excluding descendants).
#[test]
fn following_axis_returns_all_following_nodes() {
    let text = r#"<html><body><div><a>1</a></div><p>2</p><span>3</span></body></html>"#;

    let document = html::parse(text).unwrap();
    // following:: of <a> should include <p> and <span> (siblings of <div>), but not <div> (ancestor's sibling container).
    let xpath = xpath::parse("//a/following::*").unwrap();

    let items = xpath.apply(&document).unwrap();
    let names: Vec<&str> = items
        .iter()
        .filter_map(|i| match i {
            XpathItem::Node(n) => Some(n.extract_as_element_node().name.as_str()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"p"));
    assert!(names.contains(&"span"));
    // <div> and <a> should not be in following.
    assert!(!names.contains(&"div"));
    assert!(!names.contains(&"a"));
}

/// `following::` includes descendants of following siblings.
#[test]
fn following_axis_includes_descendants() {
    let text =
        r#"<html><body><a>1</a><div><p>inner</p></div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//a/following::p").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    let node = items[0].extract_as_node();
    assert_eq!(node.extract_as_element_node().name, "p");
}
