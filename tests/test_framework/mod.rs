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

    for (expected_node, actual_node) in expected_root_descendants.zip(actual_root_descendants) {
        if expected_node != actual_node {
            print_differences(expected_node, &expected, actual_node, &actual);
            return false;
        }
    }

    true
}

pub fn print_differences(
    expected: &XpathItemTreeNode,
    expected_doc: &XpathItemTree,
    actual: &XpathItemTreeNode,
    actual_doc: &XpathItemTree,
) {
    if expected != actual {
        println!("Expected: {:?}", expected);
        println!("Actual: {:?}", actual);

        if let XpathItemTreeNode::ElementNode(expected) = expected {
            let expected_parent = expected.parent(expected_doc);
            println!("Expected parent: {:?}", expected_parent);
        }

        if let XpathItemTreeNode::ElementNode(actual) = actual {
            let actual_parent = actual.parent(actual_doc);
            println!("Actual parent: {:?}", actual_parent);
        }
    }
}
