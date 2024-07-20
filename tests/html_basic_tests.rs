use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

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
                        div.add_element("p", |p| p.add_text("1"))
                            .add_element("p", |p| p.add_text("2"))
                            .add_element("p", |p| p.add_text("3"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
