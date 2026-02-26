use indextree::NodeId;

use crate::{
    html::grammar::{tokenizer::TokenizerState, NodeEntry, NodeOrMarker, MATHML_NAMESPACE, SPECIAL_ELEMENTS, SVG_NAMESPACE},
    xpath::grammar::{
        data_model::{AttributeNode, ElementNode},
        XpathItemTreeNode,
    },
};

use super::{
    super::tokenizer::{HtmlToken, Parser, TagToken, TagTokenType},
    chars, Acknowledgement, HtmlParseError, HtmlParser, HtmlParserError, InsertionMode,
};

impl HtmlParser {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody>
    pub(crate) fn in_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn ensure_open_elements_has_valid_element(
            parser: &HtmlParser,
        ) -> Result<(), HtmlParseError> {
            let valid_elements = vec![
                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc", "tbody",
                "td", "tfoot", "th", "thead", "tr", "body", "html",
            ];

            if !parser
                .open_elements
                .iter()
                .map(|node_id| parser.arena.get(*node_id).unwrap().get())
                .filter_map(|node| node.as_element_node().ok())
                .any(|node| valid_elements.contains(&node.name.as_str()))
            {
                return parser.handle_error(HtmlParserError::MinorError(String::from(
                    "open elements has no valid element",
                )));
            }

            Ok(())
        }

