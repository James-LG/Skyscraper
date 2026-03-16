//! An ordered sequence of [`XpathItem`]s.
//!
//! Unlike a set, XPath sequences may contain duplicate values.

use std::collections::HashSet;
use std::ops::Index;

use super::grammar::data_model::{AnyAtomicType, XpathItem};
use super::ExpressionApplyError;

/// An ordered sequence of [`XpathItem`]s.
///
/// XPath sequences are ordered and may contain duplicates.
#[derive(Debug, Clone)]
pub struct XpathItemSet<'tree> {
    items: Vec<XpathItem<'tree>>,
}

impl PartialEq for XpathItemSet<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<'a, 'tree> IntoIterator for &'a XpathItemSet<'tree> {
    type Item = &'a XpathItem<'tree>;

    type IntoIter = std::slice::Iter<'a, XpathItem<'tree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'tree> IntoIterator for XpathItemSet<'tree> {
    type Item = XpathItem<'tree>;

    type IntoIter = std::vec::IntoIter<XpathItem<'tree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'tree> FromIterator<XpathItem<'tree>> for XpathItemSet<'tree> {
    fn from_iter<T: IntoIterator<Item = XpathItem<'tree>>>(iter: T) -> Self {
        XpathItemSet {
            items: Vec::from_iter(iter),
        }
    }
}

impl<'tree> Extend<XpathItem<'tree>> for XpathItemSet<'tree> {
    fn extend<T: IntoIterator<Item = XpathItem<'tree>>>(&mut self, iter: T) {
        self.items.extend(iter)
    }
}

impl Default for XpathItemSet<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'tree> XpathItemSet<'tree> {
    /// Create a new empty [`XpathItemSet`].
    pub fn new() -> Self {
        XpathItemSet { items: Vec::new() }
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The number of items in the set.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Inserts a new item into the sequence.
    pub fn insert(&mut self, item: XpathItem<'tree>) {
        self.items.push(item);
    }

    /// Return an iterator over the items in the sequence.
    pub fn iter(&self) -> std::slice::Iter<'_, XpathItem<'tree>> {
        self.items.iter()
    }

    /// Returns `true` if the sequence contains the given item.
    pub fn contains(&self, item: &XpathItem<'tree>) -> bool {
        self.items.contains(item)
    }

    /// Return the effective boolean value of the result.
    ///
    /// <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-ebv>
    pub fn boolean(&self) -> Result<bool, ExpressionApplyError> {
        if self.items.len() == 1 {
            match &self.items[0] {
                XpathItem::Node(_) => Ok(true),
                XpathItem::Function(_) => Err(ExpressionApplyError::new(
                    "err:FORG0006: effective boolean value is not defined for function items"
                        .to_string(),
                )),
                XpathItem::AnyAtomicType(atomic_type) => match atomic_type {
                    AnyAtomicType::Boolean(b) => Ok(*b),
                    AnyAtomicType::Integer(n) => Ok(*n != 0),
                    AnyAtomicType::Float(n) => Ok(!n.is_nan() && *n != 0.0),
                    AnyAtomicType::Double(n) => Ok(!n.is_nan() && *n != 0.0),
                    AnyAtomicType::String(s) => Ok(!s.is_empty()),
                    AnyAtomicType::QName { .. } => Err(ExpressionApplyError::new(
                        "err:FORG0006: effective boolean value is not defined for QName"
                            .to_string(),
                    )),
                },
            }
        } else if self.items.is_empty() {
            Ok(false)
        } else {
            // Multi-item sequence: EBV is true only if first item is a node.
            match &self.items[0] {
                XpathItem::Node(_) => Ok(true),
                _ => Err(ExpressionApplyError::new(
                    "err:FORG0006: effective boolean value is not defined for a sequence starting with a non-node item"
                        .to_string(),
                )),
            }
        }
    }

    /// Reverse the order of items in the sequence.
    pub(crate) fn reverse(&mut self) {
        self.items.reverse();
    }

    /// Sort items by document order (arena NodeId).
    ///
    /// DocumentNode (which has no NodeId) sorts first since it represents the
    /// document root. Other nodes sort by NodeId (document order). Non-node
    /// items retain their relative order at the end of the sequence.
    pub(crate) fn sort_by_document_order(&mut self) {
        use super::grammar::XpathItemTreeNode;

        // Sort key: (priority, Option<NodeId>)
        // 0 = DocumentNode (sorts first), 1 = other nodes, 2 = non-nodes
        let sort_key = |item: &XpathItem| -> (u8, Option<indextree::NodeId>) {
            match item {
                XpathItem::Node(node) => match node {
                    XpathItemTreeNode::DocumentNode(_) => (0, None),
                    _ => (1, node.node_id()),
                },
                _ => (2, None),
            }
        };

        self.items.sort_by(|a, b| {
            let (a_pri, a_id) = sort_key(a);
            let (b_pri, b_id) = sort_key(b);
            match a_pri.cmp(&b_pri) {
                std::cmp::Ordering::Equal => match (a_id, b_id) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    _ => std::cmp::Ordering::Equal,
                },
                other => other,
            }
        });
    }

    /// Remove duplicate items, keeping the first occurrence of each.
    /// Uses node identity (NodeId) for Node items to correctly distinguish
    /// structurally equal nodes at different positions in the tree.
    pub(crate) fn dedup(&mut self) {
        let mut seen_node_ids: HashSet<Option<indextree::NodeId>> = HashSet::new();
        let mut seen_non_nodes = HashSet::new();
        self.items.retain(|item| match item {
            XpathItem::Node(node) => seen_node_ids.insert(node.node_id()),
            other => seen_non_nodes.insert(other.clone()),
        });
    }
}

impl<'tree> From<Vec<XpathItem<'tree>>> for XpathItemSet<'tree> {
    fn from(value: Vec<XpathItem<'tree>>) -> Self {
        XpathItemSet { items: value }
    }
}

impl<'tree> Index<usize> for XpathItemSet<'tree> {
    type Output = XpathItem<'tree>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

/// Create an [XpathItemSet] from a list of values
#[macro_export]
macro_rules! xpath_item_set {
    ($($value:expr,)+) => { $crate::xpath::xpath_item_set::xpath_item_set!($($value),+) };
    ($($value:expr),*) => {
        {
            let items: Vec<$crate::xpath::grammar::data_model::XpathItem> = vec![$($value,)*];
            $crate::xpath::XpathItemSet::from(items)
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::xpath::grammar::data_model::AnyAtomicType;

    use super::*;

    #[test]
    fn macro_works_with_one() {
        // arrange
        let node1 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("1")));

        // act
        let item_set = xpath_item_set![node1.clone()];

        // assert
        let mut expected = XpathItemSet::new();
        expected.insert(node1);

        assert_eq!(item_set, expected);
    }

    #[test]
    fn macro_works_with_multiple() {
        // arrange
        let node1 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("1")));
        let node2 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("2")));
        let node3 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("3")));

        // act
        let item_set = xpath_item_set![node1.clone(), node2.clone(), node3.clone()];

        // assert
        let mut expected = XpathItemSet::new();
        expected.insert(node1);
        expected.insert(node2);
        expected.insert(node3);

        assert_eq!(item_set, expected);
    }

    #[test]
    fn duplicates_are_preserved() {
        let val = XpathItem::AnyAtomicType(AnyAtomicType::Integer(1));
        let mut set = XpathItemSet::new();
        set.insert(val.clone());
        set.insert(val.clone());
        set.insert(val.clone());
        assert_eq!(set.len(), 3);
    }
}
