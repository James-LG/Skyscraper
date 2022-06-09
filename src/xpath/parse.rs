use std::iter::Peekable;

use thiserror::Error;

use crate::xpath::{Xpath, XpathElement, XpathPredicate, XpathQuery, tokenizer::{self, Symbol}};

use super::{tokenizer::LexError, XpathAxes, XpathSearchItem, XpathSearchNodeType};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("close square bracket has no matching opening square bracket")]
    LeadingCloseBracket,
    #[error("@ symbol cannot be outside of square brackets")]
    MisplacedAtSign,
    #[error("predicate missing assignment sign")]
    PredicateMissingAssignmentSign,
    #[error("predicate missing value")]
    PredicateMissingValue,
    #[error("predicate missing attribute")]
    PredicateMissingAttribute,
    #[error("lex error {0}")]
    LexError(#[from] LexError),
    #[error("'::' must be preceded by an axis (e.g. parent)")]
    MissingAxis,
    #[error("'::' must be proceeded by a tag (e.g. div)")]
    MissingAxisTag,
    #[error("unknown axis type `{0}`")]
    UnknownAxisType(String)
}

/// Parse an Xpath expression into an Xpath object.
pub fn parse(text: &str) -> Result<Xpath, ParseError> {
    let elements = inner_parse(text)?;
    let mut xpath_items: Vec<XpathSearchItem> = Vec::new();

    let mut cur_search_item: XpathSearchItem = Default::default();
    let mut is_first_element = true;
    let mut was_last_element_search = false;
    for element in elements.into_iter() {
        was_last_element_search = false;
        match element {
            XpathElement::SearchRoot => {
                if !is_first_element {
                    xpath_items.push(cur_search_item);
                    cur_search_item = Default::default();
                }
                was_last_element_search = true;
            },
            XpathElement::SearchAll => {
                if !is_first_element {
                    xpath_items.push(cur_search_item);
                    cur_search_item = Default::default();
                }
                was_last_element_search = true;

                // SearchAll is an abbreviation of `/descendant-or-self::node()/`
                xpath_items.push(XpathSearchItem {
                    axis: XpathAxes::DescendantOrSelf,
                    index: None,
                    query: None,
                    search_node_type: XpathSearchNodeType::Any
                });
            },
            XpathElement::Tag(tag_name) => {
                cur_search_item.search_node_type = XpathSearchNodeType::Element(tag_name);
            },
            XpathElement::Query(query) => {
                cur_search_item.query = Some(query);
            },
            XpathElement::Index(index) => {
                cur_search_item.index = Some(index);
            },
            XpathElement::Axis(axis) => {
                cur_search_item.axis = axis;
            },
        }
        is_first_element = false;
    }

    // If we have a dangling item due to no trailing slashes, add it now.
    if !was_last_element_search {
        xpath_items.push(cur_search_item);
    }

    Ok(Xpath { items: xpath_items })
}

/// First stage of parsing, converts tokens into more structured [XpathElements](XpathElement).
fn inner_parse(text: &str) -> Result<Vec<XpathElement>, ParseError> {
    let mut symbols = tokenizer::lex(text)?.into_iter().peekable();
    let mut elements: Vec<XpathElement> = Vec::new();

    while let Some(symbol) = symbols.next() {
        match symbol {
            Symbol::Slash => elements.push(XpathElement::SearchRoot),
            Symbol::DoubleSlash => elements.push(XpathElement::SearchAll),
            Symbol::OpenSquareBracket => {
                if let Some(num) = parse_index(&mut symbols) {
                    elements.push(XpathElement::Index(num));
                } else {
                    let query = parse_query(&mut symbols)?;
                    elements.push(XpathElement::Query(query));
                }
            },
            Symbol::Identifier(identifier) => {
                elements.push(XpathElement::Tag(identifier));
            },
            Symbol::DoubleColon => {
                parse_axis_selector(&mut elements)?;
            }
            _ => continue,
        }
    }

    Ok(elements)
}

/// Parses tree selectors. Triggered when a DoubleColon (Symbol)[Symbol] is found and expects a tag to
/// have preceded it which will now be converted to an axis.
/// E.g. /div/parent::div
fn parse_axis_selector(elements: &mut Vec<XpathElement>) -> Result<(), ParseError> {
    let last_item = elements.pop()
        .ok_or_else(|| ParseError::MissingAxis)?;
    let axis = match last_item {
        XpathElement::Tag(last_tag) => {
            match last_tag.as_str() {
                "parent" => XpathAxes::Parent,
                _ => return Err(ParseError::UnknownAxisType(last_tag))
            }
        
        },
        _ => return Err(ParseError::MissingAxis)
    };
    elements.push(XpathElement::Axis(axis));
    Ok(())
}

/// Parses an index selector.
/// Example: [1]
fn parse_index(symbols: &mut Peekable<std::vec::IntoIter<Symbol>>) -> Option<usize> {
    if let Some(Symbol::Number(num)) = symbols.next_if(|expected| matches!(expected, &Symbol::Number(_))) {
        if let Some(Symbol::CloseSquareBracket) = symbols.next_if_eq(&Symbol::CloseSquareBracket) {
            return Some(num as usize);
        }
    }

    return None
}

