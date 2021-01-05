use std::rc::Weak;
use std::collections::HashMap;

struct Node {
    name: String,
    attributes: HashMap<String, String>,
    children: Vec<Node>,
    parent: Weak<Node>,
}

impl Node {
    fn new(text: &str) -> Result<(), &'static String> {
        Ok(())
    }
}

type Document = Vec<DToken>;

enum DToken {
    OpenTag {
        name: String,
        attributes: Vec<String>,
    },
    CloseTag {
        name: String,
    },
    Text(String),
}

fn parse(text: &str) -> Result<(), &'static String> {
    Ok(())
}

#[derive(Debug)]
#[derive(PartialEq)]
enum Token {
    // `<` Used to start tags.
    OpenTriangle,
    // `>` Used to end tags.
    CloseTriangle,
    // `=` Used to define attributes.
    EqualSign,
    // `"` Used to give attribute values.
    Quote,
    // `/` Used to start closing tags.
    Slash,
    // Any other character.
    Char(char),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
