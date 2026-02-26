use skyscraper::html;

/// Test that DOCTYPE is preserved in the parsed tree and serialized.
#[test]
fn doctype_is_preserved_in_output() {
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.starts_with("<!DOCTYPE html>"),
        "Output should start with DOCTYPE, got: {:?}",
        &output[..std::cmp::min(50, output.len())]
    );
}

/// Test that DOCTYPE with just a name is round-tripped correctly.
#[test]
fn doctype_simple_roundtrip() {
    let text = "<!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(output, text);
}

/// Test that comments before the html element are preserved.
#[test]
fn comment_before_html_is_preserved() {
    let text = "<!DOCTYPE html><!-- test comment --><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.contains("<!-- test comment -->"),
        "Output should contain the comment, got: {:?}",
        &output[..std::cmp::min(100, output.len())]
    );
}

/// Test that comment before DOCTYPE (in initial mode) is preserved.
#[test]
fn comment_in_initial_mode_is_preserved() {
    let text = "<!-- initial comment --><!DOCTYPE html><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert!(
        output.starts_with("<!-- initial comment -->"),
        "Output should start with the comment, got: {:?}",
        &output[..std::cmp::min(80, output.len())]
    );
}

/// Test that multiple comments before html are preserved in order.
#[test]
fn multiple_comments_before_html_preserved() {
    let text =
        "<!DOCTYPE html><!-- first --><!-- second --><html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(output, text);
}

/// Test that no DOCTYPE produces output without DOCTYPE.
#[test]
fn no_doctype_no_doctype_in_output() {
    let text = "<html><head></head><body></body></html>";
    let document = html::parse(text).unwrap();
    let output = document.to_string();
    assert_eq!(output, text);
}
