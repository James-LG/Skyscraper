use skyscraper::{html, xpath};

/// The `or` operator should return nodes matching either condition
/// (XPath 3.1 section 3.6).
#[test]
fn or_operator_matches_either_condition() {
    let text = r#"<html><body>
        <div class="a">first</div>
        <div class="b">second</div>
        <div class="c">third</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class='a' or @class='b']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 2, "should match two divs: {nodes:?}");
}

/// The `or` operator should return true if the first operand is true
/// (XPath 3.1 section 3.6).
#[test]
fn or_operator_returns_true_for_first_true() {
    let text = r#"<html><body>
        <div class="a">found</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class='a' or @class='nonexistent']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "should match when first condition is true: {nodes:?}");
}

/// The `or` operator should return true if only the second operand is true
/// (XPath 3.1 section 3.6).
#[test]
fn or_operator_returns_true_for_second_true() {
    let text = r#"<html><body>
        <div class="a">found</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class='nonexistent' or @class='a']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "should match when second condition is true: {nodes:?}");
}

/// The `or` operator should return false when neither operand is true
/// (XPath 3.1 section 3.6).
#[test]
fn or_operator_returns_false_when_both_false() {
    let text = r#"<html><body>
        <div class="a">found</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class='x' or @class='y']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 0, "should match nothing when both false: {nodes:?}");
}

/// The `and` operator should return nodes matching both conditions
/// (XPath 3.1 section 3.6).
#[test]
fn and_operator_matches_both_conditions() {
    let text = r#"<html><body>
        <div class="a" id="1">both</div>
        <div class="a">only class</div>
        <div id="1">only id</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class='a' and @id='1']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 1, "should match only the div with both attributes: {nodes:?}");
}

/// The `and` operator should return false when the first operand is false
/// (XPath 3.1 section 3.6).
#[test]
fn and_operator_returns_false_when_first_false() {
    let text = r#"<html><body>
        <div class="a" id="1">match</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class='nonexistent' and @id='1']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 0, "should match nothing when first is false: {nodes:?}");
}

/// The `and` operator should return false when the second operand is false
/// (XPath 3.1 section 3.6).
#[test]
fn and_operator_returns_false_when_second_false() {
    let text = r#"<html><body>
        <div class="a" id="1">match</div>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("//div[@class='a' and @id='nonexistent']").unwrap();

    let nodes = xpath.apply(&document).unwrap();
    assert_eq!(nodes.len(), 0, "should match nothing when second is false: {nodes:?}");
}
