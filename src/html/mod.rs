mod tokenizer;

use std::{collections::HashMap, iter::Peekable};

use indextree::{Arena, Node, NodeId};
use thiserror::Error;
use tokenizer::Symbol;

pub struct HtmlTag {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

impl HtmlTag {
    pub fn new(name: String) -> HtmlTag {
        HtmlTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

pub enum HtmlNode {
    Tag(HtmlTag),
    Text(String),
}

pub struct HtmlDocument {
    pub arena: Arena<HtmlNode>,
    pub root_key: NodeId,
}

lazy_static! {
    /// List of HTML tags that do not have end tags.
    static ref UNPAIRED_TAGS: Vec<&'static str> = vec![
        "meta",
        "link",
        "img",
        "input",
        "br",
        "hr"
    ];
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Lex error {0}")]
    LexError(#[from] tokenizer::LexError),
    #[error("Start tag encountered before previous tag was closed")]
    StartTagBeforePreviousClosed,
    #[error("Tag close encountered before a tag was opened")]
    TagClosedBeforeOpened,
    #[error("End tag attempted to close a text node")]
    EndTagForTextNode,
    #[error("New end tag encountered before previous tag was closed")]
    EndTagBeforePreviousClosed,
    #[error("End tag name `{end_name}` mismatched open tag name `{open_name}`")]
    EndTagMismatch {
        end_name: String,
        open_name: String
    },
    #[error("Identifier `{identifier}` encountered outside of tag.")]
    IdentifierOutsideTag {
        identifier: String
    },
    #[error("Attempted to add attribute to text node")]
    AttributeOnTextNode,
    #[error("Expected literal after assignment sign {tag_name}")]
    MissingLiteralAfterAssignmentSign {
        tag_name: String
    },
    #[error("Unexpected end of tokens")]
    UnexpectedEndOfTokens,
    
    #[error("Text encountered before previous tag was closed")]
    TextBeforePreviousClosed,

    #[error("No root node found")]
    MissingRootNode,

    #[error("Expected identifier `html` after !DOCSTRING")]
    MissingHtmlAfterDocstring,

    #[error("Expected tag close after !DOCSTRING html")]
    MissingTagCloseAfterDocstring,
}

pub fn parse(text: &str) -> Result<HtmlDocument, ParseError> {
    let tokens = tokenizer::lex(text)?;

    let mut arena: Arena<HtmlNode> = Arena::new();
    let mut root_key_o: Option<NodeId> = None;
    let mut cur_key_o: Option<NodeId> = None;
    let mut has_tag_open = false;

    let mut tokens = tokens.into_iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            Symbol::StartTag(tag_name) => {
                // Skip the special doctype tag so a proper root is selected.
                if is_doctype(&tag_name, &mut tokens)? {
                    continue;
                }

                if has_tag_open {
                    return Err(ParseError::StartTagBeforePreviousClosed);
                }

                has_tag_open = true;

                let node = HtmlNode::Tag(HtmlTag::new(tag_name));
                let node_key = arena.new_node(node);

                if let Some(cur_key) = cur_key_o {
                    cur_key.append(node_key, &mut arena);
                }

                cur_key_o = Some(node_key);
                if root_key_o.is_none() {
                    root_key_o = cur_key_o;
                }
            },
            Symbol::TagClose => {
                if !has_tag_open {
                    return Err(ParseError::TagClosedBeforeOpened);
                }

                // This will be none if the root node was just closed.
                if cur_key_o.is_some() {
                    let cur_tree_node = get_tree_node(cur_key_o, &arena);
                    match cur_tree_node.get() {
                        HtmlNode::Tag(cur_tag) => {
                            // If this is an unpaired tag, the tag begins and ends with this token.
                            if UNPAIRED_TAGS.contains(&cur_tag.name.as_str()) {
                                // Set current key to the parent of this tag.
                                cur_key_o = cur_tree_node.parent();
                            }
                        },
                        HtmlNode::Text(_) => return Err(ParseError::EndTagForTextNode),
                    }
                }
                
                has_tag_open = false;
            },
            Symbol::EndTag(tag_name) => {
                if has_tag_open {
                    return Err(ParseError::EndTagBeforePreviousClosed);
                }

                has_tag_open = true;

                let cur_tree_node = get_tree_node(cur_key_o, &arena);
                match cur_tree_node.get() {
                    HtmlNode::Tag(cur_tag) => {
                        if cur_tag.name != tag_name {
                            return Err(ParseError::EndTagMismatch{
                                end_name: cur_tag.name.to_string(),
                                open_name: tag_name
                            });
                        }
                    },
                    HtmlNode::Text(_) => return Err(ParseError::EndTagForTextNode),
                }

                // Set current key to the parent of this tag.
                cur_key_o = cur_tree_node.parent();
            },
            Symbol::TagCloseAndEnd => {
                if !has_tag_open {
                    return Err(ParseError::TagClosedBeforeOpened);
                }
                
                has_tag_open = false;

                let cur_tree_node = get_tree_node(cur_key_o, &arena);
                if let HtmlNode::Text(_) = cur_tree_node.get() {
                    return Err(ParseError::EndTagForTextNode);
                }

                // Set current key to the parent of this tag.
                cur_key_o = cur_tree_node.parent();
            }
            Symbol::Identifier(iden) => {
                if !has_tag_open {
                    return Err(ParseError::IdentifierOutsideTag {
                        identifier: iden
                    });
                }

                if let Some(token) = tokens.peek() {
                    if let Symbol::AssignmentSign = token {
                        tokens.next();
                        if let Some(token) = tokens.next() {
                            if let Symbol::Literal(lit) = token {
                                let cur_tree_node = get_mut_tree_node(cur_key_o, &mut arena);
                                match cur_tree_node.get_mut() {
                                    HtmlNode::Tag(tag) => {
                                        tag.attributes.insert(iden, lit);
                                    },
                                    HtmlNode::Text(_) => return Err(ParseError::AttributeOnTextNode),
                                }
                            } else {
                                let cur_tree_node = get_mut_tree_node(cur_key_o, &mut arena);
                                match cur_tree_node.get_mut() {
                                    HtmlNode::Tag(tag) => return Err(ParseError::MissingLiteralAfterAssignmentSign {
                                        tag_name: tag.name.to_string()
                                    }),
                                    HtmlNode::Text(_) => return Err(ParseError::AttributeOnTextNode),
                                }
                            }
                        } else {
                            return Err(ParseError::UnexpectedEndOfTokens);
                        }
                    } else {
                        // Attribute has no value; e.g., <script defer></script>
                        let cur_tree_node = get_mut_tree_node(cur_key_o, &mut arena);
                        match cur_tree_node.get_mut() {
                            HtmlNode::Tag(tag) => {
                                tag.attributes.insert(iden, String::from(""));
                            },
                            HtmlNode::Text(_) => return Err(ParseError::AttributeOnTextNode),
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedEndOfTokens);
                }
            },
            Symbol::Text(text) => {
                if has_tag_open {
                    return Err(ParseError::TextBeforePreviousClosed);
                }

                let node = HtmlNode::Text(text);
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

    Err(ParseError::MissingRootNode)
}

fn is_doctype(tag_name: &String, tokens: &mut Peekable<std::vec::IntoIter<Symbol>>) -> Result<bool, ParseError> {
    if tag_name == "!DOCTYPE" {
        if let Some(token) = tokens.next() {
            if let Symbol::Identifier(iden) = token {
                if iden != "html" {
                    return Err(ParseError::MissingHtmlAfterDocstring);
                }
                if let Some(token) = tokens.next() {
                    match token {
                        Symbol::TagClose => {
                            // we good
                        },
                        _ => return Err(ParseError::MissingTagCloseAfterDocstring),
                    }
                } else {
                    return Err(ParseError::UnexpectedEndOfTokens);
                }
            } else {
                return Err(ParseError::MissingHtmlAfterDocstring);
            }
        } else {
            return Err(ParseError::UnexpectedEndOfTokens);
        }
        
        return Ok(true);
    }

    return Ok(false);
}

fn get_tree_node(key: Option<NodeId>, arena: &Arena<HtmlNode>) -> &Node<HtmlNode> {
    let key = key.unwrap();
    let node = arena.get(key);
    return node.unwrap();
}

fn get_mut_tree_node(key: Option<NodeId>, arena: &mut Arena<HtmlNode>) -> &mut Node<HtmlNode> {
    let key = key.unwrap();
    let node = arena.get_mut(key);
    return node.unwrap();
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
            HtmlNode::Text(node_text) => {
                assert_eq!(String::from(text), node_text.trim());
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
    fn parse_should_handle_attributes_without_value() {
        // arrange
        let html = r###"<script defer></script>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        let arena = &result.arena;

        // <script>
        let key = result.root_key;
        let mut attributes = HashMap::new();
        attributes.insert("defer", "");
        assert_tag(arena, key, "script", Some(attributes));
    }

    #[test]
    fn parse_should_handle_attributes_without_value_other_attributes() {
        // arrange
        let html = r###"<script defer src="hi"></script>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        let arena = &result.arena;

        // <script>
        let key = result.root_key;
        let mut attributes = HashMap::new();
        attributes.insert("defer", "");
        attributes.insert("src", "hi");
        assert_tag(arena, key, "script", Some(attributes));
    }

    #[test]
    fn parse_should_handle_single_tags() {
        // arrange
        let html = r###"
        <div>
            <br><hr>
            <meta charset="UTF-8">
            <link rel="stylesheet">
            <img width="500">
            <input type="submit">
        </div>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        let arena = &result.arena;

        // <div>
        let children = assert_tag(arena, result.root_key, "div", None);

        // <div> -> <br>
        {
            let key = children[0];
            assert_tag(arena, key, "br", None);
        }

        // <div> -> <hr>
        {
            let key = children[1];
            assert_tag(arena, key, "hr", None);
        }
        
        // <div> -> <meta>
        {
            let key = children[2];
            let mut attributes = HashMap::new();
            attributes.insert("charset", "UTF-8");
            assert_tag(arena, key, "meta", Some(attributes));
        }

        // <div> -> <link>
        {
            let key = children[3];
            let mut attributes = HashMap::new();
            attributes.insert("rel", "stylesheet");
            assert_tag(arena, key, "link", Some(attributes));
        }

        // <div> -> <img>
        {
            let key = children[4];
            let mut attributes = HashMap::new();
            attributes.insert("width", "500");
            assert_tag(arena, key, "img", Some(attributes));
        }

        // <div> -> <input>
        {
            let key = children[5];
            let mut attributes = HashMap::new();
            attributes.insert("type", "submit");
            assert_tag(arena, key, "input", Some(attributes));
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
