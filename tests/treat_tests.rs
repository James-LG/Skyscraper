use skyscraper::{html, xpath};

#[test]
fn treat_correct_type_should_not_fail() {
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html treat as node()").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
}

#[test]
fn treat_incorrect_type_should_fail() {
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html treat as document-node()").unwrap();

    // act
    let err = xpath.apply(&xpath_item_tree).unwrap_err();

    // assert
    assert_eq!(
        err.to_string(),
        "Error applying expression err:XPDY0050 Cannot treat as document-node()"
    );
}
