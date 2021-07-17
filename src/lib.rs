#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use indextree::{Arena, NodeId};

pub mod xpath;
pub mod html;
mod textpointer;

pub struct RTag {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

impl RTag {
    pub fn new(name: String) -> RTag {
        RTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

pub enum RNode {
    Tag(RTag),
    Text(String),
}

pub struct RDocument {
    pub arena: Arena<RNode>,
    pub root_key: NodeId,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
