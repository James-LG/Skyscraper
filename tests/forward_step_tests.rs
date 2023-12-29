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
        .map(|item| {
            item.extract_into_node()
                .extract_into_non_tree_node()
                .extract_into_attribute_node()
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
