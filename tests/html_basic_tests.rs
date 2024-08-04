use skyscraper::{
    html::{self, grammar::document_builder::DocumentBuilder},
    xpath::grammar::data_model::AttributeNode,
};

mod test_framework;

#[test]
fn parse_should_return_document() {
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

    // act
    let document = html::parse(&text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .with_root("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        {
                            div.add_element("p", |p| p.add_text("1"))
                                .add_element("p", |p| p.add_text("2"))
                                .add_element("p", |p| p.add_text("3"))
                        }
                        .add_element("div", |div| {
                            div.add_element("p", |p| p.add_text("4"))
                                .add_element("p", |p| p.add_text("5"))
                                .add_element("p", |p| p.add_text("6"))
                        })
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn text_should_include_text_before_between_and_after_child_element() {
    // arrange
    let text = r##"
        <div>
            hello
            <span>my</span>
            friend
        </div>"##;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .with_root("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_text("\n            hello\n            ")
                            .add_element("span", |span| span.add_text("my"))
                            .add_text("\n            friend\n        ")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn sample1_should_parse() {
    // arrange
    let text = r#"<html><body><div id="example">Example 1</div></body></html>"#;

    // act
    let document = html::parse(text).unwrap();
    println!("{}", document);

    // assert
    let expected = DocumentBuilder::new()
        .with_root("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_text("Example 1").add_attribute_str("id", "example")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, false));
}
