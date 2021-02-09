#[macro_use]
extern crate lazy_static;

use std::{collections::HashMap, error::Error};
use indextree::{Arena, Node, NodeId};

mod tokenizer;

use tokenizer::Symbol;

pub struct HtmlTag {
    name: String,
    attributes: HashMap<String, String>,
}

impl HtmlTag {
    fn new(name: String) -> HtmlTag {
        HtmlTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

pub struct HtmlText {
    text: String,
}

pub enum HtmlNode {
    Tag(HtmlTag),
    Text(HtmlText),
}

pub struct HtmlDocument {
    arena: Arena<HtmlNode>,
    root_key: NodeId,
}

pub fn parse(text: &str) -> Result<HtmlDocument, Box<dyn Error>> {
    let tokens = tokenizer::lex(text)?;

    let mut arena: Arena<HtmlNode> = Arena::new();
    let mut root_key_o: Option<NodeId> = None;
    let mut cur_key_o: Option<NodeId> = None;
    let mut has_tag_open = false;

    let mut tokens = tokens.into_iter();

    while let Some(token) = tokens.next() {
        match token {
            Symbol::StartTag(tag_name) => {
                // Skip the special doctype tag so a proper root is selected.
                if tag_name == "!DOCTYPE" {
                    match tokens.next() {
                        Some(token) => {
                            match token {
                                Symbol::Identifier(iden) => {
                                    if iden != String::from("html") {
                                        return Err("Expected identifier `html` after !DOCSTRING.".into());
                                    }
                                    match tokens.next() {
                                        Some(token) => {
                                            match token {
                                                Symbol::TagClose => {
                                                    // we good
                                                },
                                                _ => return Err("Expected tag close after !DOCSTRING html".into()),
                                            }
                                        },
                                        None => return Err("Unexpected end of tokens.".into()),
                                    }
                                },
                                _ => return Err("Expected identifier `html` after !DOCSTRING.".into()),
                            }
                        },
                        None => return Err("Unexpected end of tokens.".into()),
                    }
                    continue;
                }

                if has_tag_open {
                    return Err("Start tag encountered before previous tag was closed.".into());
                }

                has_tag_open = true;

                let node = HtmlNode::Tag(HtmlTag::new(tag_name));
                let node_key = arena.new_node(node);

                if let Some(cur_key) = cur_key_o {
                    cur_key.append(node_key, &mut arena);
                }

                cur_key_o = Some(node_key);
                if let None = root_key_o {
                    root_key_o = cur_key_o;
                }
            },
            Symbol::TagClose => {
                if !has_tag_open {
                    return Err("Tag close encountered before a tag was opened.".into());
                }

                // If `meta` tag, it ends with this token.
                // This will be none if the root node was just closed.
                if let Some(_) = cur_key_o {
                    let cur_tree_node = try_get_tree_node(cur_key_o, &arena)?;
                    match cur_tree_node.get() {
                        HtmlNode::Tag(cur_tag) => {
                            if cur_tag.name == String::from("meta") {
                                // Set current key to the parent of this tag.
                                cur_key_o = cur_tree_node.parent();
                            }
                        },
                        HtmlNode::Text(_) => return Err("End tag attempted to close a text node.".into()),
                    }
                }
                
                has_tag_open = false;
            },
            Symbol::EndTag(tag_name) => {
                if has_tag_open {
                    return Err("End tag encountered before previous tag was closed.".into());
                }

                has_tag_open = true;

                let cur_tree_node = try_get_tree_node(cur_key_o, &arena)?;
                match cur_tree_node.get() {
                    HtmlNode::Tag(cur_tag) => {
                        if cur_tag.name != tag_name {
                            return Err(format!("End tag name `{}` mismatched open tag name `{}`.", tag_name, cur_tag.name).into());
                        }
                    },
                    HtmlNode::Text(_) => return Err("End tag attempted to close a text node.".into()),
                }

                // Set current key to the parent of this tag.
                cur_key_o = cur_tree_node.parent();
            },
            Symbol::TagCloseAndEnd => {
                if !has_tag_open {
                    return Err("Tag close encountered before a tag was opened.".into());
                }
                
                has_tag_open = false;

                let cur_tree_node = try_get_tree_node(cur_key_o, &arena)?;
                if let HtmlNode::Text(_) = cur_tree_node.get() {
                    return Err("End tag attempted to close a text node.".into());
                }

                // Set current key to the parent of this tag.
                cur_key_o = cur_tree_node.parent();
            }
            Symbol::Identifier(iden) => {
                if !has_tag_open {
                    return Err(format!("Identifier `{}` encountered outside of tag.", iden).into());
                }

                match tokens.next() {
                    Some(token) => {
                        match token {
                            Symbol::AssignmentSign => {
                                match tokens.next() {
                                    Some(token) => {
                                        match token {
                                            Symbol::Literal(lit) => {
                                                let cur_tree_node = try_get_mut_tree_node(cur_key_o, &mut arena)?;
                                                match cur_tree_node.get_mut() {
                                                    HtmlNode::Tag(tag) => {
                                                        tag.attributes.insert(iden, lit);
                                                    },
                                                    HtmlNode::Text(_) => return Err("Attempted to add attribute to text node.".into()),
                                                }
                                            },
                                            _ => return Err("Expected literal after assignment sign.".into()),
                                        }
                                    },
                                    None => return Err("Unexpected end of tokens.".into()),
                                }
                            },
                            _ => return Err("Expected assignment sign after identifier.".into()),
                        }
                    },
                    None => return Err("Unexpected end of tokens.".into()),
                }
            },
            Symbol::Text(text) => {
                if has_tag_open {
                    return Err("Text encountered before previous tag was closed.".into());
                }

                let node = HtmlNode::Text(HtmlText { text });
                let node_key = arena.new_node(node);

                if let Some(cur_key) = cur_key_o {
                    cur_key.append(node_key, &mut arena);
                }
            }
            _ => (),
        }
    }

    if let Some(root_key) = root_key_o {
        return Ok(HtmlDocument {
            arena,
            root_key,
        });
    }

    Err("No root node found.".into())
}

fn try_get_tree_node(key: Option<NodeId>, arena: &Arena<HtmlNode>) -> Result<&Node<HtmlNode>, &'static str> {
    match key {
        Some(key) => {
            match arena.get(key) {
                Some(node) => Ok(node),
                None => Err("Could not get tree node from arena."),
            }
        },
        None => Err("Unexpected None key."),
    }
}

fn try_get_mut_tree_node(key: Option<NodeId>, arena: &mut Arena<HtmlNode>) -> Result<&mut Node<HtmlNode>, &'static str> {
    match key {
        Some(key) => {
            match arena.get_mut(key) {
                Some(node) => Ok(node),
                None => Err("Could not get tree node from arena."),
            }
        },
        None => Err("Unexpected None key."),
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tag(arena: &Arena<HtmlNode>, key: NodeId, tag_name: &str, attributes: Option<HashMap<&str, &str>>) -> Vec<NodeId> {
        let tree_node = arena.get(key).unwrap();
        let html_node = tree_node.get();

        match html_node {
            HtmlNode::Tag(tag) => {
                assert_eq!(String::from(tag_name), tag.name);

                if let Some(attributes) = attributes {
                    for attr in attributes {
                        assert_eq!(&String::from(attr.1), tag.attributes.get(attr.0).unwrap());
                    }
                }

                return key.children(&arena).collect();
            },
            _ => panic!("Expected Tag, got different variant instead."),
        }
    }

    fn assert_text(arena: &Arena<HtmlNode>, key: NodeId, text: &str) {
        let tree_node = arena.get(key).unwrap();
        let html_node = tree_node.get();

        match html_node {
            HtmlNode::Text(text_node) => {
                assert_eq!(String::from(text), text_node.text.trim());
            },
            _ => panic!("Expected Text, got different variant instead."),
        }
    }

    #[test]
    fn parse_works() {
        // arrange
        let text = "<html><a class=\"beans\"></a><b><ba/>yo</b></html>";

        // act
        let result = parse(text).unwrap();

        // assert
        let arena = &result.arena;

        // <html>
        let children = assert_tag(arena, result.root_key, "html", None);

        // <html> -> <a class="beans">
        {
            let key = children[0];
            let mut attributes = HashMap::new();
            attributes.insert("class", "beans");
            assert_tag(arena, key, "a", Some(attributes));
        }

        // <html> -> <b>
        {
            let key = children[1];
            let children = assert_tag(arena, key, "b", None);

            // <html> -> <b> -> <ba>
            {
                let key = children[0];
                assert_tag(arena, key, "ba", None);
            }

            // <html> -> <b> -> text()
            {
                let key = children[1];
                assert_text(arena, key, "yo");
            }
        }
    }

    #[test]
    fn parse_should_work_with_html() {
        // arrange
        let html = r###"<!DOCTYPE html>
        <!-- saved from url=(0026)https://www.rust-lang.org/ -->
        <html lang="en-US">
            <head>
                <title>Rust Programming Language</title>
                <meta name="viewport" content="width=device-width,initial-scale=1.0">
        
                <!-- Twitter card -->
                <meta name="twitter:card" content="summary">
            </head>
            <body>
                <main>
                    <section id="language-values" class="green">
                        <div class="w-100 mw-none ph3 mw8-m mw9-l center f3">
                            <header class="pb0">
                                <h2>
                                Why Rust?
                                </h2>
                            </header>
                            <div class="flex-none flex-l">
                                <section class="w-100 pv2 pv0-l mt4">
                                    <h3 class="f2 f1-l">Performance</h3>
                                    <p class="f3 lh-copy">
                                    Rust is blazingly fast and memory-efficient: with no runtime or
                                    garbage collector, it can power performance-critical services, run on
                                    embedded devices, and easily integrate with other languages.
                                    </p>
                                </section>
                            </div>
                        </div>
                    </section>
                </main>
                <script src="./Rust Programming Language_files/languages.js.download"/>
            </body>
        </html>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        let arena = &result.arena;

        // <html>
        let children = assert_tag(arena, result.root_key, "html", None);

        // <html> -> <head>
        {
            let key = children[0];
            let children = assert_tag(arena, key, "head", None);

            // <html> -> <head> -> <title>
            {
                let key = children[0];
                let children = assert_tag(arena, key, "title", None);

                // <html> -> head> -> <title> -> text()
                {
                    let key = children[0];
                    assert_text(arena, key, "Rust Programming Language");
                }
            }

            // <html> -> <head> -> <meta name="viewport" content="width=device-width,initial-scale=1.0">
            {
                let key = children[1];
                let mut attributes = HashMap::new();
                attributes.insert("name", "viewport");
                attributes.insert("content", "width=device-width,initial-scale=1.0");
                assert_tag(arena, key, "meta", Some(attributes));
            }

            // <html> -> <head> -> <meta name="twitter:card" content="summary">
            {
                let key = children[2];
                let mut attributes = HashMap::new();
                attributes.insert("name", "twitter:card");
                attributes.insert("content", "summary");
                assert_tag(arena, key, "meta", Some(attributes));
            }
        }

        // <html> -> <body>
        {
            let key = children[1];
            let children = assert_tag(arena, key, "body", None);

            // <html> -> <body> -> <main>
            {
                let key = children[0];
                let children = assert_tag(arena, key, "main", None);

                // <html> -> <body> -> <main> -> <section id="language-values" class="green">
                {
                    let key = children[0];
                    let mut attributes = HashMap::new();
                    attributes.insert("id", "language-values");
                    attributes.insert("class", "green");
                    let children = assert_tag(arena, key, "section", Some(attributes));

                    // <html> -> <body> -> <main> -> <section> -> <div class="w-100 mw-none ph3 mw8-m mw9-l center f3">
                    {
                        let key = children[0];
                        let mut attributes = HashMap::new();
                        attributes.insert("class", "w-100 mw-none ph3 mw8-m mw9-l center f3");
                        let children = assert_tag(arena, key, "div", Some(attributes));

                        // <html> -> <body> -> <main> -> <section> -> <div> -> <header class="pb0">
                        {
                            let key = children[0];
                            let mut attributes = HashMap::new();
                            attributes.insert("class", "pb0");
                            let children = assert_tag(arena, key, "header", Some(attributes));

                            // <html> -> <body> -> <main> -> <section> -> <div> -> <header> -> <h2>
                            {
                                let key = children[0];
                                let children = assert_tag(arena, key, "h2", None);

                                // <html> -> <body> -> <main> -> <section> -> <div> -> <header> -> <h2> -> text()
                                {
                                    let key = children[0];
                                    assert_text(arena, key, "Why Rust?")
                                }
                            }
                        }

                        // <html> -> <body> -> <main> -> <section> -> <div> -> <div class="flex-none flex-l">
                        {
                            let key = children[1];
                            let mut attributes = HashMap::new();
                            attributes.insert("class", "flex-none flex-l");
                            let children = assert_tag(arena, key, "div", Some(attributes));

                            // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section class="w-100 pv2 pv0-l mt4">
                            {
                                let key = children[0];
                                let mut attributes = HashMap::new();
                                attributes.insert("class", "w-100 pv2 pv0-l mt4");
                                let children = assert_tag(arena, key, "section", Some(attributes));

                                // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <h3 class="f2 f1-l">
                                {
                                    let key = children[0];
                                    let mut attributes = HashMap::new();
                                    attributes.insert("class", "f2 f1-l");
                                    let children = assert_tag(arena, key, "h3", Some(attributes));

                                    // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <h3> -> text()
                                    {
                                        let key = children[0];
                                        assert_text(arena, key, "Performance");
                                    }
                                }

                                // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <p class="f3 lh-copy">
                                {
                                    let key = children[1];
                                    let mut attributes = HashMap::new();
                                    attributes.insert("class", "f3 lh-copy");
                                    let children = assert_tag(arena, key, "p", Some(attributes));

                                    // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <p> -> text()
                                    {
                                        let key = children[0];
                                        assert_text(arena, key, r###"Rust is blazingly fast and memory-efficient: with no runtime or
                                    garbage collector, it can power performance-critical services, run on
                                    embedded devices, and easily integrate with other languages."###);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // <html> -> <body> -> <script src="./Rust Programming Language_files/languages.js.download"/>
            {
                let key = children[1];
                let mut attributes = HashMap::new();
                attributes.insert("src", "./Rust Programming Language_files/languages.js.download");
                assert_tag(arena, key, "script", Some(attributes));
            }
        }
    }
}
