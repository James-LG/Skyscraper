use skyscraper::xpath::grammar::XpathItemTreeNode;
use skyscraper::{html, xpath};

static HTML: &'static str = include_str!("samples/James-LG_Skyscraper.html");

#[test]
fn xpath_github_sample1() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//main").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes
        .next()
        .unwrap()
        .extract_into_node()
        .extract_into_tree_node();

    let element = tree_node.data.extract_as_element_node();
    assert_eq!(element.name, "main")
}

#[test]
fn xpath_github_sample2() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//a[@class='Link--secondary']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 5);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes
        .next()
        .unwrap()
        .extract_into_node()
        .extract_into_tree_node();

    let element = tree_node.data.extract_as_element_node();
    assert_eq!(element.name, "a");

    let children: Vec<XpathItemTreeNode> = tree_node.children(&xpath_item_tree).collect();
    assert_eq!(1, children.len());
    let mut children = children.into_iter();

    let text = children.next().unwrap().data.extract_as_text_node();
    assert_eq!(text.content, "refactor: Reorganize into workspace")
}

#[test]
fn xpath_github_sample3() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath =
        xpath::parse("//div[@class='BorderGrid-cell']/div[@class=' text-small']/a").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes
        .next()
        .unwrap()
        .extract_into_node()
        .extract_into_tree_node();

    let element = tree_node.data.extract_as_element_node();
    assert_eq!(element.name, "a");

    let children: Vec<XpathItemTreeNode> = tree_node.children(&xpath_item_tree).collect();
    assert_eq!(children.len(), 1);
    let mut children = children.into_iter();

    let text = children.next().unwrap().data.extract_as_text_node();
    assert_eq!(text.content, "Create a new release");
}

#[test]
fn xpath_github_sample4() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//div[@role='gridcell']/descendant-or-self::node()").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 100);
}

#[test]
fn xpath_github_get_text_sample() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//div[@class='flex-auto min-width-0 width-fit mr-3']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let element = nodes
        .next()
        .unwrap()
        .extract_into_node()
        .extract_into_tree_node();

    let text = element.all_text(&xpath_item_tree).trim().to_string();

    assert_eq!(text, "James-LG / Skyscraper Public");
}

#[test]
fn xpath_github_parent_axis() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//div[@role='gridcell']/parent::div").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 5);
}

#[test]
fn xpath_github_parent_axis_recursive() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//div[@role='gridcell']//parent::div").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 20);
}

#[test]
fn xpath_github_dashed_attribute() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//span[@data-view-component='true']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 19);
}

#[test]
fn xpath_github_get_attributes_sample() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//div[@class='flex-auto min-width-0 width-fit mr-3']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes
        .next()
        .unwrap()
        .extract_into_node()
        .extract_into_tree_node();
    let elem = tree_node.data.extract_as_element_node();

    assert_eq!(
        elem.get_attribute("class").unwrap(),
        "flex-auto min-width-0 width-fit mr-3"
    );
}

#[test]
fn xpath_github_root_search() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes
        .next()
        .unwrap()
        .extract_into_node()
        .extract_into_tree_node();

    let element = tree_node.data.extract_as_element_node();
    assert_eq!(element.name, "html");
}

#[test]
fn xpath_github_root_search_all() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//html").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes
        .next()
        .unwrap()
        .extract_into_node()
        .extract_into_tree_node();

    let element = tree_node.data.extract_as_element_node();
    assert_eq!(element.name, "html")
}

#[test]
fn xpath_github_root_wildcard() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//body/*").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 16);

    // assert first node
    let tree_node = &nodes[0].extract_as_node().extract_as_tree_node();
    let elem = tree_node.data.extract_as_element_node();

    assert_eq!(elem.name, "div");

    assert_eq!(
        elem.get_attribute("class").unwrap(),
        "position-relative js-header-wrapper "
    );

    // assert random node 4
    let tree_node = &nodes[3].extract_as_node().extract_as_tree_node();
    let elem = tree_node.data.extract_as_element_node();

    assert_eq!(elem.name, "include-fragment");

    // assert last node
    let tree_node = &nodes[15].extract_as_node().extract_as_tree_node();
    let elem = tree_node.data.extract_as_element_node();

    assert_eq!(elem.name, "div");

    assert_eq!(elem.get_attribute("class").unwrap(), "sr-only");
    assert_eq!(elem.get_attribute("aria-live").unwrap(), "polite")
}
