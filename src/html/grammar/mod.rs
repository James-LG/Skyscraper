//! <https://html.spec.whatwg.org/multipage/parsing.html>

use indextree::{Arena, NodeId};
use log::warn;
use nom::error;
use thiserror::Error;
use tokenizer::{HtmlToken, TagToken, TokenizerObserver};

use crate::{
    vecpointer::VecPointerRef,
    xpath::{
        grammar::{
            data_model::{AttributeNode, ElementNode, TextNode, XpathDocumentNode, XpathItem},
            XpathItemTreeNode,
        },
        Xpath, XpathItemTree,
    },
};

mod chars;
mod document_builder;
mod insertion_mode_impls;
mod tokenizer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(Debug)]
pub(crate) enum HtmlParseErrorType {
    AbruptClosingOfEmptyComment,
    AbruptDoctypePublicIdentifier,
    AbruptDoctypeSystemIdentifier,
    AbsenceOfDigitsInNumericCharacterReference,
    CdataInHtmlContent,
    CharacterReferenceOutsideUnicodeRange,
    ControlCharacterInInputStream,
    ControlCharacterReference,
    DuplicateAttribute,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    EofBeforeTagName,
    EofInCdata,
    EofInComment,
    EofInDoctype,
    EofInScriptHtmlCommentLikeText,
    EofInTag,
    IncorrectlyClosedComment,
    IncorrectlyOpenedComment,
    InvalidCharacterSequenceAfterDoctypeName,
    InvalidFirstCharacterOfTagName,
    MissingAttributeValue,
    MissingDoctypeName,
    MissingDoctypePublicIdentifier,
    MissingDoctypeSystemIdentifier,
    MissingEndTagName,
    MissingQuoteBeforeDoctypePublicIdentifier,
    MissingQuoteBeforeDoctypeSystemIdentifier,
    MissingSemicolonAfterCharacterReference,
    MissingWhitespaceAfterDoctypePublicKeyword,
    MissingWhitespaceAfterDoctypeSystemKeyword,
    MissingWhitespaceBeforeDoctypeName,
    MissingWhitespaceBetweenAttributes,
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
    NestedComment,
    NoncharacterCharacterReference,
    NoncharacterInInputStream,
    NonVoidHtmlElementStartTagWithTrailingSolidus,
    NullCharacterReference,
    SurrogateCharacterReference,
    SurrogateInInputStream,
    UnexpectedCharacterAfterDoctypeSystemIdentifier,
    UnexpectedCharacterInAttributeName,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedNullCharacter,
    UnexpectedQuestionMarkInsteadOfTagName,
    UnexpectedSolidusInTag,
    UnknownNamedCharacterReference,
}

#[derive(Debug, Error)]
#[error("parse error: {message}")]
pub struct HtmlParseError {
    pub message: String,
}

impl HtmlParseError {
    pub fn new(message: &str) -> Self {
        HtmlParseError {
            message: message.to_string(),
        }
    }
}

pub fn parse(text: &str) -> Result<XpathItemTree, HtmlParseError> {
    let mut parser = HtmlParser::new();
    parser.parse(text)
}

/// <https://infra.spec.whatwg.org/#html-namespace>
pub(crate) const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
pub(crate) static ELEMENT_IN_SCOPE_TYPES: [&str; 9] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
];
pub(crate) static GENERATE_IMPLIED_END_TAG_TYPES: [&str; 10] = [
    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
];

pub(crate) struct CreateAnElementForTheTokenResult {
    element: ElementNode,
    attributes: Vec<AttributeNode>,
}

pub struct HtmlParser {
    insertion_mode: InsertionMode,
    open_elements: Vec<NodeId>,
    context_element: Option<XpathItemTreeNode>,
    arena: Arena<XpathItemTreeNode>,
    root_node: Option<NodeId>,
    foster_parenting: bool,
    frameset_ok: bool,
    active_formatting_elements: Vec<NodeId>,
    stack_of_template_insertion_modes: Vec<InsertionMode>,
    head_element_pointer: Option<NodeId>,
    form_element_pointer: Option<NodeId>,
}

