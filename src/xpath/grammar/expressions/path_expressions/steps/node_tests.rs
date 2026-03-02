//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#node-tests>

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, error::context, sequence::tuple,
};

use crate::{
    html::grammar::{HTML_NAMESPACE, MATHML_NAMESPACE, SVG_NAMESPACE},
    xpath::{
        grammar::{
            data_model::XpathItem,
            recipes::Res,
            terminal_symbols::braced_uri_literal,
            types::{eq_name, kind_test, EQName, KindTest},
            xml_names::{nc_name, QName},
            XpathItemTree, XpathItemTreeNode,
        },
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

use super::axes::{forward_axis::ForwardAxis, reverse_axis::ReverseAxis};

pub fn node_test(input: &str) -> Res<&str, NodeTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NodeTest

    fn kind_test_map(input: &str) -> Res<&str, NodeTest> {
        kind_test(input).map(|(next_input, res)| (next_input, NodeTest::KindTest(res)))
    }

    fn name_test_map(input: &str) -> Res<&str, NodeTest> {
        name_test(input).map(|(next_input, res)| (next_input, NodeTest::NameTest(res)))
    }

    context("node_test", alt((kind_test_map, name_test_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum NodeTest {
    KindTest(KindTest),
    NameTest(NameTest),
}

impl Display for NodeTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeTest::KindTest(x) => write!(f, "{}", x),
            NodeTest::NameTest(x) => write!(f, "{}", x),
        }
    }
}

impl NodeTest {
    pub(crate) fn eval<'tree>(
        &self,
        axis: BiDirectionalAxis,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<Option<&'tree XpathItemTreeNode>, ExpressionApplyError> {
        match self {
            NodeTest::KindTest(test) => {
                let filtered_nodes =
                    test.filter(&xpath_item_set![context.item.clone()], context.item_tree)?;

                if !filtered_nodes.is_empty() {
                    let node: &'tree XpathItemTreeNode = filtered_nodes.into_iter().next().unwrap();
                    Ok(Some(node))
                } else {
                    Ok(None)
                }
            }
            NodeTest::NameTest(test) => test.eval(axis, context),
        }
    }

    /// Test whether a node matches this node test directly, without creating
    /// an `XpathExpressionContext`. This avoids per-node allocation overhead
    /// in axis evaluation loops.
    pub(crate) fn matches_node<'tree>(
        &self,
        axis: BiDirectionalAxis,
        node: &'tree XpathItemTreeNode,
        item_tree: &'tree XpathItemTree,
    ) -> Result<bool, ExpressionApplyError> {
        match self {
            NodeTest::KindTest(test) => test.matches_node(node, item_tree),
            NodeTest::NameTest(test) => test.matches_node(axis, node),
        }
    }
}

