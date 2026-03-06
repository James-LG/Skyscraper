use skyscraper::{
    html,
    xpath::{
        self,
        grammar::data_model::{AnyAtomicType, Function, XpathItem},
    },
};

// ============================================================
// fn:QName
// ============================================================

#[test]
fn qname_with_prefix() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"QName("http://www.w3.org/2005/xpath-functions", "fn:contains")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::QName {
            namespace_uri: "http://www.w3.org/2005/xpath-functions".to_string(),
            local_name: "contains".to_string(),
            prefix: Some("fn".to_string()),
        })
    );
}

#[test]
fn qname_without_prefix() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"QName("http://example.com", "localname")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::QName {
            namespace_uri: "http://example.com".to_string(),
            local_name: "localname".to_string(),
            prefix: None,
        })
    );
}

#[test]
fn qname_empty_namespace() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"QName("", "localname")"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::QName {
            namespace_uri: String::new(),
            local_name: "localname".to_string(),
            prefix: None,
        })
    );
}

#[test]
fn qname_prefix_with_empty_namespace_is_error() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"QName("", "p:localname")"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "prefix with empty namespace should error");
}

#[test]
fn qname_string_value() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"string(QName("http://www.w3.org/2005/xpath-functions", "fn:contains"))"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("fn:contains".to_string()))
    );
}

#[test]
fn qname_string_value_no_prefix() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"string(QName("http://example.com", "name"))"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("name".to_string()))
    );
}

// ============================================================
// fn:local-name-from-QName
// ============================================================

#[test]
fn local_name_from_qname() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"local-name-from-QName(QName("http://www.w3.org/2005/xpath-functions", "fn:contains"))"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("contains".to_string()))
    );
}

#[test]
fn local_name_from_qname_empty_sequence() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("local-name-from-QName(())").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "empty sequence input returns empty sequence");
}

// ============================================================
// fn:namespace-uri-from-QName
// ============================================================

#[test]
fn namespace_uri_from_qname() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"namespace-uri-from-QName(QName("http://www.w3.org/2005/xpath-functions", "fn:contains"))"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String(
            "http://www.w3.org/2005/xpath-functions".to_string()
        ))
    );
}

#[test]
fn namespace_uri_from_qname_empty_sequence() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("namespace-uri-from-QName(())").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

// ============================================================
// fn:prefix-from-QName
// ============================================================

#[test]
fn prefix_from_qname_with_prefix() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"prefix-from-QName(QName("http://www.w3.org/2005/xpath-functions", "fn:contains"))"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("fn".to_string()))
    );
}

#[test]
fn prefix_from_qname_no_prefix() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath =
        xpath::parse(r#"prefix-from-QName(QName("http://example.com", "localname"))"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "no prefix returns empty sequence");
}

#[test]
fn prefix_from_qname_empty_sequence() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("prefix-from-QName(())").unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0);
}

// ============================================================
// instance of / treat as for xs:QName
// ============================================================

#[test]
fn qname_instance_of() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"QName("http://example.com", "name") instance of xs:QName"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

#[test]
fn string_not_instance_of_qname() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" instance of xs:QName"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// fn:function-lookup
// ============================================================

#[test]
fn function_lookup_known_function() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://www.w3.org/2005/xpath-functions", "contains"), 2)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    match &items[0] {
        XpathItem::Function(Function::Named { name, arity }) => {
            assert_eq!(name, "fn:contains");
            assert_eq!(*arity, 2);
        }
        other => panic!("expected Named function, got {:?}", other),
    }
}

#[test]
fn function_lookup_unknown_function() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://www.w3.org/2005/xpath-functions", "nonexistent"), 1)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "unknown function returns empty sequence");
}

#[test]
fn function_lookup_wrong_arity() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    // fn:contains takes 2 args, not 5
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://www.w3.org/2005/xpath-functions", "contains"), 5)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "wrong arity returns empty sequence");
}

#[test]
fn function_lookup_unknown_namespace() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://example.com/unknown", "foo"), 1)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 0, "unknown namespace returns empty sequence");
}

