use std::error::Error;

use skyscraper::html::{HtmlNode, DocumentNode};
use skyscraper::{html, xpath};


static HTML: &'static str = include_str!("samples/James-LG_Skyscraper.html");

#[test]
fn xpath_github_sample1() -> Result<(), Box<dyn Error>> {
    // arrange
    let text: String = HTML.parse()?;

    let document = html::parse(&text).unwrap();
    let xpath = skyscraper::xpath::parse::parse("//main").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(1, nodes.len());

    let doc_node = nodes[0];
    match document.get_html_node(&doc_node).unwrap() {
        HtmlNode::Tag(t) => assert_eq!("main", t.name),
        HtmlNode::Text(_) => return Err("expected tag, got text instead".into()),
    }

    Ok(())
}

#[test]
fn xpath_github_sample2() -> Result<(), Box<dyn Error>> {
    // arrange
    let text: String = HTML.parse()?;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse::parse("//a[@class='Link--secondary']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(5, nodes.len());

    let doc_node = nodes[0];
    match document.get_html_node(&doc_node).unwrap() {
        HtmlNode::Tag(t) => {
            assert_eq!("a", t.name);

            let children: Vec<DocumentNode> = doc_node.children(&document).collect();
            assert_eq!(1, children.len());

            match document.get_html_node(&children[0]).unwrap() {
                HtmlNode::Tag(_) => return Err("expected text, got tag instead".into()),
                HtmlNode::Text(text) => assert_eq!("refactor: Reorganize into workspace", text),
            }
        },
        HtmlNode::Text(_) => return Err("expected tag, got text instead".into()),
    }

    Ok(())
}

#[test]
fn xpath_github_sample3() -> Result<(), Box<dyn Error>> {
    // arrange
    let text: String = HTML.parse()?;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[@class='BorderGrid-cell']/div[@class=' text-small']/a").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(1, nodes.len());

    let doc_node = nodes[0];
    match document.get_html_node(&doc_node).unwrap() {
        HtmlNode::Tag(t) => {
            assert_eq!("a", t.name);

            let children: Vec<DocumentNode> = doc_node.children(&document).collect();
            assert_eq!(1, children.len());

            match document.get_html_node(&children[0]).unwrap() {
                HtmlNode::Tag(_) => return Err("expected text, got tag instead".into()),
                HtmlNode::Text(text) => assert_eq!("Create a new release", text),
            }
        },
        HtmlNode::Text(_) => return Err("expected tag, got text instead".into()),
    }

    Ok(())
}

#[test]
fn xpath_github_get_text_sample() -> Result<(), Box<dyn Error>> {
    // arrange
    let text: String = HTML.parse()?;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[@class='flex-auto min-width-0 width-fit mr-3']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(1, nodes.len());

    let doc_node = nodes[0];
    let html_node = document.get_html_node(&doc_node).unwrap();
    let text = html_node.get_all_text(&doc_node, &document).unwrap();

    assert_eq!("James-LG / Skyscraper Public", text);

    Ok(())
}