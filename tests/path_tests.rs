use skyscraper::{html, xpath};

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
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "html");
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
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 6);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "1");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "2");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "3");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "4");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "5");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "6");
    }
}

#[test]
fn double_slash_should_select_all() {
    // arrange
    let text = r###"
        <html>
            <body>
                <div>
                    <p>1</p>
                    <p>2</p>
                    <p>3</p>
                </div>
            </body>
            <footer>
                <div>
                    <form>
                        <p>4</p>
                        <p>5</p>
                        <p>6</p>
                    </form>
                </div>
            </footer>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html/footer/div//p").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 3);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "4");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "5");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element = tree_node.data.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(tree_node.text(&xpath_item_tree), "6");
    }
}

#[test]
fn document_order_preserved_in_results() {
    // arrange
    let text = r###"
        <html>
            <body>
                <span>1</span>
                <span>
                    2
                    <span>3</span>
                </span>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html/body//span").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 3);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element_node = tree_node.data.extract_as_element_node();

        assert_eq!(element_node.name, "span");

        assert_eq!(tree_node.text(&xpath_item_tree), "1");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element_node = tree_node.data.extract_as_element_node();

        assert_eq!(element_node.name, "span");

        assert_eq!(tree_node.text(&xpath_item_tree), "2");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        let element_node = tree_node.data.extract_as_element_node();

        assert_eq!(element_node.name, "span");

        assert_eq!(tree_node.text(&xpath_item_tree), "3");
    }
}
