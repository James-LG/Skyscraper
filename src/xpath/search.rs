use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::DocumentNodeSet,
};

use super::{ApplyError, XpathAxes, XpathSearchItem, XpathSearchNodeType};

/// Search for an HTML tag matching the given search parameters in the given list of nodes.
///
/// # Example: search for a root node using `has_super_root`
/// ```rust
/// # use std::error::Error;
/// use skyscraper::{html, xpath::{self, XpathSearchItem, XpathSearchNodeType, DocumentNodeSet}};
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // Parse the html text into a document.
/// let html_text = r##"
/// <div>
///     <span>Hello</span>
///     <span>world</span>
/// </div>
/// "##;
/// let document = html::parse(html_text)?;
///
/// let search_params = XpathSearchItem {
///     search_node_type: XpathSearchNodeType::Element(String::from("div")),
///     ..Default::default()
/// };
/// let result = xpath::search(
///     &search_params,
///     &document,
///     &DocumentNodeSet::new(true),
/// ).unwrap();
///
/// assert_eq!(1, result.len());
///
/// let html_node = document.get_html_node(&result[0]).unwrap();
/// let tag = html_node.unwrap_tag();
/// assert_eq!("div", tag.name);
///
/// # Ok(())
/// # }
/// ```
///
/// # Example: search for child nodes node by starting from specified node
/// ```rust
/// # use std::error::Error;
/// #[macro_use]
/// extern crate indexmap;
///
/// use skyscraper::{html, xpath::{self, XpathSearchItem, XpathSearchNodeType, DocumentNodeSet}};
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // Parse the html text into a document.
/// let html_text = r##"
/// <div>
///     <span>Hello</span>
///     <span>world</span>
/// </div>
/// "##;
/// let document = html::parse(html_text)?;
///
/// let search_params = XpathSearchItem {
///     search_node_type: XpathSearchNodeType::Element(String::from("span")),
///     ..Default::default()
/// };
/// let result = xpath::search(
///     &search_params,
///     &document,
///     &DocumentNodeSet::from(indexset![document.root_node]),
/// ).unwrap();
///
/// assert_eq!(2, result.len());
///
/// # Ok(())
/// # }
/// ```
///
/// # Example: searching with an index and axis
/// ```rust
/// # use std::error::Error;
///
/// use skyscraper::{html, xpath::{self, XpathSearchItem, XpathSearchNodeType, DocumentNodeSet, XpathAxes}};
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // Parse the html text into a document.
/// let html_text = r##"
/// <div>
///     <span>Hello</span>
///     <span>world</span>
/// </div>
/// "##;
/// let document = html::parse(html_text)?;
///
/// let search_params = XpathSearchItem {
///     search_node_type: XpathSearchNodeType::Element(String::from("span")),
///     axis: XpathAxes::DescendantOrSelf,
///     index: Some(2), // Xpath indexes are 1-based
///     ..Default::default()
/// };
/// let result = xpath::search(
///     &search_params,
///     &document,
///     &DocumentNodeSet::new(true),
/// ).unwrap();
///
/// assert_eq!(1, result.len());
///
/// let html_node = document.get_html_node(&result[0]).unwrap();
/// let tag = html_node.unwrap_tag();
/// assert_eq!("span", tag.name);
///
/// let text = result[0].get_text(&document).unwrap();
/// assert_eq!("world", text);
///
/// # Ok(())
/// # }
/// ```
pub fn search(
    search_item: &XpathSearchItem,
    document: &HtmlDocument,
    searchable_nodes: &DocumentNodeSet,
) -> Result<DocumentNodeSet, ApplyError> {
    let searchable_nodes = apply_axis(document, &search_item.axis, searchable_nodes);

    let mut matches = DocumentNodeSet::new(searchable_nodes.has_super_root);

    for node_id in searchable_nodes.iter() {
        if let Some(node) = document.get_html_node(node_id) {
            match &search_item.search_node_type {
                XpathSearchNodeType::Element(tag_name) => match node {
                    HtmlNode::Tag(rtag) => {
                        if &rtag.name == tag_name {
                            if let Some(query) = &search_item.query {
                                if query.check_node(rtag) {
                                    matches.insert(*node_id);
                                }
                            } else {
                                matches.insert(*node_id);
                            }
                        }
                    }
                    HtmlNode::Text(_) => continue,
                },
                XpathSearchNodeType::Any => {
                    matches.insert(*node_id);
                }
            }
        }
    }

    // Apply indexing if required
    if let Some(i) = search_item.index {
        
        if i < matches.len() {
            let indexed_node = matches[i - 1];
            matches.retain(|node| *node == indexed_node);
        }
    }

    Ok(matches)
}
fn apply_axis(
    document: &HtmlDocument,
    axis: &XpathAxes,
    searchable_nodes: &DocumentNodeSet,
) -> DocumentNodeSet {
    match axis {
        XpathAxes::Child => get_all_children(document, searchable_nodes),
        XpathAxes::DescendantOrSelf => get_all_descendants_or_self(document, searchable_nodes),
        XpathAxes::Parent => get_all_parents(document, searchable_nodes),
    }
}

