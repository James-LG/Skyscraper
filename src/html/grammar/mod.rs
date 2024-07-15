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
            data_model::{AttributeNode, ElementNode, XpathDocumentNode, XpathItem},
            XpathItemTreeNode,
        },
        Xpath, XpathItemTree,
    },
};

mod chars;
mod insertion_mode_impls;
mod tokenizer;

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

pub struct HtmlParser {
    insertion_mode: InsertionMode,
    open_elements: Vec<NodeId>,
    context_element: Option<XpathItemTreeNode>,
    arena: Arena<XpathItemTreeNode>,
    root_node: Option<NodeId>,
    foster_parenting: bool,
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

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjusted-current-node>
    pub(crate) fn adjusted_current_node(&self) -> Option<&XpathItemTreeNode> {
        if self.context_element.is_some() && self.open_elements.len() == 1 {
            self.context_element.as_ref()
        } else {
            self.current_node()
        }
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
        let item_id = self
            .arena
            .new_node(XpathItemTreeNode::AttributeNode(attribute));

        element_id.append(item_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    pub(crate) fn insert_html_element(
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
        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node(None)?;

        let element_id = self.create_an_element_for_the_token(
            token,
            namespace,
            Some(adjusted_insertion_location),
        )?;

        if !only_add_to_element_stack {
            self.insert_element_at_adjusted_insertion_location(element_id)?;
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
        parent_id: Option<NodeId>,
    ) -> Result<NodeId, HtmlParseError> {
        let local_name = token.tag_name;
        let element_id = self.create_element(local_name, namespace, None, None)?;

        if let Some(parent_id) = parent_id {
            parent_id.append(element_id, &mut self.arena);
        }

        // add the attributes
        for (name, value) in token.attributes {
            self.add_attribute_to_element(element_id, name, value)?;
        }

        Ok(element_id)
    }

    /// <https://dom.spec.whatwg.org/#concept-create-element>
    pub(crate) fn create_element(
        &mut self,
        local_name: String,
        namespace: &str,
        prefix: Option<&str>,
        is: Option<&str>,
    ) -> Result<NodeId, HtmlParseError> {
        // TODO: namespace?
        let element = ElementNode::new(local_name);

        let item_id = self.arena.new_node(XpathItemTreeNode::ElementNode(element));

        self.arena
            .get_mut(item_id)
            .unwrap()
            .get_mut()
            .as_element_node_mut()
            .unwrap()
            .set_id(item_id);

        self.open_elements.push(item_id);

        Ok(item_id)
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
            InsertionMode::InHead => todo!(),
            InsertionMode::InHeadNoscript => todo!(),
            InsertionMode::AfterHead => todo!(),
            InsertionMode::InBody => todo!(),
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
            InsertionMode::AfterBody => todo!(),
            InsertionMode::InFrameset => todo!(),
            InsertionMode::AfterFrameset => todo!(),
            InsertionMode::AfterAfterBody => todo!(),
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
        assert_eq!(document.root().children(&document).len(), 1);
    }
}
