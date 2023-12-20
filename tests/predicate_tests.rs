use skyscraper::{
    html,
    xpath::{self, grammar::XpathItemTreeNodeData},
};

#[test]
fn class_equals_predicate_should_select_nodes_with_that_match() {
    // arrange
    let text = r###"
        <html>
            <div>
                bad
            </div>
            <div class="here">
                good
            </div>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from_html_document(&document);
    let xpath = xpath::parse("/html/div[@class='here']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap().unwrap_item_set();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "div")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "good");
    }
}

#[test]
fn predicate_on_double_leading_slash_should_select_nodes_with_that_match() {
    // arrange
    let text = r###"
        <html>
            <div>
                bad
            </div>
            <div class="here">
                good
            </div>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from_html_document(&document);
    let xpath = xpath::parse("//div[@class='here']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap().unwrap_item_set();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "div")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "good");
    }
}
