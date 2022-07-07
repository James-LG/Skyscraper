//! Parses XPath text into structured [Xpath] expressions.

use std::iter::Peekable;

use thiserror::Error;

use crate::xpath::{
    tokenizer::{self, Token},
    Xpath, XpathPredicate, XpathQuery,
};

use super::{tokenizer::LexError, XpathAxes, XpathSearchItem, XpathSearchNodeType};

#[derive(Debug, PartialEq)]
enum XpathElement {
    SearchRoot,
    SearchAll,
    Tag(String),
    Query(XpathQuery),
    Index(usize),
    Axis(XpathAxes),
}

/// An error occurring during XPath expression parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    /// Closing bracket appeared before a corresponding open bracket.
    /// 
    /// ```text
    /// //div @class="node"]
    ///      ^
    ///      └ Missing: Open bracket "["
    #[error("close square bracket has no matching opening square bracket")]
    LeadingCloseBracket,

    /// `@` symbol appear outside of square brackets.
    /// 
    /// ```text
    /// //div@[class="node"]/div
    ///      ^
    ///      └ Error: @ symbol should prefix attribute conditions.\
    /// ```
    #[error("@ symbol cannot be outside of square brackets")]
    MisplacedAtSign,

    /// Predicate missing an operator sign.
    /// 
    /// **Note:** Currently only assignment signs are supported.
    /// 
    /// ```text
    /// //div[@class "node"]
    ///             ^
    ///             └ Error: "=" sign missing
    /// ```
    #[error("predicate missing operator")]
    PredicateMissingOperator,

    /// Predicate has no value to compare with.
    /// 
    /// ```text
    /// //div[@class= ]
    ///              ^
    ///              └ Error: Missing some text or other value
    /// ```
    #[error("predicate missing value")]
    PredicateMissingValue,

    /// Predicate has no attribute name.
    /// 
    /// ```text
    /// //div[@ ="node"]
    ///        ^
    ///        └ Error: Missing attribute name such as "class"
    /// ```
    #[error("predicate missing attribute")]
    PredicateMissingAttribute,

    /// An error occurring during the tokenization of the Xpath expression.
    #[error("lex error {0}")]
    LexError(#[from] LexError),

    /// Axis must be provided before `::` operator.
    /// 
    /// ```text
    /// //div/ ::div
    ///       ^
    ///       └ Error: Missing axis such as "parent"
    /// ```
    #[error("'::' must be preceded by an axis (e.g. parent)")]
    MissingAxis,

    /// Tag name must be provided after `::` operator.
    /// 
    /// ```text
    /// //div/parent::
    ///               ^
    ///               └ Error: Missing tag name such as "div"
    /// ```
    #[error("'::' must be proceeded by a tag (e.g. div)")]
    MissingAxisTag,

    /// Provided axis name was unknown.
    /// 
    /// See [XpathAxes] for currently supported options.
    /// 
    /// ```text
    /// //div/bogus::div
    ///       ^^^^^
    ///       └┴┴┴┴ Error: "bogus" is not a valid axis name.
    #[error("unknown axis type `{0}`")]
    UnknownAxisType(String),

    /// Index value must be >= 1.
    ///
    /// ```text
    /// //div[0]
    ///       ^
    ///       └ Error: Index value must be >= 1.
    /// ```
    #[error("index value can not be zero")]
    IndexValueIsZero,
}

/// Parse an Xpath expression into an Xpath object.
/// 
/// # Example: parse an XPath expression
/// ```rust
/// use skyscraper::xpath::{self, parse::ParseError};
/// # fn main() -> Result<(), ParseError> {
/// let expr = xpath::parse("//div[@class='hi']/parent::span//a")?;
/// # Ok(())
/// # }
/// ```
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
            }
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
                    search_node_type: XpathSearchNodeType::Any,
                });
            }
            XpathElement::Tag(tag_name) => {
                cur_search_item.search_node_type = XpathSearchNodeType::Element(tag_name);
            }
            XpathElement::Query(query) => {
                cur_search_item.query = Some(query);
            }
            XpathElement::Index(index) => {
                cur_search_item.index = Some(index);
            }
            XpathElement::Axis(axis) => {
                cur_search_item.axis = axis;
            }
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
            Token::Slash => elements.push(XpathElement::SearchRoot),
            Token::DoubleSlash => elements.push(XpathElement::SearchAll),
            Token::OpenSquareBracket => {
                if let Some(num) = parse_index(&mut symbols) {
                    if num == 0 {
                        return Err(ParseError::IndexValueIsZero);
                    } else {
                        elements.push(XpathElement::Index(num));
                    }
                } else {
                    let query = parse_query(&mut symbols)?;
                    elements.push(XpathElement::Query(query));
                }
            }
            Token::Identifier(identifier) => {
                elements.push(XpathElement::Tag(identifier));
            }
            Token::DoubleColon => {
                parse_axis_selector(&mut elements)?;
            }
            _ => continue,
        }
    }

    Ok(elements)
}

/// Parses tree selectors. Triggered when a DoubleColon (Symbol)[Symbol] is found and expects a tag to
/// have preceded it which will now be converted to an axis.
/// 
/// Example: `/div/parent::div`
fn parse_axis_selector(elements: &mut Vec<XpathElement>) -> Result<(), ParseError> {
    let last_item = elements.pop().ok_or(ParseError::MissingAxis)?;
    let axis = match last_item {
        XpathElement::Tag(last_tag) => match last_tag.as_str() {
            "parent" => XpathAxes::Parent,
            _ => return Err(ParseError::UnknownAxisType(last_tag)),
        },
        _ => return Err(ParseError::MissingAxis),
    };
    elements.push(XpathElement::Axis(axis));
    Ok(())
}

