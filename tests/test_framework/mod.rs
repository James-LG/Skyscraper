use itertools::Itertools;
use skyscraper::xpath::{grammar::XpathItemTreeNode, XpathItemTree};

pub fn compare_documents(
    expected: XpathItemTree,
    actual: XpathItemTree,
    ignore_whitespace: bool,
) -> bool {
    let expected_root_descendants = expected.root().descendants(&expected).filter(|node| {
        if ignore_whitespace {
            if let XpathItemTreeNode::TextNode(text_node) = node {
                return !text_node.content.trim().is_empty();
            }
        }
        true
    });
    let actual_root_descendants = actual.root().descendants(&actual).filter(|node| {
        if ignore_whitespace {
            if let XpathItemTreeNode::TextNode(text_node) = node {
                return !text_node.content.trim().is_empty();
            }
        }
        true
    });

    for eb in expected_root_descendants.zip_longest(actual_root_descendants) {
        let (expected_node, actual_node) = eb.left_and_right();

        println!("Expected node: {:?}", expected_node);
        println!("Actual node: {:?}", actual_node);

        if expected_node != actual_node {
            println!(
                "Expected node display:\n{}",
                expected_node.map_or(String::new(), |n| n.display(&expected))
            );
            println!(
                "Actual node display:\n{}",
                actual_node.map_or(String::new(), |n| n.display(&actual))
            );

            println!(
                "---------------\nExpected document:\n{}\n---------------",
                expected
            );
            println!(
                "---------------\nActual document:\n{}\n---------------",
                actual
            );
            print_differences(expected_node, &expected, actual_node, &actual);
            return false;
        }
    }

    true
}

pub fn print_differences(
    expected: Option<&XpathItemTreeNode>,
    expected_doc: &XpathItemTree,
    actual: Option<&XpathItemTreeNode>,
    actual_doc: &XpathItemTree,
) {
    if expected != actual {
        println!("Expected: {:?}", expected);
        println!("Actual: {:?}", actual);

        if let Some(expected) = expected {
            print_parent("Expected", expected, expected_doc);
        }

        if let Some(actual) = actual {
            print_parent("Actual", actual, actual_doc);
        }
    }
}

fn print_parent(name: &str, node: &XpathItemTreeNode, doc: &XpathItemTree) {
    match node {
        XpathItemTreeNode::ElementNode(element) => {
            let parent = element.parent(doc);
            println!("{} Parent: {:?}", name, parent);
        }
        XpathItemTreeNode::TextNode(text) => {
            let parent = text.parent(doc);
            println!("{} Parent: {:?}", name, parent);
        }
        XpathItemTreeNode::AttributeNode(attribute) => {
            let parent = attribute.parent(doc);
            println!("{} Parent: {:?}", name, parent);
        }
        _ => {}
    }
}
