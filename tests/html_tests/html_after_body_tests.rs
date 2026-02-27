use skyscraper::html;

/// A comment token in the "after body" insertion mode should be inserted as the
/// last child of the first element in the stack of open elements (the html
/// element) per WHATWG 13.2.6.4.17.
#[test]
fn comment_after_body_inserted_in_html_element() {
    let text = "<html><head></head><body></body><!-- after body --></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- after body -->"),
        "Should contain the comment: {output:?}"
    );
    // The comment should be a child of <html>, appearing after </body>.
    assert!(
        output.contains("</body><!-- after body -->"),
        "Comment should appear after </body>: {output:?}"
    );
}

/// Multiple comments after body should all be preserved as children of the
/// html element.
#[test]
fn multiple_comments_after_body() {
    let text = "<html><head></head><body></body><!-- one --><!-- two --></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- one -->"),
        "Should contain first comment: {output:?}"
    );
    assert!(
        output.contains("<!-- two -->"),
        "Should contain second comment: {output:?}"
    );
}

/// A DOCTYPE token in the "after body" insertion mode should be treated as a
/// parse error and ignored per WHATWG 13.2.6.4.17. The parser should not panic
/// and the document should be well-formed.
#[test]
fn doctype_after_body_is_ignored() {
    let text = "<!DOCTYPE html><html><head></head><body></body><!DOCTYPE html></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    // Only the first DOCTYPE should be present.
    assert_eq!(
        output.matches("<!DOCTYPE").count(),
        1,
        "Only one DOCTYPE should survive; got: {output:?}"
    );
}

/// A comment token in the "after after body" insertion mode should be inserted
/// as the last child of the Document object per WHATWG 13.2.6.4.20.
#[test]
fn comment_after_after_body_inserted_in_document() {
    let text = "<html><head></head><body></body></html><!-- after html -->";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- after html -->"),
        "Should contain the comment: {output:?}"
    );
    // The comment should come after </html> since it's a child of Document.
    assert!(
        output.contains("</html><!-- after html -->"),
        "Comment should appear after </html>: {output:?}"
    );
}

/// Multiple comments after the closing </html> tag should all be preserved
/// as children of the Document node.
#[test]
fn multiple_comments_after_after_body() {
    let text = "<html><head></head><body></body></html><!-- a --><!-- b -->";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- a -->"),
        "Should contain first comment: {output:?}"
    );
    assert!(
        output.contains("<!-- b -->"),
        "Should contain second comment: {output:?}"
    );
}
