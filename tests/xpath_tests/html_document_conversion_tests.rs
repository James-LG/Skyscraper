//! Tests for the `From<&HtmlDocument>` conversion to `XpathItemTree`.
//!
//! The `html::parse()` and `DocumentBuilder` paths build `XpathItemTree` directly.
//! These tests exercise the separate `From<&HtmlDocument>` conversion, which is the
//! public API for users who programmatically build an `HtmlDocument` and then want
//! to query it with XPath.

use indextree::Arena;
use skyscraper::{
    html::{
        DocumentNode, HtmlComment, HtmlDoctype, HtmlDocument, HtmlNode, HtmlProcessingInstruction,
        HtmlTag, HtmlText,
    },
    xpath,
    xpath::grammar::{data_model::XpathItem, XpathItemTree, XpathItemTreeNode},
};

// ── Comment nodes ───────────────────────────────────────────────────────────

#[test]
fn converts_comment_node() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("div".to_string())));
    let comment = arena.new_node(HtmlNode::Comment(HtmlComment::new(" hello ".to_string())));
    root.append(comment, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(root));
    let tree = XpathItemTree::from(&doc);

    let xpath = xpath::parse("//comment()").unwrap();
    let items = xpath.apply(&tree).unwrap();
    assert_eq!(items.len(), 1, "should find 1 comment node");

    let node = items[0].as_node().unwrap().as_comment_node().unwrap();
    assert_eq!(node.content, " hello ");
}

#[test]
fn converts_multiple_comment_nodes() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("div".to_string())));
    let c1 = arena.new_node(HtmlNode::Comment(HtmlComment::new("first".to_string())));
    let c2 = arena.new_node(HtmlNode::Comment(HtmlComment::new("second".to_string())));
    root.append(c1, &mut arena);
    root.append(c2, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(root));
    let tree = XpathItemTree::from(&doc);

    let xpath = xpath::parse("//comment()").unwrap();
    let items = xpath.apply(&tree).unwrap();
    assert_eq!(items.len(), 2, "should find 2 comment nodes");
}

#[test]
fn comment_string_value_via_conversion() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("root".to_string())));
    let comment = arena.new_node(HtmlNode::Comment(HtmlComment::new("content".to_string())));
    root.append(comment, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(root));
    let tree = XpathItemTree::from(&doc);

    let xpath = xpath::parse("string(//comment())").unwrap();
    let items = xpath.apply(&tree).unwrap();
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(skyscraper::xpath::grammar::data_model::AnyAtomicType::String(
            "content".to_string()
        ))
    );
}

// ── Processing instruction nodes ────────────────────────────────────────────

#[test]
fn converts_pi_node() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("root".to_string())));
    let pi = arena.new_node(HtmlNode::ProcessingInstruction(
        HtmlProcessingInstruction::new("xml-stylesheet".to_string(), "type=\"text/xsl\"".to_string()),
    ));
    root.append(pi, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(root));
    let tree = XpathItemTree::from(&doc);

    let xpath = xpath::parse("//processing-instruction()").unwrap();
    let items = xpath.apply(&tree).unwrap();
    assert_eq!(items.len(), 1, "should find 1 PI node");

    let node = items[0].as_node().unwrap().as_pi_node().unwrap();
    assert_eq!(node.target, "xml-stylesheet");
    assert_eq!(node.data, "type=\"text/xsl\"");
}

#[test]
fn converts_pi_node_named_match() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("root".to_string())));
    let pi1 = arena.new_node(HtmlNode::ProcessingInstruction(
        HtmlProcessingInstruction::new("php".to_string(), "echo 'hi';".to_string()),
    ));
    let pi2 = arena.new_node(HtmlNode::ProcessingInstruction(
        HtmlProcessingInstruction::new("other".to_string(), "data".to_string()),
    ));
    root.append(pi1, &mut arena);
    root.append(pi2, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(root));
    let tree = XpathItemTree::from(&doc);

    let xpath = xpath::parse("//processing-instruction(php)").unwrap();
    let items = xpath.apply(&tree).unwrap();
    assert_eq!(items.len(), 1, "should match only the php PI");
}

// ── Doctype nodes ───────────────────────────────────────────────────────────

