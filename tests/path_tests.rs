use skyscraper::{
    html,
    xpath::{self, grammar::XpathItemTreeNodeData},
};

#[test]
fn leading_slash_should_select_html_node() {
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html").unwrap();

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
                assert_eq!(e.name, "html")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }
    }
}

#[test]
fn leading_double_slash_should_select_all() {
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
    let xpath = xpath::parse("//p").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap().unwrap_item_set();

    // assert
    assert_eq!(nodes.len(), 6);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "1");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "2");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "3");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "4");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "5");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().unwrap_node().unwrap_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree), "6");
    }
}