#[test]
fn function_lookup_map_namespace() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://www.w3.org/2005/xpath-functions/map", "size"), 1)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    match &items[0] {
        XpathItem::Function(Function::Named { name, arity }) => {
            assert_eq!(name, "map:size");
            assert_eq!(*arity, 1);
        }
        other => panic!("expected Named function, got {:?}", other),
    }
}

#[test]
fn function_lookup_math_namespace() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://www.w3.org/2005/xpath-functions/math", "pi"), 0)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    match &items[0] {
        XpathItem::Function(Function::Named { name, arity }) => {
            assert_eq!(name, "math:pi");
            assert_eq!(*arity, 0);
        }
        other => panic!("expected Named function, got {:?}", other),
    }
}

#[test]
fn function_lookup_and_call() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    // Look up fn:contains and call it dynamically
    let xpath = xpath::parse(
        r#"let $f := function-lookup(QName("http://www.w3.org/2005/xpath-functions", "contains"), 2) return $f("hello world", "world")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

#[test]
fn function_lookup_feature_detection() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    // Use function-lookup for feature detection: check if a function exists
    // Use exists() since EBV is not defined for function items per spec.
    let xpath = xpath::parse(
        r#"if (exists(function-lookup(QName("http://www.w3.org/2005/xpath-functions", "contains"), 2))) then "yes" else "no""#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("yes".to_string()))
    );
}

#[test]
fn function_lookup_feature_detection_unknown() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    // Feature detection for unknown function returns "no"
    let xpath = xpath::parse(
        r#"if (function-lookup(QName("http://www.w3.org/2005/xpath-functions", "nonexistent"), 1)) then "yes" else "no""#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("no".to_string()))
    );
}

#[test]
fn function_lookup_array_namespace() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://www.w3.org/2005/xpath-functions/array", "size"), 1)"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    match &items[0] {
        XpathItem::Function(Function::Named { name, arity }) => {
            assert_eq!(name, "array:size");
            assert_eq!(*arity, 1);
        }
        other => panic!("expected Named function, got {:?}", other),
    }
}

// ============================================================
// cast as / castable as
// ============================================================

#[test]
fn qname_cast_as_string() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"QName("http://www.w3.org/2005/xpath-functions", "fn:contains") cast as xs:string"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::String("fn:contains".to_string()))
    );
}

#[test]
fn string_cast_as_qname_is_error() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" cast as xs:QName"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "casting string to QName should error");
}

#[test]
fn qname_castable_as_string() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"QName("http://example.com", "name") castable as xs:string"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

#[test]
fn string_castable_as_qname() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#""hello" castable as xs:QName"#).unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// Arithmetic errors
// ============================================================

#[test]
fn qname_arithmetic_is_error() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(r#"QName("http://example.com", "name") + 1"#).unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "QName in arithmetic should error");
}

// ============================================================
// QName equality
// ============================================================

#[test]
fn qname_eq_same() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"QName("http://example.com", "name") eq QName("http://example.com", "name")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(true))
    );
}

#[test]
fn qname_eq_different() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"QName("http://example.com", "a") eq QName("http://example.com", "b")"#,
    )
    .unwrap();

    let items = xpath.apply(&document).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        XpathItem::AnyAtomicType(AnyAtomicType::Boolean(false))
    );
}

// ============================================================
// fn:function-lookup edge cases
// ============================================================

#[test]
fn function_lookup_negative_arity_is_error() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse(
        r#"function-lookup(QName("http://www.w3.org/2005/xpath-functions", "contains"), -1)"#,
    )
    .unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "negative arity should error");
}

#[test]
fn function_lookup_empty_qname_is_error() {
    let text = r#"<html><body></body></html>"#;
    let document = html::parse(text).unwrap();
    let xpath = xpath::parse("function-lookup((), 1)").unwrap();

    let result = xpath.apply(&document);
    assert!(result.is_err(), "empty sequence for QName arg should error");
}
