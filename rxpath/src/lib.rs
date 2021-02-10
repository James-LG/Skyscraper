use std::collections::HashMap;

use indextree::{Arena, NodeId};

pub struct RXTag {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

impl RXTag {
    pub fn new(name: String) -> RXTag {
        RXTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

pub enum RXNode {
    Tag(RXTag),
    Text(String),
}

pub struct RXDocument {
    pub arena: Arena<RXNode>,
    pub root_key: NodeId,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
