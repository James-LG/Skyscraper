use skyscraper::{
    html,
    xpath::{
        self,
        grammar::{data_model::AttributeNode, NonTreeXpathNode, XpathItemTreeNodeData},
    },
};

#[test]
fn text_test_should_match_all_text() {
    // arrange
    let text = r###"
        <html>
            <body>
                <div>
                    <p>1</p>
                    <p>2</p>
                    <p>3</p>
                </div>
                <div>
                    <p>4</p>
                    <p>5</p>
                    <p>6</p>
                </div>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//text()").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    // filter out whitespace nodes before testing since they vary by HTML parser.
    let nodes: Vec<_> = nodes
        .into_iter()
        .filter(|n| match n.unwrap_node_ref().unwrap_tree_node_ref().data {
            XpathItemTreeNodeData::TextNode(text) => !text.only_whitespace,
            _ => true,
        })
        .collect();

    assert_eq!(nodes.len(), 6);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::TextNode(e) => {
                assert_eq!(e.content, "1")
            }
            _ => panic!("expected text, got {:?}", tree_node.data),
        }
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::TextNode(e) => {
                assert_eq!(e.content, "2")
            }
            _ => panic!("expected text, got {:?}", tree_node.data),
        }
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::TextNode(e) => {
                assert_eq!(e.content, "3")
            }
            _ => panic!("expected text, got {:?}", tree_node.data),
        }
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::TextNode(e) => {
                assert_eq!(e.content, "4")
            }
            _ => panic!("expected text, got {:?}", tree_node.data),
        }
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::TextNode(e) => {
                assert_eq!(e.content, "5")
            }
            _ => panic!("expected text, got {:?}", tree_node.data),
        }
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::TextNode(e) => {
                assert_eq!(e.content, "6")
            }
            _ => panic!("expected text, got {:?}", tree_node.data),
        }
    }
}

#[test]
fn attribute_test_should_match_all_attributes() {
    // arrange
    let text = r###"
        <html id="foo" class="bar" style="baz">
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html/attribute::attribute()").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 3);
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
            .find(|x| x.name == "id")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "foo")
    }

    // assert attribute
    {
        let attribute = attributes
            .iter()
            .find(|x| x.name == "class")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "bar")
    }

    // assert attribute
    {
        let attribute = attributes
            .iter()
            .find(|x| x.name == "style")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "baz")
    }
}
