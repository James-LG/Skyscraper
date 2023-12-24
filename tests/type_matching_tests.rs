use skyscraper::{
    html,
    xpath::{self, grammar::XpathItemTreeNodeData},
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