/// Parses the inner section of square brackets in an Xpath expression.
/// Assumes this set of square brackets has already been checked for an index value.
/// Example: [@name='hi']
fn parse_query(symbols: &mut Peekable<std::vec::IntoIter<Symbol>>) -> Result<XpathQuery, ParseError> {
    let mut query = XpathQuery::new();

    let mut open_square_bracket = true;
    while let Some(symbol) = symbols.peek() {
        match symbol {
            Symbol::OpenSquareBracket => {
                symbols.next();
                open_square_bracket = true;
            },
            Symbol::CloseSquareBracket => {
                symbols.next();
                if open_square_bracket {
                    open_square_bracket = false;
                } else {
                    return Err(ParseError::LeadingCloseBracket);
                }
            }
            Symbol::AtSign => {
                symbols.next();
                if !open_square_bracket {
                    return Err(ParseError::MisplacedAtSign);
                }
                let predicate = parse_equals_predicate(symbols)?;
                query.predicates.push(predicate);
            },
            _ => break,
        }
    }

    Ok(query)
}

fn parse_equals_predicate(symbols: &mut Peekable<std::vec::IntoIter<Symbol>>) -> Result<XpathPredicate, ParseError> {
    let mut attr: Option<String> = None;
    let mut val: Option<String> = None;

    if let Some(Symbol::Identifier(attribute)) = symbols.next_if(|expected| matches!(expected, &Symbol::Identifier(_))) {
        attr = Some(attribute);
    }

    if let Some(Symbol::AssignmentSign) = symbols.next_if(|expected| matches!(expected, &Symbol::AssignmentSign)) {
        // good
    } else {
        return Err(ParseError::PredicateMissingAssignmentSign);
    }

    if let Some(Symbol::Text(value)) = symbols.next_if(|expected| matches!(expected, &Symbol::Text(_))) {
        val = Some(value);
    }
    
    if let Some(attribute) = attr {
        if let Some(value) = val {
            Ok(XpathPredicate::Equals { attribute, value })
        } else {
            Err(ParseError::PredicateMissingValue)
        }
    } else {
        Err(ParseError::PredicateMissingAttribute)
    }
}

#[cfg(test)]
mod tests {
    use crate::xpath::XpathPredicate;

    use super::*;

    #[test]
    fn inner_parse_works() {
        let text = "//book/title";

        let result = inner_parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Tag(String::from("book")),
            XpathElement::SearchRoot,
            XpathElement::Tag(String::from("title"))
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn inner_parse_index() {
        let text = r###"//a[0]"###;

        let result = inner_parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Tag(String::from("a")),
            XpathElement::Index(0),
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn inner_parse_attribute() {
        let text = r###"//a[@hello="world"]"###;

        let result = inner_parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Tag(String::from("a")),
            XpathElement::Query(
                XpathQuery {
                    predicates: vec![
                        XpathPredicate::Equals {
                            attribute: String::from("hello"),
                            value: String::from("world")
                        }
                    ]
                }
            ),
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn parse_works() {
        let text = r###"//book/title"###;

        let result = parse(text).unwrap();

        let expected = vec![
            XpathSearchItem {
                axis: XpathAxes::DescendantOrSelf,
                search_node_type: XpathSearchNodeType::Any,
                index: None,
                query: None
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("book")),
                index: None,
                query: None
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("title")),
                index: None,
                query: None
            }
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        assert_eq!(expected.len(), result.items.len());
        for (e, r) in expected.into_iter().zip(result.items) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn parse_works_with_root() {
        let text = r###"/book/title"###;

        let result = parse(text).unwrap();

        let expected = vec![
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("book")),
                index: None,
                query: None
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("title")),
                index: None,
                query: None
            }
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        assert_eq!(expected.len(), result.items.len());
        for (e, r) in expected.into_iter().zip(result.items) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn parse_index() {
        let text = r###"//a[0]"###;

        let result = parse(text).unwrap();

        let expected = vec![
            XpathSearchItem {
                axis: XpathAxes::DescendantOrSelf,
                search_node_type: XpathSearchNodeType::Any,
                index: None,
                query: None
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("a")),
                index: Some(0),
                query: None
            }
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        assert_eq!(expected.len(), result.items.len());
        for (e, r) in expected.into_iter().zip(result.items) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn parse_attribute() {
        let text = r###"//a[@hello="world"]"###;

        let result = parse(text).unwrap();

        let expected = vec![
            XpathSearchItem {
                axis: XpathAxes::DescendantOrSelf,
                search_node_type: XpathSearchNodeType::Any,
                index: None,
                query: None
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("a")),
                index: None,
                query: Some(XpathQuery {
                    predicates: vec![
                        XpathPredicate::Equals {
                            attribute: String::from("hello"),
                            value: String::from("world")
                        }
                    ]
                })
            }
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        assert_eq!(expected.len(), result.items.len());
        for (e, r) in expected.into_iter().zip(result.items) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn parse_parent_axis() {
        let text = r###"//div[@role='gridcell']//parent::div"###;

        let result = parse(text).unwrap();

        let expected = vec![
            XpathSearchItem {
                axis: XpathAxes::DescendantOrSelf,
                search_node_type: XpathSearchNodeType::Any,
                index: None,
                query: None
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("div")),
                index: None,
                query: Some(XpathQuery {
                    predicates: vec![
                        XpathPredicate::Equals {
                            attribute: String::from("role"),
                            value: String::from("gridcell")
                        }
                    ]
                })
            },
            XpathSearchItem {
                axis: XpathAxes::DescendantOrSelf,
                search_node_type: XpathSearchNodeType::Any,
                index: None,
                query: None
            },
            XpathSearchItem {
                axis: XpathAxes::Parent,
                search_node_type: XpathSearchNodeType::Element(String::from("div")),
                index: None,
                query: None
            },
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        assert_eq!(expected.len(), result.items.len());
        for (e, r) in expected.into_iter().zip(result.items) {
            assert_eq!(e, r);
        }
    }
}
