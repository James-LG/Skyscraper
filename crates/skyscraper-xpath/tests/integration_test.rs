use std::error::Error;

use indextree::NodeId;
use skyscraper::RNode;

static HTML: &'static str = include_str!("samples/James-LG_Skyscraper.html");

#[test]
fn xpath_github_sample1() -> Result<(), Box<dyn Error>> {
    // arrange
    let text: String = HTML.parse()?;

    let document = skyscraper_html::parse(&text).unwrap();
    let xpath = skyscraper_xpath::parse::parse("//main").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(1, nodes.len());

    let node = nodes[0];
    match document.arena.get(node).unwrap().get() {
        RNode::Tag(t) => assert_eq!("main", t.name),
        RNode::Text(_) => return Err("expected tag, got text instead".into()),
    }

    Ok(())
}

#[test]
fn xpath_github_sample2() -> Result<(), Box<dyn Error>> {
    // arrange
    let text: String = HTML.parse()?;

    let document = skyscraper_html::parse(&text).unwrap();
    let xpath = skyscraper_xpath::parse::parse("//a[@class='Link--secondary']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(5, nodes.len());

    let node = nodes[0];
    match document.arena.get(node).unwrap().get() {
        RNode::Tag(t) => {
            assert_eq!("a", t.name);

            let children: Vec<NodeId> = node.children(&document.arena).collect();
            assert_eq!(1, children.len());

            match document.arena.get(children[0]).unwrap().get() {
                RNode::Tag(_) => return Err("expected text, got tag instead".into()),
                RNode::Text(text) => assert_eq!("refactor: Reorganize into workspace", text),
            }
        },
        RNode::Text(_) => return Err("expected tag, got text instead".into()),
    }

    Ok(())
}

#[test]
fn xpath_github_sample3() -> Result<(), Box<dyn Error>> {
    // arrange
    let text: String = HTML.parse()?;

    let document = skyscraper_html::parse(&text).unwrap();
    let xpath = skyscraper_xpath::parse::parse("//div[@class='BorderGrid-cell']/div[@class=' text-small']/a").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(1, nodes.len());

    let node = nodes[0];
    match document.arena.get(node).unwrap().get() {
        RNode::Tag(t) => {
            assert_eq!("a", t.name);

            let children: Vec<NodeId> = node.children(&document.arena).collect();
            assert_eq!(1, children.len());

            match document.arena.get(children[0]).unwrap().get() {
                RNode::Tag(_) => return Err("expected text, got tag instead".into()),
                RNode::Text(text) => assert_eq!("Create a new release", text),
            }
        },
        RNode::Text(_) => return Err("expected tag, got text instead".into()),
    }

    Ok(())
}