impl HtmlParser {
    pub fn new() -> Self {
        HtmlParser {
            insertion_mode: InsertionMode::Initial,
            open_elements: Vec::new(),
            context_element: None,
            arena: Arena::new(),
            root_node: None,
            foster_parenting: false,
            frameset_ok: true,
            active_formatting_elements: Vec::new(),
            stack_of_template_insertion_modes: Vec::new(),
            head_element_pointer: None,
            form_element_pointer: None,
        }
    }

    pub fn parse(&mut self, text: &str) -> Result<XpathItemTree, HtmlParseError> {
        let mut open_elements: Vec<XpathItemTreeNode> = Vec::new();

        let chars: Vec<char> = text.chars().collect();
        let input_stream = VecPointerRef::new(&chars);
        let mut tokenizer = tokenizer::Tokenizer::new(input_stream);
        let mut tokenizer_error_handler = tokenizer::DefaultTokenizerErrorHandler;

        let mut error_handler = DefaultParseErrorHandler;

        tokenizer.set_observer(Box::new(self));
        tokenizer.set_error_handler(Box::new(&tokenizer_error_handler));

        while !tokenizer.is_terminated() {
            tokenizer.step()?;
        }

        let arena = std::mem::replace(&mut self.arena, Arena::new());
        let document = XpathItemTree::new(arena, self.root_node.expect("root node not set"));
        Ok(document)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-node>
    pub(crate) fn current_node(&self) -> Option<&XpathItemTreeNode> {
        self.open_elements
            .last()
            .map(|id| self.arena.get(*id).unwrap().get())
    }

    pub(crate) fn current_node_as_element(&self) -> Option<&ElementNode> {
        self.current_node().and_then(|node| match node {
            XpathItemTreeNode::ElementNode(element) => Some(element),
            _ => None,
        })
    }

    pub(crate) fn top_node(&self) -> Option<&XpathItemTreeNode> {
        self.open_elements
            .first()
            .map(|id| self.arena.get(*id).unwrap().get())
    }

    pub(crate) fn top_node_mut(&mut self) -> Option<&mut XpathItemTreeNode> {
        self.open_elements
            .first()
            .map(|id| self.arena.get_mut(*id).unwrap().get_mut())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjusted-current-node>
    pub(crate) fn adjusted_current_node(&self) -> Option<&XpathItemTreeNode> {
        if self.context_element.is_some() && self.open_elements.len() == 1 {
            self.context_element.as_ref()
        } else {
            self.current_node()
        }
    }

    pub(crate) fn new_node(&mut self, node: XpathItemTreeNode) -> NodeId {
        println!("new node: {:?}", node);
        let is_element = node.is_element_node();
        let is_attribute = node.is_attribute_node();
        let id = self.arena.new_node(node);

        if is_element {
            self.arena
                .get_mut(id)
                .unwrap()
                .get_mut()
                .extract_as_element_node_mut()
                .set_id(id);
        } else if is_attribute {
            self.arena
                .get_mut(id)
                .unwrap()
                .get_mut()
                .extract_as_attribute_node_mut()
                .set_id(id);
        }

        id
    }

    pub(crate) fn open_elements_as_nodes(&self) -> Vec<&XpathItemTreeNode> {
        self.open_elements
            .iter()
            .map(|id| self.arena.get(*id).unwrap().get())
            .collect()
    }

    pub(crate) fn handle_error(&self, error: HtmlParserError) -> Result<(), HtmlParseError> {
        match error {
            HtmlParserError::MinorError(err) => {
                warn!("{}", err);
                Ok(())
            }
            HtmlParserError::FatalError(err) => Err(HtmlParseError::new(&err)),
        }
    }

    pub(crate) fn add_attribute_to_element(
        &mut self,
        element_id: NodeId,
        name: String,
        value: String,
    ) -> Result<(), HtmlParseError> {
        let attribute = AttributeNode::new(name, value);
        let item_id = self.new_node(XpathItemTreeNode::AttributeNode(attribute));

        element_id.append(item_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    pub(crate) fn insert_an_html_element(
        &mut self,
        token: TagToken,
    ) -> Result<NodeId, HtmlParseError> {
        self.insert_foreign_element(token, HTML_NAMESPACE, false)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element>
    pub(crate) fn insert_foreign_element(
        &mut self,
        token: TagToken,
        namespace: &str,
        only_add_to_element_stack: bool,
    ) -> Result<NodeId, HtmlParseError> {
        let adjusted_insertion_location = if only_add_to_element_stack {
            None
        } else {
            Some(self.appropriate_place_for_inserting_a_node(None)?)
        };

        let result = self.create_an_element_for_the_token(token, namespace)?;

        // insert the result
        let element_id = self.insert_create_an_element_for_the_token_result(result)?;

        // append the element to the adjusted insertion location
        if let Some(adjusted_insertion_location) = adjusted_insertion_location {
            adjusted_insertion_location.append(element_id, &mut self.arena);
        }

        Ok(element_id)
    }

    pub(crate) fn insert_create_an_element_for_the_token_result(
        &mut self,
        result: CreateAnElementForTheTokenResult,
    ) -> Result<NodeId, HtmlParseError> {
        // add the element to the arena
        let element_id = self.new_node(XpathItemTreeNode::ElementNode(result.element));

        // add the attributes to the element
        for attribute in result.attributes {
            self.add_attribute_to_element(element_id, attribute.name, attribute.value)?;
        }

        self.open_elements.push(element_id);

        Ok(element_id)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-element-at-the-adjusted-insertion-location>
    pub(crate) fn insert_element_at_adjusted_insertion_location(
        &mut self,
        element_id: NodeId,
    ) -> Result<(), HtmlParseError> {
        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node(None)?;

        adjusted_insertion_location.append(element_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    pub(crate) fn appropriate_place_for_inserting_a_node(
        &self,
        override_target: Option<NodeId>,
    ) -> Result<NodeId, HtmlParseError> {
        let target = if let Some(override_target) = override_target {
            override_target
        } else {
            println!("open elements: {:?}", self.open_elements.len());
            self.open_elements
                .last()
                .cloned()
                .ok_or(HtmlParseError::new("no current node to insert a node into"))?
        };

        let adjusted_insertion_location = if self.foster_parenting {
            let last_template = self.get_last_element_by_tag_name("template");
            let last_table = self.get_last_element_by_tag_name("table");

            // if there is a last template element and either there is no last table element or the last table element is lower in the stack of open elements than the last template element
            // then the adjusted insertion location is inside the last template element's template contents.
            todo!()
        } else {
            target
        };

        Ok(adjusted_insertion_location)
    }

    fn get_last_element_by_tag_name(&self, tag_name: &str) -> Option<(usize, NodeId)> {
        for i in (0..self.open_elements.len()).rev() {
            let node_id = self.open_elements[i];
            if let Some(node) = self.arena.get(node_id) {
                if let XpathItemTreeNode::ElementNode(element) = node.get() {
                    if element.name == tag_name {
                        return Some((i, node_id));
                    }
                }
            }
        }

        None
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token>
    pub(crate) fn create_an_element_for_the_token(
        &mut self,
        token: TagToken,
        namespace: &str,
    ) -> Result<CreateAnElementForTheTokenResult, HtmlParseError> {
        let local_name = token.tag_name;
        let element = self.create_element(local_name, namespace, None, None)?;

        // add the attributes
        let attributes: Vec<AttributeNode> = token
            .attributes
            .into_iter()
            .map(|(name, value)| AttributeNode::new(name, value))
            .collect();

        Ok(CreateAnElementForTheTokenResult {
            element,
            attributes,
        })
    }

    /// <https://dom.spec.whatwg.org/#concept-create-element>
    pub(crate) fn create_element(
        &mut self,
        local_name: String,
        namespace: &str,
        prefix: Option<&str>,
        is: Option<&str>,
    ) -> Result<ElementNode, HtmlParseError> {
        // TODO: namespace?
        let element = ElementNode::new(local_name);

        Ok(element)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements>
    pub(crate) fn reconstruct_the_active_formatting_elements(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        if self.active_formatting_elements.is_empty() {
            return Ok(());
        }

        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character>
    pub(crate) fn insert_character(&mut self, data: Vec<char>) -> Result<(), HtmlParseError> {
        let adjusted_insertion_location_id = self.appropriate_place_for_inserting_a_node(None)?;
        let node = self
            .arena
            .get(adjusted_insertion_location_id)
            .unwrap()
            .get();

        if let XpathItemTreeNode::DocumentNode(_) = node {
            // The DOM will no let Document nodes have Text node children, so they are dropped on the floor.
            return Ok(());
        }

        // the adjusted insertion location in this implementation returns the parent node id
        // where we are expected to insert the new node as the last child of this parent node.
        // this means the previous sibling of the adjusted insertion location is the current last child of the parent node before inserting the new node.
        let prev_sibling_id = self
            .arena
            .get(adjusted_insertion_location_id)
            .unwrap()
            .last_child();

        let prev_sibling: Option<&mut XpathItemTreeNode> =
            prev_sibling_id.map(|id| self.arena.get_mut(id).unwrap().get_mut());

        if let Some(&mut XpathItemTreeNode::TextNode(ref mut text)) = prev_sibling {
            // If the adjusted insertion location's last child is a Text node, append the data to that Text node.
            text.content.extend(data.iter());
        } else {
            // Otherwise, insert a new Text node with the data as its data.
            let string = data.iter().collect::<String>();
            let text = XpathItemTreeNode::TextNode(TextNode::new(string));
            let text_id = self.new_node(text);

            adjusted_insertion_location_id.append(text_id, &mut self.arena);
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-the-specific-scope>
    pub(crate) fn has_an_element_in_the_specific_scope(
        &self,
        tag_name: &str,
        element_types: Vec<&str>,
    ) -> bool {
        for node_id in self.open_elements.iter().rev() {
            if let Some(node) = self.arena.get(*node_id) {
                if let XpathItemTreeNode::ElementNode(element) = node.get() {
                    if element.name == tag_name {
                        return true;
                    }

                    if element_types.contains(&element.name.as_str()) {
                        return false;
                    }
                }
            }
        }

        false
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope>
    pub(crate) fn has_an_element_in_scope(&self, tag_name: &str) -> bool {
        self.has_an_element_in_the_specific_scope(tag_name, ELEMENT_IN_SCOPE_TYPES.to_vec())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope>
    pub(crate) fn has_an_element_in_button_scope(&self, tag_name: &str) -> bool {
        let mut element_types = ELEMENT_IN_SCOPE_TYPES.to_vec();
        element_types.push("button");

        self.has_an_element_in_the_specific_scope(tag_name, element_types)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element>
    pub(crate) fn close_a_p_element(&mut self) -> Result<(), HtmlParseError> {
        self.generate_implied_end_tags(Some("p"))?;

        // If the current node is not a p element, then this is a parse error.
        if let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if element.name != "p" {
                    return self.handle_error(HtmlParserError::MinorError(
                        "closing a p element that is not the current node".to_string(),
                    ));
                }
            }
        }

        // Pop elements until a p element is popped.
        self.pop_until_tag_name("p")?;
        Ok(())
    }

    pub(crate) fn pop_until_tag_name(&mut self, tag_name: &str) -> Result<(), HtmlParseError> {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if element.name == tag_name {
                    self.open_elements.pop();
                    break;
                }
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags>
    pub(crate) fn generate_implied_end_tags(
        &mut self,
        exclude_element: Option<&str>,
    ) -> Result<(), HtmlParseError> {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                // if the element is excluded, then stop
                if let Some(exclude_element) = exclude_element {
                    if element.name == exclude_element {
                        break;
                    }
                }

                // if it is not in the list of implied end tag types, then stop
                if !GENERATE_IMPLIED_END_TAG_TYPES.contains(&element.name.as_str()) {
                    break;
                }
            }

            // otherwise keep popping elements from the stack
            self.open_elements.pop();
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#using-the-rules-for>
    pub(crate) fn using_the_rules_for(
        &mut self,
        token: HtmlToken,
        insertion_mode: InsertionMode,
    ) -> Result<(), HtmlParseError> {
        let before_insertion_mode = self.insertion_mode;
        self.insertion_mode = insertion_mode;
        self.token_emitted(token)?;

        // if the insertion mode was not changed while processing the token, then set it back to the original value
        if self.insertion_mode == insertion_mode {
            self.insertion_mode = before_insertion_mode;
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stop-parsing>
    pub(crate) fn stop_parsing(&mut self) -> Result<(), HtmlParseError> {
        // mostly scripting stuff that is unsupported by Skyscraper
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum HtmlParserError {
    #[error("minor error: {0}")]
    MinorError(String),
    #[error("fatal error: {0}")]
    FatalError(String),
}

impl TokenizerObserver for HtmlParser {
    fn token_emitted(&mut self, token: HtmlToken) -> Result<(), HtmlParseError> {
        match self.insertion_mode {
            InsertionMode::Initial => self.initial_insertion_mode(token),
            InsertionMode::BeforeHtml => self.before_html_insertion_mode(token),
            InsertionMode::BeforeHead => self.before_head_insertion_mode(token),
            InsertionMode::InHead => self.in_head_insertion_mode(token),
            InsertionMode::InHeadNoscript => todo!(),
            InsertionMode::AfterHead => self.after_head_insertion_mode(token),
            InsertionMode::InBody => self.in_body_insertion_mode(token),
            InsertionMode::Text => todo!(),
            InsertionMode::InTable => todo!(),
            InsertionMode::InTableText => todo!(),
            InsertionMode::InCaption => todo!(),
            InsertionMode::InColumnGroup => todo!(),
            InsertionMode::InTableBody => todo!(),
            InsertionMode::InRow => todo!(),
            InsertionMode::InCell => todo!(),
            InsertionMode::InSelect => todo!(),
            InsertionMode::InSelectInTable => todo!(),
            InsertionMode::InTemplate => todo!(),
            InsertionMode::AfterBody => self.after_body_insertion_mode(token),
            InsertionMode::InFrameset => todo!(),
            InsertionMode::AfterFrameset => todo!(),
            InsertionMode::AfterAfterBody => self.after_after_body_insertion_mode(token),
            InsertionMode::AfterAfterFrameset => todo!(),
        }
    }
}

pub trait ParseErrorHandler {
    fn error_emitted(&self, error: HtmlParseErrorType) -> Result<(), HtmlParseError>;
}

pub struct DefaultParseErrorHandler;

impl ParseErrorHandler for DefaultParseErrorHandler {
    fn error_emitted(&self, error: HtmlParseErrorType) -> Result<(), HtmlParseError> {
        Err(HtmlParseError {
            message: format!("{:?}", error),
        })
    }
}

#[cfg(test)]
mod tests {
    use document_builder::DocumentBuilder;

    use crate::html;

    use super::*;

    #[test]
    fn parse_should_return_document() {
        // arrange
        let text = r###"
            <html>
                <body>
                    <div>
                        <p>1</p>
                        <p>2</p>
                        <p>3</p>
                    </div>
                    <div>
                        <p>4</p>
                        <p>5</p>
                        <p>6</p>
                    </div>
                </body>
            </html>"###;

        // act
        let document = parse(&text).unwrap();

        // assert
        let expected = DocumentBuilder::new()
            .with_root("html", |html| {
                html.add_element("head", |head| head)
                    .add_element("body", |body| {
                        body.add_element("div", |div| {
                            div.add_element("p", |p| p.add_text("1"))
                                .add_element("p", |p| p.add_text("2"))
                                .add_element("p", |p| p.add_text("3"))
                        })
                    })
            })
            .build()
            .unwrap();

        assert_eq!(document, expected);
    }
}