/// Parses an index selector.
/// 
/// Example: `[1]`
fn parse_index(symbols: &mut Peekable<std::vec::IntoIter<Token>>) -> Option<usize> {
    if let Some(Token::Number(num)) =
        symbols.next_if(|expected| matches!(expected, &Token::Number(_)))
    {
        if let Some(Token::CloseSquareBracket) = symbols.next_if_eq(&Token::CloseSquareBracket) {
            return Some(num as usize);
        }
    }

    None
}

/// Parses the inner section of square brackets in an Xpath expression.
/// 
/// Assumes this set of square brackets has already been checked for an index value.
/// 
/// Example: `[@name='hi']`
fn parse_query(
    symbols: &mut Peekable<std::vec::IntoIter<Token>>,
) -> Result<XpathQuery, ParseError> {
    let mut query = XpathQuery::new();

    let mut open_square_bracket = true;
    while let Some(symbol) = symbols.peek() {
        match symbol {
            Token::OpenSquareBracket => {
                symbols.next();
                open_square_bracket = true;
            }
            Token::CloseSquareBracket => {
                symbols.next();
                if open_square_bracket {
                    open_square_bracket = false;
                } else {
                    return Err(ParseError::LeadingCloseBracket);
                }
            }
            Token::AtSign => {
                symbols.next();
                if !open_square_bracket {
                    return Err(ParseError::MisplacedAtSign);
                }
                let predicate = parse_equals_predicate(symbols)?;
                query.predicates.push(predicate);
            }
            _ => break,
        }
    }

    Ok(query)
}

fn parse_equals_predicate(
    symbols: &mut Peekable<std::vec::IntoIter<Token>>,
) -> Result<XpathPredicate, ParseError> {
    let mut attr: Option<String> = None;
    let mut val: Option<String> = None;

    if let Some(Token::Identifier(attribute)) =
        symbols.next_if(|expected| matches!(expected, &Token::Identifier(_)))
    {
        attr = Some(attribute);
    }

    if let Some(Token::AssignmentSign) =
        symbols.next_if(|expected| matches!(expected, &Token::AssignmentSign))
    {
        // good
    } else {
        return Err(ParseError::PredicateMissingOperator);
    }

    if let Some(Token::Text(value)) =
        symbols.next_if(|expected| matches!(expected, &Token::Text(_)))
    {
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
            XpathElement::Tag(String::from("title")),
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn inner_parse_index() {
        let text = r###"//a[1]"###;

        let result = inner_parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Tag(String::from("a")),
            XpathElement::Index(1),
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn inner_parse_zero_index() {
        let text = r###"//a[0]"###;
        let result = inner_parse(text);
	assert!(result.is_err());
	let err = result.unwrap_err();
	assert!(matches!(&err, ParseError::IndexValueIsZero));
	assert_eq!(err.to_string(), "index value can not be zero");
    }

    #[test]
    fn inner_parse_attribute() {
        let text = r###"//a[@hello="world"]"###;

        let result = inner_parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Tag(String::from("a")),
            XpathElement::Query(XpathQuery {
                predicates: vec![XpathPredicate::Equals {
                    attribute: String::from("hello"),
                    value: String::from("world"),
                }],
            }),
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
                query: None,
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("book")),
                index: None,
                query: None,
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("title")),
                index: None,
                query: None,
            },
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
                query: None,
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("title")),
                index: None,
                query: None,
            },
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        assert_eq!(expected.len(), result.items.len());
        for (e, r) in expected.into_iter().zip(result.items) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn parse_index() {
        let text = r###"//a[1]"###;

        let result = parse(text).unwrap();

        let expected = vec![
            XpathSearchItem {
                axis: XpathAxes::DescendantOrSelf,
                search_node_type: XpathSearchNodeType::Any,
                index: None,
                query: None,
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("a")),
                index: Some(1),
                query: None,
            },
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
                query: None,
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("a")),
                index: None,
                query: Some(XpathQuery {
                    predicates: vec![XpathPredicate::Equals {
                        attribute: String::from("hello"),
                        value: String::from("world"),
                    }],
                }),
            },
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
                query: None,
            },
            XpathSearchItem {
                axis: XpathAxes::Child,
                search_node_type: XpathSearchNodeType::Element(String::from("div")),
                index: None,
                query: Some(XpathQuery {
                    predicates: vec![XpathPredicate::Equals {
                        attribute: String::from("role"),
                        value: String::from("gridcell"),
                    }],
                }),
            },
            XpathSearchItem {
                axis: XpathAxes::DescendantOrSelf,
                search_node_type: XpathSearchNodeType::Any,
                index: None,
                query: None,
            },
            XpathSearchItem {
                axis: XpathAxes::Parent,
                search_node_type: XpathSearchNodeType::Element(String::from("div")),
                index: None,
                query: None,
            },
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        assert_eq!(expected.len(), result.items.len());
        for (e, r) in expected.into_iter().zip(result.items) {
            assert_eq!(e, r);
        }
    }
}
