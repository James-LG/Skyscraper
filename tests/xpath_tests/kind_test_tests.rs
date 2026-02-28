use skyscraper::{
    html,
    html::grammar::document_builder::DocumentBuilder,
    xpath,
    xpath::grammar::data_model::{AnyAtomicType, XpathItem},
};

/// `element()` with no arguments matches any element.
#[test]
fn element_test_no_args_matches_all_elements() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match div and span: {items:?}");
}

/// `element(div)` matches only div elements.
#[test]
fn element_test_named_matches_specific_element() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element(div)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match only div: {items:?}");
}

/// `element(*)` matches any element (same as no args).
#[test]
fn element_test_wildcard_matches_all() {
    let text = r#"<html><body><div>a</div><span>b</span></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element(*)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match div and span: {items:?}");
}

/// `element(nonexistent)` matches nothing.
#[test]
fn element_test_named_no_match() {
    let text = r#"<html><body><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/element(section)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should match nothing: {items:?}");
}

/// `comment()` matches comment nodes.
#[test]
fn comment_test_matches_comments() {
    let text = r#"<html><body><!-- hello --><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//body/comment()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match 1 comment: {items:?}");
}

/// `comment()` string value is the comment's content (XPath 3.1 §6.7.6).
#[test]
fn comment_node_string_value() {
    let text = r#"<html><body><!-- hello world --><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("string(//body/comment())").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(" hello world ".to_string())),
        "comment string value should be its content"
    );
}

/// `comment()` parent axis navigates to the containing element.
#[test]
fn comment_node_parent_navigation() {
    let text = r#"<html><body><!-- hello --><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("name(//comment()/parent::*)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("body".to_string())),
        "comment parent should be 'body'"
    );
}

/// `comment()` ancestor axis navigates up the tree.
#[test]
fn comment_node_ancestor_navigation() {
    let text = r#"<html><body><div><!-- inside div --></div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("count(//comment()/ancestor::*)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(3)),
        "comment should have 3 ancestors: div, body, html"
    );
}

/// `data()` atomization of a comment node returns its content.
#[test]
fn comment_node_data_atomization() {
    let text = r#"<html><body><!-- atomize me --><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("data(//body/comment())").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(" atomize me ".to_string())),
        "data() on comment should return its content"
    );
}

/// `namespace-node()` returns empty in HTML context.
#[test]
fn namespace_node_test_returns_empty() {
    let text = r#"<html><body><div>a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div/namespace-node()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "HTML has no namespace nodes: {items:?}");
}

/// `attribute(id)` matches only the id attribute.
#[test]
fn attribute_test_named_matches_specific() {
    let text = r#"<html><body><div id="foo" class="bar">a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div/attribute(id)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match only id attribute: {items:?}");
}

/// `attribute(*)` matches all attributes.
#[test]
fn attribute_test_wildcard_matches_all() {
    let text = r#"<html><body><div id="foo" class="bar">a</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div/attribute(*)").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should match both attributes: {items:?}");
}

/// `processing-instruction()` display formatting.
#[test]
fn pi_test_display() {
    let xpath = xpath::parse("//processing-instruction()").unwrap();
    assert_eq!(xpath.to_string(), "//processing-instruction()");
}

/// `processing-instruction(name)` display formatting.
#[test]
fn pi_test_named_display() {
    let xpath = xpath::parse("//processing-instruction(xml)").unwrap();
    assert_eq!(xpath.to_string(), "//processing-instruction(xml)");
}

/// `schema-attribute(name)` display formatting.
#[test]
fn schema_attribute_test_display() {
    let xpath = xpath::parse("//schema-attribute(price)").unwrap();
    assert_eq!(xpath.to_string(), "//schema-attribute(price)");
}

/// `document-node()` matches the root document node via `instance of`.
#[test]
fn document_node_test_matches_root() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    // fn:root() returns the document node; check it's a document-node()
    let xpath = xpath::parse("fn:root(.) instance of document-node()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)),
        "root should be a document-node()"
    );
}

/// `document-node(element(html))` matches a document whose single element child is `html`.
#[test]
fn document_node_element_test_matches_html() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("fn:root(.) instance of document-node(element(html))").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)),
        "document should match document-node(element(html))"
    );
}

/// `document-node(element(wrong))` does NOT match when element name doesn't match.
#[test]
fn document_node_element_test_wrong_name() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("fn:root(.) instance of document-node(element(wrong))").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false)),
        "document should NOT match document-node(element(wrong))"
    );
}

/// `document-node(element(*))` matches any document with a single element child.
#[test]
fn document_node_element_wildcard_matches() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("fn:root(.) instance of document-node(element(*))").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true)),
        "document should match document-node(element(*))"
    );
}