fn name_test(input: &str) -> Res<&str, NameTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NameTest

    fn eq_name_map(input: &str) -> Res<&str, NameTest> {
        eq_name(input).map(|(next_input, res)| (next_input, NameTest::Name(res)))
    }

    fn wildcard_map(input: &str) -> Res<&str, NameTest> {
        wildcard(input).map(|(next_input, res)| (next_input, NameTest::Wildcard(res)))
    }

    // Try wildcard first: `NCName:*`, `*:NCName`, `Q{URI}*`, and `*` are
    // never valid EQNames, so there is no ambiguity. If we try eq_name first,
    // nom greedily parses `svg:*` as the unprefixed name "svg" (leaving ":*"),
    // because `QName::PrefixedName` fails (`*` is not a valid NCName for the
    // local part) and `QName::UnprefixedName` succeeds on just the prefix.
    context("name_test", alt((wildcard_map, eq_name_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum NameTest {
    Name(EQName),
    Wildcard(Wildcard),
}

impl Display for NameTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameTest::Name(x) => write!(f, "{}", x),
            NameTest::Wildcard(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum BiDirectionalAxis {
    ForwardAxis(ForwardAxis),
    ReverseAxis(ReverseAxis),
}

impl NameTest {
    /// Test whether a node matches this name test directly, without requiring
    /// a full `XpathExpressionContext`.
    pub(crate) fn matches_node<'tree>(
        &self,
        axis: BiDirectionalAxis,
        node: &'tree XpathItemTreeNode,
    ) -> Result<bool, ExpressionApplyError> {
        let is_match = match self {
            NameTest::Name(expected_name) => {
                let is_principal_node_kind = match axis {
                    BiDirectionalAxis::ForwardAxis(ForwardAxis::Attribute) => {
                        matches!(node, XpathItemTreeNode::AttributeNode(_))
                    }
                    _ => {
                        matches!(node, XpathItemTreeNode::ElementNode(_))
                    }
                };

                if !is_principal_node_kind {
                    false
                } else {
                    let (node_name, node_ns): (Option<&str>, Option<&str>) = match node {
                        XpathItemTreeNode::ElementNode(e) => {
                            (Some(&e.name), e.namespace.as_deref())
                        }
                        XpathItemTreeNode::AttributeNode(a) => (Some(&a.name), None),
                        _ => (None, None),
                    };

                    match node_name {
                        Some(node_name) => match expected_name {
                            EQName::QName(qname) => match qname {
                                QName::PrefixedName(p) => p.local_part == node_name,
                                QName::UnprefixedName(unprefixed_name) => {
                                    unprefixed_name == node_name
                                }
                            },
                            EQName::UriQualifiedName(uqn) => {
                                uqn.name == node_name
                                    && node_ns.map_or(false, |ns| ns == uqn.uri)
                            }
                        },
                        None => false,
                    }
                }
            }
            NameTest::Wildcard(wildcard) => wildcard.is_match(axis, node)?,
        };

        Ok(is_match)
    }

    pub(crate) fn eval<'tree>(
        &self,
        axis: BiDirectionalAxis,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<Option<&'tree XpathItemTreeNode>, ExpressionApplyError> {
        let node = if let XpathItem::Node(node) = &context.item {
            node
        } else {
            // Name tests only match nodes; non-node items never match.
            return Ok(None);
        };

        let is_match = match self {
            NameTest::Name(expected_name) => {
                // A name test is true only if the node is the principal node kind
                // for the axis. (XPath 3.1 §2.5.4.2)
                // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-principal-node-kind
                let is_principal_node_kind = match axis {
                    BiDirectionalAxis::ForwardAxis(ForwardAxis::Attribute) => {
                        matches!(node, XpathItemTreeNode::AttributeNode(_))
                    }
                    _ => {
                        matches!(node, XpathItemTreeNode::ElementNode(_))
                    }
                };

                if !is_principal_node_kind {
                    false
                } else {
                // Get the name and namespace of the node, if available for the node type.
                let (node_name, node_ns): (Option<&str>, Option<&str>) = match node {
                    XpathItemTreeNode::ElementNode(e) => {
                        (Some(&e.name), e.namespace.as_deref())
                    }
                    XpathItemTreeNode::AttributeNode(a) => (Some(&a.name), None),
                    _ => (None, None),
                };

                match node_name {
                    Some(node_name) => match expected_name {
                        EQName::QName(qname) => match qname {
                            QName::PrefixedName(p) => {
                                // In HTML context, there are no in-scope namespace
                                // bindings for prefixes. Match the local part only
                                // when the prefix is a well-known namespace.
                                // For now, treat the local part as the name to match.
                                p.local_part == node_name
                            }
                            QName::UnprefixedName(unprefixed_name) => unprefixed_name == node_name,
                        },
                        EQName::UriQualifiedName(uqn) => {
                            // Match both the namespace URI and the local name.
                            uqn.name == node_name
                                && node_ns.map_or(false, |ns| ns == uqn.uri)
                        }
                    },

                    // Name tests need a name to match.
                    // If the node does not have a name, it cannot match.
                    None => false,
                }
                }
            }
            NameTest::Wildcard(wildcard) => wildcard.is_match(axis, node)?,
        };

        if is_match {
            Ok(Some(*node))
        } else {
            Ok(None)
        }
    }
}

fn wildcard(input: &str) -> Res<&str, Wildcard> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-Wildcard
    // ws: explicit

    fn prefixed_name_map(input: &str) -> Res<&str, Wildcard> {
        tuple((tag("*:"), nc_name))(input)
            .map(|(next_input, res)| (next_input, Wildcard::PrefixedName(res.1.to_string())))
    }

    fn suffixed_name_map(input: &str) -> Res<&str, Wildcard> {
        tuple((nc_name, tag(":*")))(input)
            .map(|(next_input, res)| (next_input, Wildcard::SuffixedName(res.0.to_string())))
    }

    fn braced_uri_map(input: &str) -> Res<&str, Wildcard> {
        tuple((braced_uri_literal, char('*')))(input)
            .map(|(next_input, res)| (next_input, Wildcard::BracedUri(res.0.to_string())))
    }

    fn simple_map(input: &str) -> Res<&str, Wildcard> {
        tuple((char('*'),))(input).map(|(next_input, _res)| (next_input, Wildcard::Simple))
    }

    context(
        "wildcard",
        alt((
            prefixed_name_map,
            suffixed_name_map,
            braced_uri_map,
            simple_map,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Wildcard {
    Simple,
    PrefixedName(String),
    SuffixedName(String),
    BracedUri(String),
}

impl Display for Wildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wildcard::Simple => write!(f, "*"),
            Wildcard::PrefixedName(x) => write!(f, "*:{}", x),
            Wildcard::SuffixedName(x) => write!(f, "{}:*", x),
            Wildcard::BracedUri(x) => write!(f, "Q{{{}}}*", x),
        }
    }
}

/// Resolve a namespace prefix to its namespace URI using well-known bindings.
///
/// In an HTML-only processor there are no in-scope namespace bindings from an
/// XML context, so we use the three namespaces defined by the WHATWG HTML spec:
/// - `html`   → `http://www.w3.org/1999/xhtml`
/// - `svg`    → `http://www.w3.org/2000/svg`
/// - `mathml` → `http://www.w3.org/1998/Math/MathML`
///
/// Note: `math` is deliberately *not* bound here because the XPath 3.1 spec
/// reserves it for the XPath math functions namespace
/// (`http://www.w3.org/2005/xpath-functions/math`), which is already used by
/// function dispatch (e.g. `math:pi()`). Use `mathml` for MathML elements.
///
/// Returns `Err` with `XPST0081` for unrecognized prefixes per XPath 3.1 §3.1.1.
fn resolve_prefix(prefix: &str) -> Result<&'static str, ExpressionApplyError> {
    match prefix {
        "html" => Ok(HTML_NAMESPACE),
        "svg" => Ok(SVG_NAMESPACE),
        "mathml" => Ok(MATHML_NAMESPACE),
        _ => Err(ExpressionApplyError::new(format!(
            "err:XPST0081 Namespace prefix `{prefix}` is not bound to a namespace URI"
        ))),
    }
}

impl Wildcard {
    pub(crate) fn is_match<'tree>(
        &self,
        axis: BiDirectionalAxis,
        node: &'tree XpathItemTreeNode,
    ) -> Result<bool, ExpressionApplyError> {
        // Wildcards only match context items that are the axis' principal node kind.
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-principal-node-kind
        let is_principal_node_kind = match axis {
            BiDirectionalAxis::ForwardAxis(ForwardAxis::Attribute) => {
                // For the attribute axis, the principal node kind is attribute.
                matches!(node, XpathItemTreeNode::AttributeNode(_))
            }
            _ => {
                // For all other axes, the principal node kind is element.
                matches!(node, XpathItemTreeNode::ElementNode(_),)
            }
        };

        if !is_principal_node_kind {
            return Ok(false);
        }

        // Get node name and namespace for matching.
        // HTML elements have namespace `None` (implicitly HTML_NAMESPACE).
        let (node_name, node_ns): (Option<&str>, Option<&str>) = match node {
            XpathItemTreeNode::ElementNode(e) => {
                (Some(e.name.as_str()), e.namespace.as_deref())
            }
            XpathItemTreeNode::AttributeNode(a) => (Some(a.name.as_str()), a.namespace.as_deref()),
            _ => (None, None),
        };

        match self {
            Wildcard::Simple => Ok(true),
            Wildcard::PrefixedName(local) => {
                // `*:local` — matches any namespace, local name must match.
                Ok(node_name.map_or(false, |n| n == local))
            }
            Wildcard::SuffixedName(prefix) => {
                // `prefix:*` — matches any local name in the namespace bound
                // to the prefix. Resolve the prefix via well-known bindings;
                // raises XPST0081 if the prefix is unknown.
                let target_ns = resolve_prefix(prefix)?;
                // HTML elements store namespace as None (implicitly HTML_NAMESPACE).
                let effective_ns = node_ns.unwrap_or(HTML_NAMESPACE);
                Ok(effective_ns == target_ns)
            }
            Wildcard::BracedUri(uri) => {
                // `Q{uri}*` — matches any local name in the specified namespace.
                let effective_ns = node_ns.unwrap_or(HTML_NAMESPACE);
                Ok(effective_ns == uri.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_should_parse_simple() {
        // arrange
        let input = "*";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn wildcard_should_parse_prefixed_name() {
        // arrange
        let input = "*:foo";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "*:foo");
    }

    #[test]
    fn wildcard_should_parse_suffixed_name() {
        // arrange
        let input = "foo:*";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "foo:*");
    }

    #[test]
    fn wildcard_should_parse_braced_uri() {
        // arrange
        let input = "Q{http://example.com/ns}*";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
