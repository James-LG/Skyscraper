use skyscraper::{
    html,
    xpath::{
        self,
        grammar::{data_model::AttributeNode, NonTreeXpathNode},
    },
};

#[test]
fn abbrev_attribute_step_should_match_given_attribute() {
    // arrange
    let text = r###"
        <html id="foo" class="bar" style="baz">
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html/@class").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let attributes: Vec<AttributeNode> = nodes
        .into_iter()
        .filter_map(|x| {
            let non_tree_node = x.unwrap_node().unwrap_non_tree_node();

            match non_tree_node {
                NonTreeXpathNode::AttributeNode(e) => Some(e),
                _ => None,
            }
        })
        .collect();

    // assert attribute
    {
        let attribute = attributes
            .iter()
            .find(|x| x.name == "class")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "bar")
    }
}
