use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, XpathItem},
    },
};

/// Basic let expression binding a literal value.
/// `let $x := 4 return $x` should return 4.
#[test]
fn let_expr_single_literal() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := 4 return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(4))
    );
}

/// Let expression with arithmetic in the return clause.
/// `let $x := 4, $y := 3 return $x + $y` should return 7.
#[test]
fn let_expr_two_bindings_arithmetic() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := 4, $y := 3 return $x + $y").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(7))
    );
}

/// Let expression binding a node sequence.
/// `let $x := //li return $x` should return all li elements.
#[test]
fn let_expr_node_sequence() {
    let text = r#"<html><body><ul><li>a</li><li>b</li><li>c</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := //li return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3, "let $x := //li should bind all 3 li elements: {items:?}");
}

/// Let expression with dependent bindings.
/// `let $x := //ul, $y := $x/li return $y` should return all li elements.
#[test]
fn let_expr_dependent_bindings() {
    let text = r#"<html><body>
        <ul><li>a</li><li>b</li></ul>
        <ul><li>c</li></ul>
    </body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := //ul, $y := $x/li return $y").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(
        items.len(),
        3,
        "should return all 3 li elements from both ul: {items:?}"
    );
}

/// Let expression returning text from bound nodes.
/// `let $x := //li return $x/text()` should return text nodes.
#[test]
fn let_expr_return_text() {
    let text = r#"<html><body><ul><li>alpha</li><li>beta</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := //li return $x/text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 text nodes: {items:?}");

    let texts: Vec<String> = items
        .iter()
        .map(|item| {
            item.extract_as_node()
                .extract_as_text_node()
                .content
                .to_string()
        })
        .collect();
    assert_eq!(texts, vec!["alpha", "beta"]);
}

/// Let expression with empty binding.
/// `let $x := //nonexistent return $x` should return empty.
#[test]
fn let_expr_empty_binding() {
    let text = r#"<html><body><div>content</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := //nonexistent return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "empty binding should produce empty result: {items:?}");
}

/// Let expression returning attributes from bound nodes.
/// `let $x := //div return $x/@class`
#[test]
fn let_expr_return_attribute() {
    let text =
        r#"<html><body><div class="a">1</div><div class="b">2</div></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := //div return $x/@class").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 attribute nodes: {items:?}");
}

/// Let expression with literal sequence binding.
/// `let $x := (1, 2, 3) return $x` should return 3 items.
#[test]
fn let_expr_literal_sequence() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := (1, 2, 3) return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 3, "let over (1,2,3) should return 3 items: {items:?}");
}

/// Let + for combined: let binds a sequence, for iterates over it.
/// `let $items := //li return for $x in $items return $x/text()`
#[test]
fn let_expr_with_for() {
    let text = r#"<html><body><ul><li>one</li><li>two</li></ul></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse("let $items := //li return for $x in $items return $x/text()").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 2, "should return 2 text nodes: {items:?}");

    let texts: Vec<String> = items
        .iter()
        .map(|item| {
            item.extract_as_node()
                .extract_as_text_node()
                .content
                .to_string()
        })
        .collect();
    assert_eq!(texts, vec!["one", "two"]);
}

/// Variable shadowing: inner let should shadow outer let.
/// `let $x := 1 return let $x := 2 return $x` should return 2.
#[test]
fn let_expr_variable_shadowing() {
    let text = r#"<html><body></body></html>"#;

    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("let $x := 1 return let $x := 2 return $x").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1, "should return 1 item: {items:?}");
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(2))
    );
}
