use std::vec;

use crate::{
    html::grammar::{tokenizer::TokenizerState, NodeOrMarker, SPECIAL_ELEMENTS},
    xpath::grammar::{
        data_model::{AttributeNode, DoctypeNode, ElementNode, TextNode},
        XpathItemTreeNode,
    },
};

use super::{
    chars,
    tokenizer::{HtmlToken, Parser, TagToken, TagTokenType},
    Acknowledgement, HtmlParseError, HtmlParser, HtmlParserError, InsertionMode, HTML_NAMESPACE,
};

pub(crate) mod in_body_insertion_mode;

pub use in_body_insertion_mode::*;

impl HtmlParser {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode>
    pub(super) fn initial_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Character(
                c @ (chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE),
            ) => {
                // WHATWG says ignore, but we preserve for round-trip fidelity
                self.insert_character_at_document_level(c)?;
            }
            HtmlToken::Comment(comment) => {
                // Insert a comment as the last child of the Document object.
                let parent = self
                    .root_node
                    .ok_or(HtmlParseError::new("root node is None"))?;

                self.insert_a_comment(comment, Some(parent))?;
            }
            HtmlToken::DocType(doctype) => {
                // Append a DocumentType node to the Document node.
                let doctype_node = XpathItemTreeNode::DoctypeNode(DoctypeNode::new(
                    doctype.name,
                    doctype.public_identifier,
                    doctype.system_identifier,
                ));

                let doctype_id = self.arena.new_node(doctype_node);

                self.root_node
                    .ok_or(HtmlParseError::new("root node is None"))?
                    .append(doctype_id, &mut self.arena);

                self.insertion_mode = InsertionMode::BeforeHtml;
            }
            _ => {
                // TODO: If the document is not an iframe srcdoc document, then this is a parse error;
                //       if the parser cannot change the mode flag is false, set the Document to quirks mode.

                self.insertion_mode = InsertionMode::BeforeHtml;
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode>
    pub(super) fn before_html_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            let result = parser.create_element(String::from("html"), HTML_NAMESPACE, None, None)?;

            // append the node to the document
            let node_id = parser.new_node(XpathItemTreeNode::ElementNode(result));
            parser
                .root_node
                .expect("root node is None")
                .append(node_id, &mut parser.arena);

            parser.open_elements.push(node_id);

            parser.insertion_mode = InsertionMode::BeforeHead;
            parser.token_emitted(token)?;

            Ok(())
        }