        match token {
            HtmlToken::Character(chars::NULL) => {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "null character in body",
                )))?;
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
                self.reconstruct_the_active_formatting_elements()?;

                self.insert_character(vec![c])?;
            }
            HtmlToken::Character(c) => {
                self.reconstruct_the_active_formatting_elements()?;

                self.insert_character(vec![c])?;

                self.frameset_ok = false;
            }
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected DOCTYPE in body",
                )))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "html start tag inside body",
                )))?;

                // if there is a template element on the stack, ignore the token
                let elements: Vec<&ElementNode> = self
                    .open_elements_as_nodes()
                    .iter()
                    .filter_map(|node| node.as_element_node().ok())
                    .collect();

                if elements.iter().any(|node| node.name == "template") {
                    return Ok(Acknowledgement::no());
                }

                // for each attribute, check if the attribute is already present on top element of the stack
                let top_element_res = self.top_node().unwrap().as_element_node();

                let top_element = match top_element_res {
                    Ok(node) => node,
                    Err(_) => {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "top element is not an element node",
                        )))?;
                        return Ok(Acknowledgement::no());
                    }
                };

                let top_element_attrs = top_element
                    .attributes_arena(&self.arena)
                    .into_iter()
                    .map(|attr| attr.name.to_string())
                    .collect::<Vec<String>>();

                for attribute in token.attributes.into_iter() {
                    // if the element doesn't already have the attribute, add it
                    if !top_element_attrs.contains(&attribute.name) {
                        let top_node_id = *self.open_elements.first().unwrap();

                        let attr_node_id = self.new_node(XpathItemTreeNode::AttributeNode(
                            AttributeNode::with_prefix(attribute.name, attribute.value, attribute.prefix, attribute.original_name),
                        ));
                        top_node_id.append(attr_node_id, &mut self.arena);
                    }
                }
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
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "body" => {
                if !self.has_an_element_in_scope("body") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no body element in scope",
                    )))?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                }

                self.insertion_mode = InsertionMode::AfterBody;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "frameset" => {
                // Parse error.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "frameset start tag in body",
                )))?;

                // If the stack of open elements has only one node on it, or if the second
                // element on the stack of open elements is not a body element, ignore the token.
                if self.open_elements.len() == 1 {
                    return Ok(Acknowledgement::no());
                }

                let second_element_id = self.open_elements[1];
                let is_body = self
                    .arena
                    .get(second_element_id)
                    .and_then(|node| node.get().as_element_node().ok())
                    .map_or(false, |el| el.name == "body");

                if !is_body {
                    return Ok(Acknowledgement::no());
                }

                // If the frameset-ok flag is set to "not ok", ignore the token.
                if !self.frameset_ok {
                    return Ok(Acknowledgement::no());
                }

                // Remove the second element on the stack of open elements from its parent node.
                second_element_id.detach(&mut self.arena);

                // Pop all the nodes from the bottom of the stack of open elements, from the
                // current node up to, but not including, the root html element.
                while self.open_elements.len() > 1 {
                    self.open_elements.pop();
                }

                // Insert an HTML element for the token.
                self.insert_an_html_element(token)?;

                // Switch the insertion mode to "in frameset".
                self.insertion_mode = InsertionMode::InFrameset;
            }
            HtmlToken::EndOfFile => {
                if !self.template_insertion_modes.is_empty() {
                    self.using_the_rules_for(token, InsertionMode::InTemplate)?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                    self.stop_parsing()?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "body" => {
                if !self.has_an_element_in_scope("body") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has body element in scope",
                    )))?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                }

                self.insertion_mode = InsertionMode::AfterBody;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "html" => {
                if !self.has_an_element_in_scope("body") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has body element in scope",
                    )))?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                }

                self.insertion_mode = InsertionMode::AfterBody;

                self.token_emitted(HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "address",
                    "article",
                    "aside",
                    "blockquote",
                    "center",
                    "details",
                    "dialog",
                    "dir",
                    "div",
                    "dl",
                    "fieldset",
                    "figcaption",
                    "figure",
                    "footer",
                    "header",
                    "hgroup",
                    "main",
                    "menu",
                    "nav",
                    "ol",
                    "p",
                    "search",
                    "section",
                    "summary",
                    "ul",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&token.tag_name.as_str()) =>
            {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                if let Some(element) = self.current_node_as_element() {
                    if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&element.name.as_str()) {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is h1, h2, h3, h4, h5, or h6",
                        )))?;
                        self.open_elements.pop();
                    }
                }

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["pre", "listing"].contains(&token.tag_name.as_str()) =>
            {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                self.insert_an_html_element(token)?;

                // If the next token is a U+000A LINE FEED character token, ignore it.
                self.skip_next_line_feed = true;

                self.frameset_ok = false;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "form" => {
                if self.form_element_pointer.is_some()
                    && !self.open_elements_has_element("template")
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "form element pointer is not none and there is no template element",
                    )))?;
                } else {
                    if self.has_an_element_in_button_scope("p") {
                        self.close_a_p_element()?;
                    }

                    let element_id = self.insert_an_html_element(token)?;

                    if !self.open_elements_has_element("template") {
                        self.form_element_pointer = Some(element_id);
                    }
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "li" => {
                fn step_3_loop(
                    parser: &mut HtmlParser,
                    element: &ElementNode,
                    token: TagToken,
                ) -> Result<(), HtmlParseError> {
                    if element.name == "li" {
                        parser.generate_implied_end_tags(Some("li"))?;

                        if parser.current_node_as_element_result()?.name != "li" {
                            parser.handle_error(HtmlParserError::MinorError(String::from(
                                "current node is not li",
                            )))?;
                        }

                        parser.pop_until_tag_name("li")?;
                    }

                    if SPECIAL_ELEMENTS.contains(&element.name.as_str())
                        && !["address", "div", "p"].contains(&element.name.as_str())
                    {
                        step_6_done(parser, token)?;
                    } else {
                        let current_element_index = parser
                            .open_elements
                            .iter()
                            .position(|node_id| node_id == &element.id())
                            .expect("current element is not in open elements");

                        let previous_element_id = parser
                            .open_elements
                            .get(current_element_index - 1)
                            .expect("previous element is not in open elements");

                        let previous_element = parser
                            .arena
                            .get(*previous_element_id)
                            .unwrap()
                            .get()
                            .as_element_node()
                            .map_err(|_| {
                                HtmlParserError::MinorError(String::from(
                                    "previous element is not an element node",
                                ))
                            });

                        match previous_element {
                            Err(_) => {
                                parser.handle_error(HtmlParserError::MinorError(String::from(
                                    "previous element is not an element node",
                                )))?;
                            }
                            Ok(previous_element) => {
                                let previous_element = previous_element.clone();
                                return step_3_loop(parser, &previous_element, token);
                            }
                        }
                    }

                    Ok(())
                }

                fn step_6_done(
                    parser: &mut HtmlParser,
                    token: TagToken,
                ) -> Result<(), HtmlParseError> {
                    if parser.has_an_element_in_button_scope("p") {
                        parser.close_a_p_element()?;
                    }

                    parser.insert_an_html_element(token)?;

                    Ok(())
                }

                self.frameset_ok = false;

                let node = self.current_node_as_element_result()?.clone();
                step_3_loop(self, &node, token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["dd", "dt"].contains(&token.tag_name.as_str()) =>
            {
                fn step_3_loop(
                    parser: &mut HtmlParser,
                    element: &ElementNode,
                    token: TagToken,
                ) -> Result<(), HtmlParseError> {
                    // If node is a dd element, then run these substeps:
                    if element.name == "dd" {
                        parser.generate_implied_end_tags(Some("dd"))?;

                        if parser.current_node_as_element_result()?.name != "dd" {
                            parser.handle_error(HtmlParserError::MinorError(String::from(
                                "current node is not dd",
                            )))?;
                        }

                        parser.pop_until_tag_name("dd")?;
                    }

                    // If node is a dt element, then run these substeps:
                    if element.name == "dt" {
                        parser.generate_implied_end_tags(Some("dt"))?;

                        if parser.current_node_as_element_result()?.name != "dt" {
                            parser.handle_error(HtmlParserError::MinorError(String::from(
                                "current node is not dt",
                            )))?;
                        }

                        parser.pop_until_tag_name("dt")?;
                    }

                    // If node is in the special category, but is not an address, div, or p
                    // element, then jump to the step labeled done below.
                    if SPECIAL_ELEMENTS.contains(&element.name.as_str())
                        && !["address", "div", "p"].contains(&element.name.as_str())
                    {
                        step_done(parser, token)?;
                    } else {
                        // Otherwise, set node to the previous entry in the stack of open
                        // elements and return to the step labeled loop.
                        let current_element_index = parser
                            .open_elements
                            .iter()
                            .position(|node_id| node_id == &element.id())
                            .expect("current element is not in open elements");

                        let previous_element_id = parser
                            .open_elements
                            .get(current_element_index - 1)
                            .expect("previous element is not in open elements");

                        let previous_element = parser
                            .arena
                            .get(*previous_element_id)
                            .unwrap()
                            .get()
                            .as_element_node()
                            .map_err(|_| {
                                HtmlParserError::MinorError(String::from(
                                    "previous element is not an element node",
                                ))
                            });

                        match previous_element {
                            Err(_) => {
                                parser.handle_error(HtmlParserError::MinorError(String::from(
                                    "previous element is not an element node",
                                )))?;
                            }
                            Ok(previous_element) => {
                                let previous_element = previous_element.clone();
                                return step_3_loop(parser, &previous_element, token);
                            }
                        }
                    }

                    Ok(())
                }

                fn step_done(
                    parser: &mut HtmlParser,
                    token: TagToken,
                ) -> Result<(), HtmlParseError> {
                    if parser.has_an_element_in_button_scope("p") {
                        parser.close_a_p_element()?;
                    }

                    parser.insert_an_html_element(token)?;

                    Ok(())
                }

                self.frameset_ok = false;

                let node = self.current_node_as_element_result()?.clone();
                step_3_loop(self, &node, token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "plaintext" => {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                self.insert_an_html_element(token)?;

                // Switch the tokenizer to the PLAINTEXT state.
                return Ok(Acknowledgement {
                    self_closed: false,
                    tokenizer_state: Some(TokenizerState::PLAINTEXT),
                });
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "button" => {
                if self.has_an_element_in_scope("button") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has button element in scope",
                    )))?;

                    self.generate_implied_end_tags(None)?;

                    self.pop_until_tag_name("button")?;
                }

                self.reconstruct_the_active_formatting_elements()?;
                self.insert_an_html_element(token)?;
                self.frameset_ok = false;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if [
                    "address",
                    "article",
                    "aside",
                    "blockquote",
                    "button",
                    "center",
                    "details",
                    "dialog",
                    "dir",
                    "div",
                    "dl",
                    "fieldset",
                    "figcaption",
                    "figure",
                    "footer",
                    "header",
                    "hgroup",
                    "listing",
                    "main",
                    "menu",
                    "nav",
                    "ol",
                    "pre",
                    "section",
                    "summary",
                    "ul",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_scope(&token.tag_name) {
                    self.handle_error(HtmlParserError::MinorError(String::from(format!(
                        "open elements has {} element in scope",
                        token.tag_name
                    ))))?;
                } else {
                    self.generate_implied_end_tags(None)?;

                    if self.current_node_as_element().unwrap().name != token.tag_name {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is not the same as the token tag name",
                        )))?;
                    }

                    self.pop_until_tag_name(&token.tag_name)?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "form" => {
                if !self.open_elements_has_element("template") {
                    let node = self.form_element_pointer;
                    self.form_element_pointer = None;

                    match node {
                        Some(node) => {
                            let element = self
                                .arena
                                .get(node)
                                .expect("form element pointer is none")
                                .get()
                                .as_element_node()
                                .expect("form element pointer is not an element node")
                                .clone();

                            if !self.has_an_element_in_scope(&element.name) {
                                self.handle_error(HtmlParserError::MinorError(String::from(
                                    "open elements has no form element in scope",
                                )))?;
                            } else {
                                self.generate_implied_end_tags(None)?;

                                if self.current_node_id_result()? != node {
                                    self.handle_error(HtmlParserError::MinorError(String::from(
                                        "current node is not the same as the form element",
                                    )))?;
                                }

                                // remove node from open elements
                                self.open_elements.retain(|node_id| node_id != &node);
                            }
                        }
                        None => {
                            self.handle_error(HtmlParserError::MinorError(String::from(
                                "form element pointer is none",
                            )))?;

                            return Ok(Acknowledgement::no());
                        }
                    }
                } else {
                    if !self.has_an_element_in_scope("form") {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "open elements has no form element in scope",
                        )))?;
                        return Ok(Acknowledgement::no());
                    } else {
                        self.generate_implied_end_tags(None)?;

                        if self.current_node_as_element_result()?.name != "form" {
                            self.handle_error(HtmlParserError::MinorError(String::from(
                                "current node is not form",
                            )))?;
                        }

                        self.pop_until_tag_name("form")?;
                    }
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "p" => {
                if !self.has_an_element_in_button_scope("p") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no p element in button scope",
                    )))?;

                    self.insert_an_html_element(TagToken::new(String::from("p")))?;
                }

                self.close_a_p_element()?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "li" => {
                if !self.has_an_element_in_list_item_scope("li") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no li element in list item scope",
                    )))?;
                } else {
                    self.generate_implied_end_tags(Some("li"))?;

                    if self.current_node_as_element().unwrap().name != "li" {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is not li",
                        )))?;
                    }

                    self.pop_until_tag_name("li")?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["dd", "dt"].contains(&token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_scope(&token.tag_name) {
                    // Parse error. Ignore the token.
                    self.handle_error(HtmlParserError::MinorError(format!(
                        "no {} element in scope",
                        token.tag_name
                    )))?;
                } else {
                    self.generate_implied_end_tags(Some(&token.tag_name))?;

                    if self.current_node_as_element().unwrap().name != token.tag_name {
                        self.handle_error(HtmlParserError::MinorError(format!(
                            "current node is not {}",
                            token.tag_name
                        )))?;
                    }

                    self.pop_until_tag_name(&token.tag_name)?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&token.tag_name.as_str()) =>
            {
                if !self
                    .has_an_element_in_scope_by_tag_names(vec!["h1", "h2", "h3", "h4", "h5", "h6"])
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no h1, h2, h3, h4, h5, or h6 element in scope",
                    )))?;

                    return Ok(Acknowledgement::no());
                }

                self.generate_implied_end_tags(None)?;
                if self.current_node_as_element_result()?.name != token.tag_name {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "current node is not the same as the token tag name",
                    )))?;
                }

                self.pop_until_tag_name_one_of(vec!["h1", "h2", "h3", "h4", "h5", "h6"])?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "sarcasm" => {
                // "Take a deep breath, then act as described in the 'any other end tag' entry below."
                self.any_other_end_tag(&token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "a" => {
                // Check if there's already an <a> in active formatting elements
                let old_a_node_id = self
                    .active_formatting_elements
                    .iter()
                    .rev()
                    .take_while(|e| !matches!(e, NodeOrMarker::Marker))
                    .find_map(|e| {
                        if let NodeOrMarker::Node(entry) = e {
                            if let Ok(element) =
                                self.arena.get(entry.node_id).unwrap().get().as_element_node()
                            {
                                if element.name == "a" {
                                    return Some(entry.node_id);
                                }
                            }
                        }
                        None
                    });
                if let Some(old_a_id) = old_a_node_id {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "active formatting elements contains an a element",
                    )))?;
                    self.adoption_agency_algorithm(&token)?;
                    self.remove_from_active_formatting_elements(old_a_id)?;
                    self.open_elements.retain(|id| *id != old_a_id);
                }
                self.reconstruct_the_active_formatting_elements()?;
                let element_id = self.insert_an_html_element(token.clone())?;
                self.push_onto_the_list_of_active_formatting_elements(element_id, token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "b", "big", "code", "em", "font", "i", "s", "small", "strike", "strong",
                    "tt", "u",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                self.reconstruct_the_active_formatting_elements()?;
                let element_id = self.insert_an_html_element(token.clone())?;
                self.push_onto_the_list_of_active_formatting_elements(element_id, token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "nobr" => {
                self.reconstruct_the_active_formatting_elements()?;

                if self.has_an_element_in_scope("nobr") {
                    // Parse error.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "nobr element already in scope",
                    )))?;
                    self.adoption_agency_algorithm(&token)?;
                    self.reconstruct_the_active_formatting_elements()?;
                }

                let element_id = self.insert_an_html_element(token.clone())?;
                self.push_onto_the_list_of_active_formatting_elements(element_id, token)?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if [
                    "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                    "strong", "tt", "u",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                self.adoption_agency_algorithm(&token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["applet", "marquee", "object"].contains(&token.tag_name.as_str()) =>
            {
                self.reconstruct_the_active_formatting_elements()?;
                self.insert_an_html_element(token)?;
                // Insert a marker at the end of the list of active formatting elements.
                self.active_formatting_elements.push(NodeOrMarker::Marker);
                self.frameset_ok = false;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["applet", "marquee", "object"].contains(&token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_scope(&token.tag_name) {
                    // Parse error. Ignore the token.
                    self.handle_error(HtmlParserError::MinorError(format!(
                        "no {} element in scope",
                        token.tag_name
                    )))?;
                } else {
                    self.generate_implied_end_tags(None)?;

                    if self.current_node_as_element().unwrap().name != token.tag_name {
                        self.handle_error(HtmlParserError::MinorError(format!(
                            "current node is not {}",
                            token.tag_name
                        )))?;
                    }

                    self.pop_until_tag_name(&token.tag_name)?;
                    self.clear_the_list_of_active_formatting_elements_up_to_the_last_marker()?;
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "table" => {
                // TODO: If the Document is not set to quirks mode, and ...
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                self.insert_an_html_element(token)?;
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::InTable;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "br" => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "br end tag in body",
                )))?;

                // drop attributes
                let token = TagToken {
                    tag_name: String::from("br"),
                    attributes: vec![],
                    self_closing: token.self_closing,
                };

                // act as if it was a start tag
                self.reconstruct_the_active_formatting_elements()?;

                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                self.frameset_ok = false;

                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["area", "br", "embed", "img", "keygen", "wbr"]
                    .contains(&token.tag_name.as_str()) =>
            {
                self.reconstruct_the_active_formatting_elements()?;

                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                self.frameset_ok = false;

                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "input" => {
                self.reconstruct_the_active_formatting_elements()?;

                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                self.frameset_ok = false;
                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["param", "source", "track"].contains(&token.tag_name.as_str()) =>
            {
                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "hr" => {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                self.frameset_ok = false;

                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(mut token)) if token.tag_name == "image" => {
                // Parse error. Change the token's tag name to "img" and reprocess.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "image start tag, rewriting to img",
                )))?;
                token.tag_name = String::from("img");
                return self.in_body_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "textarea" => {
                self.insert_an_html_element(token)?;

                // If the next token is a U+000A LINE FEED character token, ignore it.
                self.skip_next_line_feed = true;

                self.original_insertion_mode = Some(self.insertion_mode);
                self.insertion_mode = InsertionMode::Text;
                self.frameset_ok = false;

                return Ok(Acknowledgement {
                    self_closed: false,
                    tokenizer_state: Some(TokenizerState::RCDATA),
                });
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "xmp" => {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                self.reconstruct_the_active_formatting_elements()?;
                self.frameset_ok = false;

                return self.generic_raw_text_element_parsing_algorithm(token);
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "iframe" => {
                self.frameset_ok = false;

                return self.generic_raw_text_element_parsing_algorithm(token);
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "noembed" => {
                // Follow the generic raw text element parsing algorithm.
                return self.generic_raw_text_element_parsing_algorithm(token);
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "noscript" => {
                // Scripting is disabled in Skyscraper, so treat as a normal element:
                // reconstruct active formatting elements and insert.
                self.reconstruct_the_active_formatting_elements()?;
                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "select" => {
                // Reconstruct the active formatting elements, if any.
                self.reconstruct_the_active_formatting_elements()?;
                // Insert an HTML element for the token.
                self.insert_an_html_element(token)?;
                // Set the frameset-ok flag to "not ok".
                self.frameset_ok = false;
                // If the insertion mode is one of "in table", "in caption", "in table body",
                // "in row", or "in cell", then switch the insertion mode to "in select in table".
                // Otherwise, switch the insertion mode to "in select".
                if matches!(
                    self.insertion_mode,
                    InsertionMode::InTable
                        | InsertionMode::InCaption
                        | InsertionMode::InTableBody
                        | InsertionMode::InRow
                        | InsertionMode::InCell
                ) {
                    self.insertion_mode = InsertionMode::InSelectInTable;
                } else {
                    self.insertion_mode = InsertionMode::InSelect;
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["optgroup", "option"].contains(&token.tag_name.as_str()) =>
            {
                // If the current node is an option element, then pop that node from the stack
                // of open elements.
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "option" {
                        self.open_elements.pop();
                    }
                }
                // Reconstruct the active formatting elements, if any.
                self.reconstruct_the_active_formatting_elements()?;
                // Insert an HTML element for the token.
                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["rb", "rtc"].contains(&token.tag_name.as_str()) =>
            {
                // If the stack of open elements has a ruby element in scope,
                // then generate implied end tags.
                if self.has_an_element_in_scope("ruby") {
                    self.generate_implied_end_tags(None)?;

                    // If the current node is not now a ruby element, this is a parse error.
                    if self.current_node_as_element().unwrap().name != "ruby" {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is not a ruby element",
                        )))?;
                    }
                }

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["rp", "rt"].contains(&token.tag_name.as_str()) =>
            {
                // If the stack of open elements has a ruby element in scope,
                // then generate implied end tags, except for rtc elements.
                if self.has_an_element_in_scope("ruby") {
                    self.generate_implied_end_tags(Some("rtc"))?;

                    // If the current node is not now a rtc element or a ruby element,
                    // this is a parse error.
                    let current_name = &self.current_node_as_element().unwrap().name;
                    if current_name != "rtc" && current_name != "ruby" {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is not a rtc or ruby element",
                        )))?;
                    }
                }

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "math" => {
                self.reconstruct_the_active_formatting_elements()?;

                // TODO: adjust MathML attributes
                // TODO: adjust foreign attributes

                let self_closing = token.self_closing;
                self.insert_foreign_element(token, MATHML_NAMESPACE, false)?;

                if self_closing {
                    self.open_elements.pop();
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "svg" => {
                self.reconstruct_the_active_formatting_elements()?;

                // TODO: adjust SVG attribtues
                // TODO: adjust foreign attributes

                let self_closing = token.self_closing;
                self.insert_foreign_element(token, SVG_NAMESPACE, false)?;

                if self_closing {
                    self.open_elements.pop();
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "caption", "col", "colgroup", "frame", "head", "tbody", "td", "tfoot", "th",
                    "thead", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected {} start tag in body",
                    token.tag_name
                )))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) => {
                self.reconstruct_the_active_formatting_elements()?;

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) => {
                self.any_other_end_tag(&token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    fn any_other_end_tag(&mut self, token: &TagToken) -> Result<(), HtmlParseError> {
        let node = self.current_node_as_element_result()?.clone();

        self.in_body_other_end_tag_loop(0, &node, token)?;

        Ok(())
    }

    fn in_body_other_end_tag_loop(
        &mut self,
        node_index: usize,
        node: &ElementNode,
        token: &TagToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        if node.name == token.tag_name {
            self.generate_implied_end_tags(Some(&token.tag_name))?;

            if node != self.current_node_as_element().unwrap() {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "node is not the same as the current node",
                )))?;
            }

            // pop all nodes from the current node up to node
            while node != self.current_node_as_element_result()? {
                self.open_elements.pop();
            }

            // should now be the same as node, pop it as well
            self.open_elements.pop();

            // stop these steps
            return Ok(Acknowledgement::no());
        }
        // if node is in special category, parse error and ignore token
        else if SPECIAL_ELEMENTS.contains(&node.name.as_str()) {
            self.handle_error(HtmlParserError::MinorError(String::from(
                "node is in special category",
            )))?;
            return Ok(Acknowledgement::no());
        }

        // set node to the previous entry
        let node = self
            .open_elements
            .iter()
            .rev()
            .skip(node_index)
            .next()
            .map(|node_id| {
                self.arena
                    .get(*node_id)
                    .expect("node not found")
                    .get()
                    .as_element_node()
                    .expect("node is not an element node")
                    .clone()
            })
            .expect("node not found");

        self.in_body_other_end_tag_loop(node_index + 1, &node, token)?;

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adoption-agency-algorithm>
    pub(crate) fn adoption_agency_algorithm(
        &mut self,
        token: &TagToken,
    ) -> Result<(), HtmlParseError> {
        let subject = &token.tag_name;

        // Step 1: If the current node is an HTML element whose tag name is subject,
        // and the current node is not in the list of active formatting elements,
        // then pop the current node off the stack of open elements and return.
        if let Some(current) = self.current_node() {
            if let Ok(element) = current.as_element_node() {
                if element.name == *subject {
                    let current_id = self.current_node_id().unwrap();
                    let in_active = self
                        .active_formatting_elements
                        .iter()
                        .any(|e| {
                            if let NodeOrMarker::Node(entry) = e {
                                entry.node_id == current_id
                            } else {
                                false
                            }
                        });
                    if !in_active {
                        self.open_elements.pop();
                        return Ok(());
                    }
                }
            }
        }

        // Step 2
        let mut outer_loop_counter = 0;

        // Step 3: Outer loop
        loop {
            // Step 4.1
            if outer_loop_counter >= 8 {
                return Ok(());
            }

            // Step 4.2
            outer_loop_counter += 1;

            // Step 4.3: Find the formatting element
            let formatting_element_index = self
                .active_formatting_elements
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, e)| {
                    match e {
                        NodeOrMarker::Marker => None, // stop at marker
                        NodeOrMarker::Node(entry) => {
                            let node = self.arena.get(entry.node_id).unwrap().get();
                            if let Ok(element) = node.as_element_node() {
                                if element.name == *subject {
                                    return Some(i);
                                }
                            }
                            None
                        }
                    }
                });

            // Step 4.4: If there is no such element, then return and instead act
            // as described in the "any other end tag" entry
            let formatting_element_index = match formatting_element_index {
                Some(idx) => idx,
                None => {
                    // "any other end tag" behavior
                    self.any_other_end_tag(token)?;
                    return Ok(());
                }
            };

            let formatting_element_entry = match &self.active_formatting_elements[formatting_element_index] {
                NodeOrMarker::Node(entry) => entry.clone(),
                _ => unreachable!(),
            };
            let formatting_element_id = formatting_element_entry.node_id;

            // Step 4.5: If the formatting element is not in the stack of open elements
            let formatting_in_stack = self
                .open_elements
                .iter()
                .position(|id| *id == formatting_element_id);

            if formatting_in_stack.is_none() {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "formatting element is not in the stack of open elements",
                )))?;
                self.active_formatting_elements.remove(formatting_element_index);
                return Ok(());
            }

            let formatting_in_stack_index = formatting_in_stack.unwrap();

            // Step 4.6: If the formatting element is not in scope
            // (simplified: just check it's in the stack)

            // Step 4.7: If the formatting element is not the current node
            if self.current_node_id() != Some(formatting_element_id) {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "formatting element is not the current node",
                )))?;
            }

            // Step 4.8: Find the furthest block
            let furthest_block_index = self
                .open_elements
                .iter()
                .enumerate()
                .skip(formatting_in_stack_index + 1)
                .find_map(|(i, id)| {
                    let node = self.arena.get(*id).unwrap().get();
                    if let Ok(element) = node.as_element_node() {
                        if SPECIAL_ELEMENTS.contains(&element.name.as_str()) {
                            return Some(i);
                        }
                    }
                    None
                });

            // Step 4.9: If there is no furthest block
            if furthest_block_index.is_none() {
                // Pop elements up to and including the formatting element
                while let Some(id) = self.open_elements.pop() {
                    if id == formatting_element_id {
                        break;
                    }
                }
                self.active_formatting_elements.remove(formatting_element_index);
                return Ok(());
            }

            let furthest_block_index = furthest_block_index.unwrap();
            let furthest_block_id = self.open_elements[furthest_block_index];

            // Step 4.10: Let common ancestor be the element immediately above
            // the formatting element in the stack of open elements.
            let common_ancestor_id = self.open_elements[formatting_in_stack_index - 1];

            // Step 4.11: Let a bookmark note the position of the formatting element
            // in the list of active formatting elements relative to the elements on either side of it
            let mut bookmark = formatting_element_index;

            // Step 4.12: Let node and last node be the furthest block.
            let mut node_index = furthest_block_index;
            let mut last_node_id = furthest_block_id;

            // Step 4.13: inner loop counter
            let mut inner_loop_counter = 0;

            // Step 4.14: Inner loop
            loop {
                // Step 4.14.1
                inner_loop_counter += 1;

                // Step 4.14.2: Let node be the element immediately above node in the
                // stack of open elements
                node_index -= 1;
                let node_id = self.open_elements[node_index];

                // Step 4.14.3: If node is the formatting element, then break
                if node_id == formatting_element_id {
                    break;
                }

                // Step 4.14.4: If the inner loop counter is greater than 3 and node
                // is in the list of active formatting elements, then remove node from
                // the list of active formatting elements
                let node_active_index = self
                    .active_formatting_elements
                    .iter()
                    .position(|e| {
                        if let NodeOrMarker::Node(entry) = e {
                            entry.node_id == node_id
                        } else {
                            false
                        }
                    });

                if inner_loop_counter > 3 {
                    if let Some(active_idx) = node_active_index {
                        self.active_formatting_elements.remove(active_idx);
                        if active_idx < bookmark {
                            bookmark -= 1;
                        }
                        continue;
                    }
                }

                // Step 4.14.5: If node is not in the list of active formatting elements,
                // remove node from the stack of open elements and continue
                let node_active_index = match self.active_formatting_elements.iter().position(|e| {
                    if let NodeOrMarker::Node(entry) = e {
                        entry.node_id == node_id
                    } else {
                        false
                    }
                }) {
                    Some(idx) => idx,
                    None => {
                        self.open_elements.remove(node_index);
                        // Adjust furthest_block_index if needed
                        continue;
                    }
                };

                // Step 4.14.6: Create an element for the token for which the element
                // "node" was created, in the HTML namespace, with common ancestor as
                // the intended parent; replace the entry for "node" in the list of
                // active formatting elements with an entry for the new element, replace
                // the entry for "node" in the stack of open elements with an entry for
                // the new element, and let "node" be the new element.
                let old_token = match &self.active_formatting_elements[node_active_index] {
                    NodeOrMarker::Node(entry) => entry.token.clone(),
                    _ => unreachable!(),
                };

                let new_element_id = self.create_an_element_for_the_token(old_token.clone(), super::HTML_NAMESPACE)?;
                let new_node_id = self.insert_create_an_element_for_the_token_result(new_element_id)?;

                // Replace in active formatting elements
                self.active_formatting_elements[node_active_index] =
                    NodeOrMarker::Node(NodeEntry {
                        node_id: new_node_id,
                        token: old_token,
                    });

                // Replace in open elements
                self.open_elements[node_index] = new_node_id;

                let node_id = new_node_id;

                // Step 4.14.7: If last node is the furthest block, move the bookmark
                // to be immediately after the new node in the list of active formatting elements
                if last_node_id == furthest_block_id {
                    bookmark = node_active_index + 1;
                }

                // Step 4.14.8: Append last node to node
                last_node_id.detach(&mut self.arena);
                node_id.append(last_node_id, &mut self.arena);

                // Step 4.14.9: Set last node to node
                last_node_id = node_id;
            }

            // Step 4.15: Insert whatever last node ended up being in the appropriate
            // place for inserting a node, but using common ancestor as the override target.
            last_node_id.detach(&mut self.arena);
            common_ancestor_id.append(last_node_id, &mut self.arena);

            // Step 4.16: Create an element for the token for which the formatting element
            // was created, in the HTML namespace, with the furthest block as the intended parent
            let new_element_id = self.create_an_element_for_the_token(
                formatting_element_entry.token.clone(),
                super::HTML_NAMESPACE,
            )?;
            let new_formatting_id = self.insert_create_an_element_for_the_token_result(new_element_id)?;

            // Step 4.17: Take all of the child nodes of the furthest block and append
            // them to the new element
            let children: Vec<NodeId> = furthest_block_id
                .children(&self.arena)
                .collect();
            for child_id in children {
                child_id.detach(&mut self.arena);
                new_formatting_id.append(child_id, &mut self.arena);
            }

            // Step 4.18: Append the new element to the furthest block
            furthest_block_id.append(new_formatting_id, &mut self.arena);

            // Step 4.19: Remove the formatting element from the list of active
            // formatting elements, and insert the new element into the list of
            // active formatting elements at the position of the aforementioned bookmark.
            self.active_formatting_elements.remove(formatting_element_index);
            let insert_pos = if bookmark > self.active_formatting_elements.len() {
                self.active_formatting_elements.len()
            } else {
                bookmark
            };
            self.active_formatting_elements.insert(
                insert_pos,
                NodeOrMarker::Node(NodeEntry {
                    node_id: new_formatting_id,
                    token: formatting_element_entry.token.clone(),
                }),
            );

            // Step 4.20: Remove the formatting element from the stack of open elements,
            // and insert the new element into the stack of open elements immediately
            // below the position of the furthest block in that stack.
            self.open_elements.retain(|id| *id != formatting_element_id);
            let fb_pos = self
                .open_elements
                .iter()
                .position(|id| *id == furthest_block_id)
                .unwrap();
            self.open_elements.insert(fb_pos + 1, new_formatting_id);
        }
    }
}