/// Get all the children for all the given matched nodes.
fn get_all_children(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    let mut child_nodes = DocumentNodeSet::new(false);

    if matched_nodes.has_super_root {
        // Pretend a super root item *above* the official document root is included
        // in the given matched_nodes. Meaning we must now move down to the official document
        // root as it is a child of the super root.
        child_nodes.insert(document.root_node);
    }

    for node_id in matched_nodes {
        let children = node_id.children(document);
        child_nodes.insert_all(children);
    }

    child_nodes
}

/// Get all descendants and self for all the given matched nodes.
fn get_all_descendants_or_self(
    document: &HtmlDocument,
    matched_nodes: &DocumentNodeSet,
) -> DocumentNodeSet {
    fn internal_get_all_descendants_or_self(
        node_id: DocumentNode,
        document: &HtmlDocument,
        descendant_or_self_nodes: &mut DocumentNodeSet,
    ) {
        descendant_or_self_nodes.insert(node_id);
        let mut children: DocumentNodeSet = node_id.children(document).collect();
        if !children.is_empty() {
            children.insert_all(get_all_descendants_or_self(document, &children).into_iter())
        }
        descendant_or_self_nodes.insert_all(children.into_iter());
    }

    let mut descendant_or_self_nodes = DocumentNodeSet::new(matched_nodes.has_super_root);

    if matched_nodes.has_super_root {
        // Pretend a super root item *above* the official document root is included
        // in the given matched_nodes. Meaning we must now move down to the official document
        // root as it is a child of the super root.
        internal_get_all_descendants_or_self(
            document.root_node,
            document,
            &mut descendant_or_self_nodes,
        );
    }

    for node_id in matched_nodes {
        internal_get_all_descendants_or_self(*node_id, document, &mut descendant_or_self_nodes);
    }

    descendant_or_self_nodes
}

/// Get all the parents for all the given matched nodes.
fn get_all_parents(document: &HtmlDocument, matched_nodes: &DocumentNodeSet) -> DocumentNodeSet {
    // If the matched nodes currently contains the document root, then we must move up to the super root.
    let has_super_root = matched_nodes.contains(&document.root_node);

    let mut parent_nodes = DocumentNodeSet::new(has_super_root);
    for node_id in matched_nodes {
        if let Some(parent) = node_id.parent(document) {
            parent_nodes.insert(parent);
        }
    }

    parent_nodes
}

#[cfg(test)]
mod tests {
    use crate::{
        html,
        xpath::{XpathPredicate, XpathQuery},
    };

    use super::*;

    #[test]
    fn search_root_works() {
        let text = r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a/></b>
            <c><b><a/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("root")),
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::new(true)).unwrap();

        assert_eq!(1, result.len());
        let node = document.get_html_node(&result[0]).unwrap();

        let tag = node.unwrap_tag();
        assert_eq!("root", tag.name.as_str());
    }

    #[test]
    fn search_should_apply_index() {
        let text = r###"<!DOCTYPE html>
        <root>
            <node>1</node>
            <node>2</node>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("node")),
            axis: XpathAxes::DescendantOrSelf,
            index: Some(2),
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::new(true)).unwrap();

        assert_eq!(1, result.len());
        let node = document.get_html_node(&result[0]).unwrap();

        let tag = node.unwrap_tag();
        assert_eq!("node", tag.name);

        let text = result[0].get_text(&document).unwrap();
        assert_eq!("2", text);
    }

    #[test]
    fn search_all_works() {
        let text = r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a/></b>
            <c><b><a/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::new(true)).unwrap();

        assert_eq!(3, result.len());

        for r in result {
            let node = document.get_html_node(&r).unwrap();

            let tag = node.unwrap_tag();
            assert_eq!("a", tag.name);
        }
    }

    #[test]
    fn search_all_should_only_select_with_matching_attribute_predicate() {
        let text = r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a/></b>
            <c><b><a hello="world"/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let query = XpathQuery {
            predicates: vec![XpathPredicate::Equals {
                attribute: String::from("hello"),
                value: String::from("world"),
            }],
        };
        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            query: Some(query),
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::new(true)).unwrap();

        assert_eq!(1, result.len());

        let node = document.get_html_node(&result[0]).unwrap();

        let tag = node.unwrap_tag();
        assert_eq!("a", tag.name.as_str());
        assert!(tag.attributes["hello"] == String::from("world"));
    }

    #[test]
    fn search_all_should_only_select_with_multiple_matching_attribute_predicate() {
        let text = r###"<!DOCTYPE html>
        <root>
            <a/>
            <b><a hello="world"/></b>
            <c><b><a hello="world" foo="bar"/></b></c>
        </root>
        "###;

        let document = html::parse(text).unwrap();

        let query = XpathQuery {
            predicates: vec![
                XpathPredicate::Equals {
                    attribute: String::from("hello"),
                    value: String::from("world"),
                },
                XpathPredicate::Equals {
                    attribute: String::from("foo"),
                    value: String::from("bar"),
                },
            ],
        };
        let search_params = XpathSearchItem {
            search_node_type: XpathSearchNodeType::Element(String::from("a")),
            axis: XpathAxes::DescendantOrSelf,
            query: Some(query),
            ..Default::default()
        };
        let result = search(&search_params, &document, &DocumentNodeSet::new(true)).unwrap();

        assert_eq!(1, result.len());

        let node = document.get_html_node(&result[0]).unwrap();

        let tag = node.unwrap_tag();
        assert_eq!("a", tag.name.as_str());
        assert!(tag.attributes["hello"] == String::from("world"));
    }
}