        match token {
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::Comment(token) => {
                let parent = self
                    .root_node
                    .ok_or(HtmlParseError::new("root node is None"))?;

                self.insert_a_comment(token, Some(parent))?;
            }
            HtmlToken::Character(
                c @ (chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE),
            ) => {
                // WHATWG says ignore, but we preserve for round-trip fidelity
                self.insert_character_at_document_level(c)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                let result = self.create_an_element_for_the_token(token, HTML_NAMESPACE)?;

                // insert the result
                let node_id = self.insert_create_an_element_for_the_token_result(result)?;

                // append it to the document
                self.root_node
                    .expect("root node is None")
                    .append(node_id, &mut self.arena);

                self.insertion_mode = InsertionMode::BeforeHead;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(TagToken { tag_name, .. }))
                if ["head", "body", "html", "br"].contains(&tag_name.as_ref()) =>
            {
                anything_else(
                    self,
                    HtmlToken::TagToken(TagTokenType::EndTag(TagToken::new(tag_name))),
                )?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                todo!()
            }
            _ => {
                anything_else(self, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode>
    pub(super) fn before_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            let node_id = parser.insert_an_html_element(TagToken::new(String::from("head")))?;

            parser.head_element_pointer = Some(node_id);

            parser.insertion_mode = InsertionMode::InHead;
            parser.token_emitted(token)?;

            Ok(())
        }

        match token {
            HtmlToken::Character(
                c @ (chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE),
            ) => {
                // WHATWG says ignore, but we preserve for round-trip fidelity
                self.insert_character(vec![c])?;
            }
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "head" => {
                let node_id = self.insert_an_html_element(token)?;

                self.head_element_pointer = Some(node_id);

                self.insertion_mode = InsertionMode::InHead;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["head", "body", "html", "br"].contains(&token.tag_name.as_ref()) =>
            {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                todo!()
            }
            _ => anything_else(self, token)?,
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead>
    pub(super) fn in_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            parser.open_elements.pop().expect("open elements is empty");

            parser.insertion_mode = InsertionMode::AfterHead;

            parser.token_emitted(token)?;

            Ok(())
        }
        match token {
            HtmlToken::Character(c)
                if [
                    chars::CHARACTER_TABULATION,
                    chars::LINE_FEED,
                    chars::FORM_FEED,
                    chars::CARRIAGE_RETURN,
                    chars::SPACE,
                ]
                .contains(&c) =>
            {
                self.insert_character(vec![c])?;
            }
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["base", "basefont", "bgsound", "link"].contains(&token.tag_name.as_str()) =>
            {
                self.insert_an_html_element(token)?;

                self.open_elements.pop().expect("open elements is empty");

                // acknowledge the self closing tag
                return Ok(Acknowledgement::yes());
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "meta" => {
                self.insert_an_html_element(token)?;

                self.open_elements.pop().expect("open elements is empty");

                // TODO: some encoding stuff

                // acknowledge the self closing tag
                return Ok(Acknowledgement::yes());
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "title" => {
                return self.generic_rcdata_element_parsing_algorithm(token);
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["noframes", "style"].contains(&token.tag_name.as_str()) =>
            {
                return self.generic_raw_text_element_parsing_algorithm(token);
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "noscript" => {
                // Scripting is disabled in Skyscraper, so:
                // Insert an HTML element for the token.
                self.insert_an_html_element(token)?;
                // Switch the insertion mode to "in head noscript".
                self.insertion_mode = InsertionMode::InHeadNoscript;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "script" => {
                let node = self.insert_an_html_element(token)?;

                // TODO: lots of script and template stuff

                self.original_insertion_mode = Some(self.insertion_mode);
                self.insertion_mode = InsertionMode::Text;

                // set tokenizer state to script data state
                return Ok(Acknowledgement {
                    self_closed: false,
                    tokenizer_state: Some(TokenizerState::ScriptData),
                });
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "head" => {
                self.open_elements.pop().expect("open elements is empty");

                self.insertion_mode = InsertionMode::AfterHead;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["body", "html", "br"].contains(&token.tag_name.as_str()) =>
            {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "template" => {
                self.active_formatting_elements.push(NodeOrMarker::Marker);
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::InTemplate;
                self.template_insertion_modes
                    .push(InsertionMode::InTemplate);

                // TODO: shadow root mode
                if self.adjusted_current_node_id().ok() == self.open_elements.last().map(|x| *x) {
                    self.insert_an_html_element(token)?;
                    return Ok(Acknowledgement::no());
                }

                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                if !self.open_elements_has_element("template") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "unexpected template end tag",
                    )))?;
                    return Ok(Acknowledgement::no());
                }

                self.generate_all_implied_end_tags_thoroughly()?;

                let current_node = self.current_node_as_element_result()?;
                if current_node.name != "template" {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "template end tag not found",
                    )))?;
                }

                self.pop_until_tag_name("template")?;
                self.clear_the_list_of_active_formatting_elements_up_to_the_last_marker()?;
                self.template_insertion_modes.pop();
                self.reset_the_insertion_mode_appropriately()?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                // Any other end tag: parse error, ignore the token.
            }
            _ => {
                anything_else(self, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inheadnoscript>
    pub(super) fn in_head_noscript_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            // Parse error.
            // Pop the current node (which will be a noscript element) from the stack of open
            // elements; the new current node will be a head element.
            parser.open_elements.pop().expect("open elements is empty");
            // Switch the insertion mode to "in head".
            parser.insertion_mode = InsertionMode::InHead;
            // Reprocess the token.
            parser.token_emitted(token)?;
            Ok(())
        }

        match token {
            // A DOCTYPE token: Parse error. Ignore the token.
            HtmlToken::DocType(_) => {
                // parse error, ignore
            }
            // A comment token: Process the token using the rules for the "in head" insertion mode.
            HtmlToken::Comment(comment) => {
                return self.in_head_insertion_mode(HtmlToken::Comment(comment));
            }
            // A character token that is one of U+0009, U+000A, U+000C, U+000D, or U+0020:
            // Process the token using the rules for the "in head" insertion mode.
            HtmlToken::Character(c)
                if [
                    chars::CHARACTER_TABULATION,
                    chars::LINE_FEED,
                    chars::FORM_FEED,
                    chars::CARRIAGE_RETURN,
                    chars::SPACE,
                ]
                .contains(&c) =>
            {
                return self.in_head_insertion_mode(HtmlToken::Character(c));
            }
            // A start tag whose tag name is "html":
            // Process the token using the rules for the "in body" insertion mode.
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                return self
                    .in_body_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // An end tag whose tag name is "noscript":
            // Pop the current node (noscript) from the stack of open elements;
            // the new current node will be a head element.
            // Switch the insertion mode to "in head".
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "noscript" => {
                self.open_elements.pop().expect("open elements is empty");
                self.insertion_mode = InsertionMode::InHead;
            }
            // A start tag whose tag name is one of: "basefont", "bgsound", "link", "meta",
            // "noframes", "style":
            // Process the token using the rules for the "in head" insertion mode.
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["basefont", "bgsound", "link", "meta", "noframes", "style"]
                    .contains(&token.tag_name.as_str()) =>
            {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // An end tag whose tag name is "br":
            // Act as described in the "anything else" entry below.
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "br" => {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            // A start tag whose tag name is one of: "head", "noscript":
            // Parse error. Ignore the token.
            HtmlToken::TagToken(TagTokenType::StartTag(_token))
                if ["head", "noscript"].contains(&_token.tag_name.as_str()) =>
            {
                // parse error, ignore
            }
            // Any other end tag: Parse error. Ignore the token.
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                // parse error, ignore
            }
            // Anything else: Parse error.
            // Pop the current node (noscript), switch to "in head", reprocess.
            _ => {
                anything_else(self, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode>
    pub(super) fn after_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            parser.insert_an_html_element(TagToken::new(String::from("body")))?;

            parser.insertion_mode = InsertionMode::InBody;

            parser.token_emitted(token)?;

            Ok(())
        }
        match token {
            HtmlToken::Character(c)
                if [
                    chars::CHARACTER_TABULATION,
                    chars::LINE_FEED,
                    chars::FORM_FEED,
                    chars::CARRIAGE_RETURN,
                    chars::SPACE,
                ]
                .contains(&c) =>
            {
                self.insert_character(vec![c])?;
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "body" => {
                self.insert_an_html_element(token)?;

                self.frameset_ok = false;

                self.insertion_mode = InsertionMode::InBody;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "frameset" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["body", "html", "br"].contains(&token.tag_name.as_str()) =>
            {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                todo!()
            }
            _ => {
                anything_else(self, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata>
    pub(super) fn text_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Character(c) => {
                self.insert_character(vec![c])?;
            }
            HtmlToken::EndOfFile => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "script" => {
                let script = self.current_node_as_element_result()?;

                self.open_elements.pop().expect("open elements is empty");

                self.insertion_mode = self
                    .original_insertion_mode
                    .expect("original insertion mode is None");

                // lots of unsupported scripting logic would go here
                // it is intentionally not included
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_token)) => {
                self.open_elements.pop().expect("open elements is empty");

                self.insertion_mode = self.original_insertion_mode.unwrap();
            }
            _ => {
                // ignore
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intemplate>
    pub(super) fn in_template_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Character(_) | HtmlToken::Comment(_) | HtmlToken::DocType(_) => {
                self.using_the_rules_for(token, InsertionMode::InBody)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InHead,
                )?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::EndTag(token)),
                    InsertionMode::InHead,
                )?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["caption", "colgroup", "tbody", "tfoot", "thead"]
                    .contains(&token.tag_name.as_str()) =>
            {
                self.template_insertion_modes.pop();
                self.template_insertion_modes.push(InsertionMode::InTable);
                self.insertion_mode = InsertionMode::InTable;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["col"].contains(&token.tag_name.as_str()) =>
            {
                self.template_insertion_modes.pop();
                self.template_insertion_modes
                    .push(InsertionMode::InColumnGroup);
                self.insertion_mode = InsertionMode::InColumnGroup;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["tr"].contains(&token.tag_name.as_str()) =>
            {
                self.template_insertion_modes.pop();
                self.template_insertion_modes
                    .push(InsertionMode::InTableBody);
                self.insertion_mode = InsertionMode::InTableBody;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["td", "th"].contains(&token.tag_name.as_str()) =>
            {
                self.template_insertion_modes.pop();
                self.template_insertion_modes.push(InsertionMode::InRow);
                self.insertion_mode = InsertionMode::InRow;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) => {
                self.template_insertion_modes.pop();
                self.template_insertion_modes.push(InsertionMode::InBody);
                self.insertion_mode = InsertionMode::InBody;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected end tag",
                )))?;
            }
            HtmlToken::EndOfFile => {
                if self.open_elements_has_element("template") {
                    self.stop_parsing()?;
                    return Ok(Acknowledgement::no());
                }

                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected end of file",
                )))?;
                self.pop_until_tag_name("template")?;
                self.clear_the_list_of_active_formatting_elements_up_to_the_last_marker()?;
                self.template_insertion_modes.pop();
                self.reset_the_insertion_mode_appropriately()?;
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }
    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody>
    pub(super) fn after_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Character(c)
                if [
                    chars::CHARACTER_TABULATION,
                    chars::LINE_FEED,
                    chars::FORM_FEED,
                    chars::CARRIAGE_RETURN,
                    chars::SPACE,
                ]
                .contains(&c) =>
            {
                // WHATWG says use InBody rules, but we insert as a child of
                // the html element (after body) for round-trip fidelity.
                let html_node = *self.open_elements.first().ok_or(
                    HtmlParseError::new("no elements on open elements stack"),
                )?;
                self.insert_character_at_node(html_node, c)?;
            }
            HtmlToken::Comment(_) => {
                todo!()
            }
            HtmlToken::DocType(_) => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "html" => {
                // TODO: If parser was created as part of the HTML fragment parsing algorithm...

                self.insertion_mode = InsertionMode::AfterAfterBody;
            }
            HtmlToken::EndOfFile => {
                self.stop_parsing()?;
            }
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token after body",
                )))?;

                self.insertion_mode = InsertionMode::InBody;
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-body-insertion-mode>
    pub(super) fn after_after_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Comment(_) => {
                todo!()
            }
            HtmlToken::DocType(_) => {
                self.using_the_rules_for(token, InsertionMode::InBody)?;
            }
            HtmlToken::Character(c)
                if [
                    chars::CHARACTER_TABULATION,
                    chars::LINE_FEED,
                    chars::FORM_FEED,
                    chars::CARRIAGE_RETURN,
                    chars::SPACE,
                ]
                .contains(&c) =>
            {
                // WHATWG says use InBody rules, but we preserve at document
                // level for round-trip fidelity (whitespace after </html>).
                self.insert_character_at_document_level(c)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
            }
            HtmlToken::EndOfFile => {
                self.stop_parsing()?;
            }
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token after after body",
                )))?;

                self.insertion_mode = InsertionMode::InBody;
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }
}