#[test]
fn converts_doctype_node() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("html".to_string())));
    let doctype = arena.new_node(HtmlNode::Doctype(HtmlDoctype::new(
        "html".to_string(),
        None,
        None,
    )));

    // Build a document with doctype as root, html as sibling (mimicking real structure).
    // We need a wrapper node to hold both doctype and html as siblings.
    let wrapper = arena.new_node(HtmlNode::Tag(HtmlTag::new("wrapper".to_string())));
    wrapper.append(doctype, &mut arena);
    wrapper.append(root, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(wrapper));
    let tree = XpathItemTree::from(&doc);

    // DoctypeNode exists in the tree but is excluded from node() per XPath 3.1 spec.
    let doctype_nodes: Vec<_> = tree
        .iter()
        .filter(|node| matches!(node, XpathItemTreeNode::DoctypeNode(_)))
        .collect();
    assert_eq!(doctype_nodes.len(), 1, "doctype should be in the tree");

    // node() should not return it.
    let xpath = xpath::parse("//node()").unwrap();
    let items = xpath.apply(&tree).unwrap();
    let has_doctype = items
        .iter()
        .any(|item| matches!(item.as_node(), Ok(XpathItemTreeNode::DoctypeNode(_))));
    assert!(!has_doctype, "node() should exclude DoctypeNode");
}

#[test]
fn converts_doctype_with_public_and_system_ids() {
    let mut arena = Arena::new();
    let doctype = arena.new_node(HtmlNode::Doctype(HtmlDoctype::new(
        "html".to_string(),
        Some("-//W3C//DTD XHTML 1.0 Strict//EN".to_string()),
        Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd".to_string()),
    )));

    let doc = HtmlDocument::new(arena, DocumentNode::new(doctype));
    let tree = XpathItemTree::from(&doc);

    let doctype_node = tree
        .iter()
        .find_map(|node| {
            if let XpathItemTreeNode::DoctypeNode(d) = node {
                Some(d.clone())
            } else {
                None
            }
        })
        .expect("doctype should exist in tree");

    assert_eq!(doctype_node.name, "html");
    assert_eq!(
        doctype_node.public_id.as_deref(),
        Some("-//W3C//DTD XHTML 1.0 Strict//EN")
    );
    assert_eq!(
        doctype_node.system_id.as_deref(),
        Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd")
    );
}

// ── Mixed node types ────────────────────────────────────────────────────────

#[test]
fn converts_mixed_node_types() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("div".to_string())));
    let text = arena.new_node(HtmlNode::Text(HtmlText::from_str("hello")));
    let comment = arena.new_node(HtmlNode::Comment(HtmlComment::new("a comment".to_string())));
    let pi = arena.new_node(HtmlNode::ProcessingInstruction(
        HtmlProcessingInstruction::new("target".to_string(), "data".to_string()),
    ));
    root.append(text, &mut arena);
    root.append(comment, &mut arena);
    root.append(pi, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(root));
    let tree = XpathItemTree::from(&doc);

    // All three child types should be present.
    let comments = xpath::parse("//comment()").unwrap();
    assert_eq!(comments.apply(&tree).unwrap().len(), 1);

    let pis = xpath::parse("//processing-instruction()").unwrap();
    assert_eq!(pis.apply(&tree).unwrap().len(), 1);

    // text() should find the text node but not comment/PI content.
    let texts = xpath::parse("//div/text()").unwrap();
    let text_items = texts.apply(&tree).unwrap();
    assert_eq!(text_items.len(), 1);
}

#[test]
fn comment_has_correct_parent_via_conversion() {
    let mut arena = Arena::new();
    let root = arena.new_node(HtmlNode::Tag(HtmlTag::new("section".to_string())));
    let comment = arena.new_node(HtmlNode::Comment(HtmlComment::new("note".to_string())));
    root.append(comment, &mut arena);

    let doc = HtmlDocument::new(arena, DocumentNode::new(root));
    let tree = XpathItemTree::from(&doc);

    let xpath = xpath::parse("//comment()").unwrap();
    let items = xpath.apply(&tree).unwrap();
    let comment_node = items[0].as_node().unwrap().as_comment_node().unwrap();
    let parent = comment_node.parent(&tree).unwrap();
    let parent_element = parent.as_element_node().unwrap();
    assert_eq!(parent_element.name, "section");
}
