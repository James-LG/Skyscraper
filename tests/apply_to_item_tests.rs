use skyscraper::{html, xpath};

#[test]
fn apply_to_item_should_select_given_node() {
    /* Stage 0: Document setup. */
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);

    /* Stage 1: Select a node. */
    // arrange
    let xpath = xpath::parse("/html/body").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let first_expr_item = nodes.into_iter().next().unwrap();

    /* Stage 2: Apply context expression to selected node. */
    // arrange
    let xpath = xpath::parse(".").unwrap();

    // act
    let nodes = xpath
        .apply_to_item(&xpath_item_tree, first_expr_item.clone())
        .unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let second_expr_item = nodes.into_iter().next().unwrap();

    assert_eq!(first_expr_item, second_expr_item);

    // assert node
    let tree_node = second_expr_item
        .extract_into_node()
        .extract_into_tree_node();

    let element = tree_node.data.extract_as_element_node();
    assert_eq!(element.name, "body")
}
