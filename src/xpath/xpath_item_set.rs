use std::ops::Index;

use indexmap::{self, IndexSet};

use super::grammar::data_model::XpathItem;

#[derive(Debug)]
pub struct XpathItemSet<'tree> {
    index_set: IndexSet<XpathItem<'tree>>,
}

impl PartialEq for XpathItemSet<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.index_set == other.index_set
    }
}

impl PartialOrd for XpathItemSet<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index_set.iter().partial_cmp(&other.index_set)
    }
}

impl<'a, 'tree> IntoIterator for &'a XpathItemSet<'tree> {
    type Item = &'a XpathItem<'tree>;

    type IntoIter = indexmap::set::Iter<'a, XpathItem<'tree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.index_set.iter()
    }
}

impl<'tree> IntoIterator for XpathItemSet<'tree> {
    type Item = XpathItem<'tree>;

    type IntoIter = indexmap::set::IntoIter<XpathItem<'tree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.index_set.into_iter()
    }
}

impl<'tree> FromIterator<XpathItem<'tree>> for XpathItemSet<'tree> {
    fn from_iter<T: IntoIterator<Item = XpathItem<'tree>>>(iter: T) -> Self {
        let index_set = IndexSet::from_iter(iter);
        XpathItemSet { index_set }
    }
}

impl<'tree> Extend<XpathItem<'tree>> for XpathItemSet<'tree> {
    fn extend<T: IntoIterator<Item = XpathItem<'tree>>>(&mut self, iter: T) {
        self.index_set.extend(iter)
    }
}

impl<'tree> XpathItemSet<'tree> {
    pub fn new() -> Self {
        XpathItemSet {
            index_set: IndexSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.index_set.is_empty()
    }

    pub fn len(&self) -> usize {
        self.index_set.len()
    }

    pub fn insertb(&mut self, item: XpathItem<'tree>) -> bool {
        self.index_set.insert(item)
    }

    /// Inserts a new item into the set.
    ///
    /// Drops the bool return by [XpathItemSet::insertb] so that it can be used in match arms
    /// without causing incompatible types with [XpathItemSet::extend].
    pub fn insert(&mut self, item: XpathItem<'tree>) {
        self.insertb(item);
    }

    pub fn iter(&self) -> indexmap::set::Iter<'_, XpathItem<'tree>> {
        self.index_set.iter()
    }
}

impl<'tree> From<IndexSet<XpathItem<'tree>>> for XpathItemSet<'tree> {
    fn from(value: IndexSet<XpathItem<'tree>>) -> Self {
        XpathItemSet { index_set: value }
    }
}

impl<'tree> Index<usize> for XpathItemSet<'tree> {
    type Output = XpathItem<'tree>;

    fn index(&self, index: usize) -> &Self::Output {
        self.index_set.index(index)
    }
}

/// Create an [XpathItemSet] from a list of values
#[macro_export]
macro_rules! xpath_item_set {
    ($($value:expr,)+) => { $crate::xpath::xpath_item_set::xpath_item_set!($($value),+) };
    ($($value:expr),*) => {
        {
            let set = indexset![$($value)*];

            XpathItemSet::from(set)
        }
    };
}