/// `processing-instruction()` matches PI nodes.
#[test]
fn pi_test_matches_pi_nodes() {
    let document = DocumentBuilder::new()
        .add_element("root", |e| {
            e.add_processing_instruction("xml-stylesheet", "type=\"text/xsl\"")
                .add_text("hello")
        })
        .build()
        .unwrap();

    let xpath = xpath::parse("//processing-instruction()").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match 1 PI node: {items:?}");
}

/// `processing-instruction(name)` matches PIs with the specified target.
#[test]
fn pi_test_named_matches_target() {
    let document = DocumentBuilder::new()
        .add_element("root", |e| {
            e.add_processing_instruction("xml-stylesheet", "type=\"text/xsl\"")
                .add_processing_instruction("php", "echo 'hello';")
        })
        .build()
        .unwrap();

    let xpath = xpath::parse("//processing-instruction(php)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should match only the php PI: {items:?}");
}

/// `processing-instruction(name)` doesn't match PIs with different targets.
#[test]
fn pi_test_named_no_match() {
    let document = DocumentBuilder::new()
        .add_element("root", |e| {
            e.add_processing_instruction("xml-stylesheet", "type=\"text/xsl\"")
        })
        .build()
        .unwrap();

    let xpath = xpath::parse("//processing-instruction(php)").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "should match nothing: {items:?}");
}

/// PI node string value is its data content.
#[test]
fn pi_node_string_value() {
    let document = DocumentBuilder::new()
        .add_element("root", |e| {
            e.add_processing_instruction("xml-stylesheet", "type=\"text/xsl\"")
        })
        .build()
        .unwrap();

    let xpath = xpath::parse("string(//processing-instruction())").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(
            "type=\"text/xsl\"".to_string()
        ))
    );
}

/// PI node name returns its target.
#[test]
fn pi_node_name_value() {
    let document = DocumentBuilder::new()
        .add_element("root", |e| {
            e.add_processing_instruction("xml-stylesheet", "type=\"text/xsl\"")
        })
        .build()
        .unwrap();

    let xpath = xpath::parse("name(//processing-instruction())").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(
            "xml-stylesheet".to_string()
        ))
    );
}

/// PI node with empty data returns empty string.
#[test]
fn pi_node_empty_data_string_value() {
    let document = DocumentBuilder::new()
        .add_element("root", |e| {
            e.add_processing_instruction("target", "")
        })
        .build()
        .unwrap();

    let xpath = xpath::parse("string(//processing-instruction())").unwrap();
    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(String::new()))
    );
}

/// `node()` should NOT match DoctypeNode (DOCTYPE is not a valid XPath 3.1 node type).
#[test]
fn node_test_excludes_doctype() {
    // Parse HTML with DOCTYPE — the direct path creates a DoctypeNode child of the document.
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();

    // /node() selects the document's children that are XPath node types.
    // DOCTYPE should be excluded; only the <html> element should match.
    let xpath = xpath::parse("/node()").unwrap();
    let items = xpath.apply(&document).unwrap();

    assert_eq!(items.len(), 1, "node() should return only the html element, not the doctype");
    let node = items[0].as_node().unwrap();
    let element = node.as_element_node().unwrap();
    assert_eq!(element.name, "html");
}

/// `node()` should not match DoctypeNode even in a DocumentBuilder tree.
#[test]
fn node_test_excludes_doctype_in_builder() {
    let document = DocumentBuilder::new()
        .add_doctype("html")
        .add_element("html", |e| {
            e.add_element("body", |e| e.add_text("hello"))
        })
        .build()
        .unwrap();

    let xpath = xpath::parse("/node()").unwrap();
    let items = xpath.apply(&document).unwrap();

    // Only the html element should match, not the doctype.
    assert_eq!(items.len(), 1, "should match only the html element: {items:?}");
    let node = items[0].as_node().unwrap();
    let element = node.as_element_node().unwrap();
    assert_eq!(element.name, "html");
}

/// DoctypeNode should have a working parent() — it should navigate to the document node.
#[test]
fn doctype_node_has_parent() {
    use skyscraper::xpath::grammar::XpathItemTreeNode;

    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();

    // Find the DoctypeNode by iterating the tree.
    let doctype = document.iter().find(|node| {
        matches!(node, XpathItemTreeNode::DoctypeNode(_))
    });

    assert!(doctype.is_some(), "DoctypeNode should exist in the tree");
    let doctype = doctype.unwrap();
    let parent = doctype.parent(&document);
    assert!(parent.is_some(), "DoctypeNode should have a parent");
    assert!(
        matches!(parent.unwrap(), XpathItemTreeNode::DocumentNode(_)),
        "DoctypeNode parent should be the document node"
    );
}

/// count(/node()) should not count the DoctypeNode.
#[test]
fn count_node_excludes_doctype() {
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();

    let xpath = xpath::parse("count(/node())").unwrap();
    let items = xpath.apply(&document).unwrap();

    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(1)),
        "count(/node()) should be 1 (only the html element, not the doctype)"
    );
}
