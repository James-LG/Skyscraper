use crate::{
    html::grammar::{tokenizer::TokenizerState, NodeOrMarker},
    xpath::grammar::{
        data_model::DoctypeNode,
        XpathItemTreeNode,
    },
};

use super::{
    chars,
    tokenizer::{HtmlToken, Parser, TagToken, TagTokenType},
    Acknowledgement, HtmlParseError, HtmlParser, HtmlParserError, InsertionMode, QuirksMode,
    HTML_NAMESPACE,
};

pub(crate) mod in_body_insertion_mode;
pub(crate) mod in_foreign_content;

/// Determine the quirks mode from a DOCTYPE token per WHATWG 13.2.6.4.1.
fn determine_quirks_mode(
    force_quirks: bool,
    name: &str,
    public_id: Option<&str>,
    system_id: Option<&str>,
) -> QuirksMode {
    if force_quirks || name != "html" {
        return QuirksMode::Quirks;
    }

    let public_lower = public_id.map(|s| s.to_ascii_lowercase());
    let public_lower_ref = public_lower.as_deref();

    // Exact public identifier matches that trigger quirks mode.
    const QUIRKY_PUBLIC_MATCHES: &[&str] = &[
        "-//w3o//dtd w3 html strict 3.0//en//",
        "-/w3c/dtd html 4.0 transitional/en",
        "html",
    ];

    if let Some(pid) = public_lower_ref {
        if QUIRKY_PUBLIC_MATCHES.contains(&pid) {
            return QuirksMode::Quirks;
        }
    }

    // Exact system identifier match that triggers quirks mode.
    if let Some(sid) = system_id {
        if sid.eq_ignore_ascii_case("http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd") {
            return QuirksMode::Quirks;
        }
    }

    // Public identifier prefixes that trigger quirks mode (case-insensitive).
    const QUIRKY_PUBLIC_PREFIXES: &[&str] = &[
        "+//silmaril//dtd html pro v0r11 19970101//",
        "-//as//dtd html 3.0 aswedit + extensions//",
        "-//advasoft ltd//dtd html 3.0 aswedit + extensions//",
        "-//ietf//dtd html 2.0 level 1//",
        "-//ietf//dtd html 2.0 level 2//",
        "-//ietf//dtd html 2.0 strict level 1//",
        "-//ietf//dtd html 2.0 strict level 2//",
        "-//ietf//dtd html 2.0 strict//",
        "-//ietf//dtd html 2.0//",
        "-//ietf//dtd html 2.1e//",
        "-//ietf//dtd html 3.0//",
        "-//ietf//dtd html 3.2 final//",
        "-//ietf//dtd html 3.2//",
        "-//ietf//dtd html 3//",
        "-//ietf//dtd html level 0//",
        "-//ietf//dtd html level 1//",
        "-//ietf//dtd html level 2//",
        "-//ietf//dtd html level 3//",
        "-//ietf//dtd html strict level 0//",
        "-//ietf//dtd html strict level 1//",
        "-//ietf//dtd html strict level 2//",
        "-//ietf//dtd html strict level 3//",
        "-//ietf//dtd html strict//",
        "-//ietf//dtd html//",
        "-//metrius//dtd metrius presentational//",
        "-//microsoft//dtd internet explorer 2.0 html strict//",
        "-//microsoft//dtd internet explorer 2.0 html//",
        "-//microsoft//dtd internet explorer 2.0 tables//",
        "-//microsoft//dtd internet explorer 3.0 html strict//",
        "-//microsoft//dtd internet explorer 3.0 html//",
        "-//microsoft//dtd internet explorer 3.0 tables//",
        "-//netscape comm. corp.//dtd html//",
        "-//netscape comm. corp.//dtd strict html//",
        "-//o'reilly and associates//dtd html 2.0//",
        "-//o'reilly and associates//dtd html extended 1.0//",
        "-//o'reilly and associates//dtd html extended relaxed 1.0//",
        "-//sq//dtd html 2.0 hotmetal + extensions//",
        "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
        "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
        "-//spyglass//dtd html 2.0 extended//",
        "-//sun microsystems corp.//dtd hotjava html//",
        "-//sun microsystems corp.//dtd hotjava strict html//",
        "-//w3c//dtd html 3 1995-03-24//",
        "-//w3c//dtd html 3.2 draft//",
        "-//w3c//dtd html 3.2 final//",
        "-//w3c//dtd html 3.2//",
        "-//w3c//dtd html 3.2s draft//",
        "-//w3c//dtd html 4.0 frameset//",
        "-//w3c//dtd html 4.0 transitional//",
        "-//w3c//dtd html experimental 19960712//",
        "-//w3c//dtd html experimental 970421//",
        "-//w3c//dtd w3 html//",
        "-//w3o//dtd w3 html 3.0//",
        "-//webtechs//dtd mozilla html 2.0//",
        "-//webtechs//dtd mozilla html//",
    ];

    if let Some(pid) = public_lower_ref {
        if QUIRKY_PUBLIC_PREFIXES.iter().any(|prefix| pid.starts_with(prefix)) {
            return QuirksMode::Quirks;
        }
    }

    // Public identifier prefixes that trigger quirks if system identifier is missing.
    const HTML4_PUBLIC_PREFIXES: &[&str] = &[
        "-//w3c//dtd html 4.01 frameset//",
        "-//w3c//dtd html 4.01 transitional//",
    ];

    if system_id.is_none() {
        if let Some(pid) = public_lower_ref {
            if HTML4_PUBLIC_PREFIXES.iter().any(|prefix| pid.starts_with(prefix)) {
                return QuirksMode::Quirks;
            }
        }
    }

    // Limited quirks mode checks.
    const LIMITED_QUIRKY_PUBLIC_PREFIXES: &[&str] = &[
        "-//w3c//dtd xhtml 1.0 frameset//",
        "-//w3c//dtd xhtml 1.0 transitional//",
    ];

    if let Some(pid) = public_lower_ref {
        if LIMITED_QUIRKY_PUBLIC_PREFIXES.iter().any(|prefix| pid.starts_with(prefix)) {
            return QuirksMode::LimitedQuirks;
        }
    }

    // HTML 4.01 Frameset/Transitional with system identifier → limited quirks.
    if system_id.is_some() {
        if let Some(pid) = public_lower_ref {
            if HTML4_PUBLIC_PREFIXES.iter().any(|prefix| pid.starts_with(prefix)) {
                return QuirksMode::LimitedQuirks;
            }
        }
    }

    QuirksMode::NoQuirks
}

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
                // Determine quirks mode per WHATWG 13.2.6.4.1.
                self.quirks_mode = determine_quirks_mode(
                    doctype.force_quirks,
                    &doctype.name,
                    doctype.public_identifier.as_deref(),
                    doctype.system_identifier.as_deref(),
                );

                // Append a DocumentType node to the Document node.
                let doctype_id = DoctypeNode::create(
                    doctype.name,
                    doctype.public_identifier,
                    doctype.system_identifier,
                    &mut self.arena,
                );

                self.root_node
                    .ok_or(HtmlParseError::new("root node is None"))?
                    .append(doctype_id, &mut self.arena);

                self.insertion_mode = InsertionMode::BeforeHtml;
            }
            _ => {
                // WHATWG 13.2.6.4.1: If the document is not an iframe srcdoc document,
                // this is a parse error; set the Document to quirks mode.
                self.quirks_mode = QuirksMode::Quirks;

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
            HtmlToken::DocType(_) => {
                // parse error, ignore the token
            }
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
                // parse error, ignore the token
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
                self.insert_character(c)?;
            }
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => {
                // parse error, ignore the token
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                // process the token using the rules for the "in body" insertion mode
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
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
                // parse error, ignore the token
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
                self.insert_character(c)?;
            }
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => {
                // parse error, ignore the token
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                // process the token using the rules for the "in body" insertion mode
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
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
                let _node = self.insert_an_html_element(token)?;

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

                // Declarative shadow DOM is not supported, so shadowrootmode is
                // always in the None state. Per WHATWG step 9, when any of the
                // three conditions is false we simply insert an HTML element.
                self.insert_an_html_element(token)?;
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
                // parse error, ignore the token
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
                self.insert_character(c)?;
            }
            HtmlToken::Comment(comment) => {
                // A comment token: Insert a comment.
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => {
                // A DOCTYPE token: Parse error. Ignore the token.
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                // Process the token using the rules for the "in body" insertion mode.
                return self
                    .in_body_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "body" => {
                self.insert_an_html_element(token)?;

                self.frameset_ok = false;

                self.insertion_mode = InsertionMode::InBody;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "frameset" => {
                self.insert_an_html_element(token)?;
                self.insertion_mode = InsertionMode::InFrameset;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                // Parse error.
                // Push the node pointed to by the head element pointer onto the stack of open
                // elements.
                let head_node_id = self
                    .head_element_pointer
                    .ok_or(HtmlParseError::new("head element pointer is None"))?;
                self.open_elements.push(head_node_id);

                // Process the token using the rules for the "in head" insertion mode.
                let ack =
                    self.in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));

                // Remove the node pointed to by the head element pointer from the stack of open
                // elements. (It might not be the current node at this point.)
                self.open_elements.retain(|&id| id != head_node_id);

                return ack;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                // Process the token using the rules for the "in head" insertion mode.
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::EndTag(token)));
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["body", "html", "br"].contains(&token.tag_name.as_str()) =>
            {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(_token)) if _token.tag_name == "head" => {
                // A start tag whose tag name is "head": Parse error. Ignore the token.
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                // Any other end tag: Parse error. Ignore the token.
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
                self.insert_character(c)?;
            }
            HtmlToken::Characters(ref s) => {
                self.insert_characters(s)?;
            }
            HtmlToken::EndOfFile => {
                // Parse error.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected end-of-file in text insertion mode",
                )))?;

                // If the current node is a script element, then set its
                // "already started" flag. (Scripting is not supported.)

                // Pop the current node off the stack of open elements.
                self.open_elements.pop().expect("open elements is empty");

                // Switch the insertion mode to the original insertion mode.
                self.insertion_mode = self
                    .original_insertion_mode
                    .expect("original insertion mode is None");

                // Reprocess the token.
                self.token_emitted(HtmlToken::EndOfFile)?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "script" => {
                let _script = self.current_node_as_element_result()?;

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
            HtmlToken::Character(_)
            | HtmlToken::Characters(_)
            | HtmlToken::Comment(_)
            | HtmlToken::DocType(_) => {
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
                if !self.open_elements_has_element("template") {
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
            HtmlToken::Comment(comment) => {
                // Insert a comment as the last child of the first element in
                // the stack of open elements (the html element).
                let html_node = *self.open_elements.first().ok_or(
                    HtmlParseError::new("no elements on open elements stack"),
                )?;
                self.insert_a_comment(comment, Some(html_node))?;
            }
            HtmlToken::DocType(_) => {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected DOCTYPE after body",
                )))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "html" => {
                if self.is_fragment_parser() {
                    // Parse error. Ignore the token.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "unexpected </html> end tag in fragment parser after body",
                    )))?;
                } else {
                    self.insertion_mode = InsertionMode::AfterAfterBody;
                }
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
            HtmlToken::Comment(comment) => {
                // Insert a comment as the last child of the Document object.
                let parent = self
                    .root_node
                    .ok_or(HtmlParseError::new("root node is None"))?;
                self.insert_a_comment(comment, Some(parent))?;
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

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inframeset>
    pub(super) fn in_frameset_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A character token that is one of U+0009, U+000A, U+000C, U+000D, or U+0020
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
                self.insert_character(c)?;
            }
            // A comment token
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            // A DOCTYPE token: parse error, ignore
            HtmlToken::DocType(_) => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected DOCTYPE in frameset",
                )))?;
            }
            // A start tag whose tag name is "html"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                return self
                    .in_body_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // A start tag whose tag name is "frameset"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if token.tag_name == "frameset" =>
            {
                self.insert_an_html_element(token)?;
            }
            // An end tag whose tag name is "frameset"
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if token.tag_name == "frameset" =>
            {
                // If the current node is the root html element, this is a parse error; ignore.
                let is_root = self.open_elements.len() == 1;
                if is_root {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "frameset end tag at root html element",
                    )))?;
                } else {
                    // Pop the current node from the stack of open elements.
                    self.open_elements.pop();

                    // If the parser was not created as part of the HTML fragment parsing algorithm
                    // (fragment case), and the current node is no longer a frameset element, then
                    // switch the insertion mode to "after frameset".
                    if !self.is_fragment_parser() {
                        let is_frameset = self
                            .current_node_as_element()
                            .map(|el| el.name == "frameset")
                            .unwrap_or(false);
                        if !is_frameset {
                            self.insertion_mode = InsertionMode::AfterFrameset;
                        }
                    }
                }
            }
            // A start tag whose tag name is "frame"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "frame" => {
                self.insert_an_html_element(token)?;
                // Immediately pop the current node off the stack of open elements.
                self.open_elements.pop();
                // Acknowledge the token's self-closing flag, if it is set.
                return Ok(Acknowledgement::yes());
            }
            // A start tag whose tag name is "noframes"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if token.tag_name == "noframes" =>
            {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // An end-of-file token
            HtmlToken::EndOfFile => {
                if self.open_elements.len() > 1 {
                    // If the current node is not the root html element, this is a parse error.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "unexpected EOF in frameset",
                    )))?;
                }
                self.stop_parsing()?;
            }
            // Anything else: parse error, ignore the token.
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token in frameset",
                )))?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterframeset>
    pub(super) fn after_frameset_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A character token that is one of U+0009, U+000A, U+000C, U+000D, or U+0020
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
                self.insert_character(c)?;
            }
            // A comment token
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            // A DOCTYPE token: parse error, ignore
            HtmlToken::DocType(_) => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected DOCTYPE after frameset",
                )))?;
            }
            // A start tag whose tag name is "html"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                return self
                    .in_body_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // An end tag whose tag name is "html"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "html" => {
                self.insertion_mode = InsertionMode::AfterAfterFrameset;
            }
            // A start tag whose tag name is "noframes"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if token.tag_name == "noframes" =>
            {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // An end-of-file token
            HtmlToken::EndOfFile => {
                self.stop_parsing()?;
            }
            // Anything else: parse error, ignore the token.
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token after frameset",
                )))?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-frameset-insertion-mode>
    pub(super) fn after_after_frameset_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A comment token: insert a comment as the last child of the Document object.
            HtmlToken::Comment(comment) => {
                let parent = self
                    .root_node
                    .ok_or(HtmlParseError::new("root node is None"))?;

                self.insert_a_comment(comment, Some(parent))?;
            }
            // A DOCTYPE token: process using the rules for the "in body" insertion mode.
            HtmlToken::DocType(_) => {
                self.using_the_rules_for(token, InsertionMode::InBody)?;
            }
            // A character token that is one of U+0009, U+000A, U+000C, U+000D, or U+0020:
            // process using the rules for the "in body" insertion mode.
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
                self.using_the_rules_for(HtmlToken::Character(c), InsertionMode::InBody)?;
            }
            // A start tag whose tag name is "html"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
            }
            // A start tag whose tag name is "noframes"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if token.tag_name == "noframes" =>
            {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // An end-of-file token
            HtmlToken::EndOfFile => {
                self.stop_parsing()?;
            }
            // Anything else: parse error, ignore the token.
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token after after frameset",
                )))?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intable>
    pub(super) fn in_table_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A character token, if the current node is table, tbody, template, tfoot, thead, or tr element:
            HtmlToken::Character(c) => {
                let is_table_text_element = self
                    .current_node_as_element()
                    .map(|el| {
                        matches!(
                            el.name.as_str(),
                            "table" | "tbody" | "template" | "tfoot" | "thead" | "tr"
                        )
                    })
                    .unwrap_or(false);

                if is_table_text_element {
                    // Let the pending table character tokens be an empty list of tokens.
                    self.pending_table_character_tokens = Vec::new();
                    // Let the original insertion mode be the current insertion mode.
                    self.original_insertion_mode = Some(self.insertion_mode);
                    // Switch the insertion mode to "in table text" and reprocess the token.
                    self.insertion_mode = InsertionMode::InTableText;
                    self.token_emitted(HtmlToken::Character(c))?;
                } else {
                    // Anything else
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "unexpected character token in table",
                    )))?;
                    self.foster_parenting = true;
                    self.using_the_rules_for(HtmlToken::Character(c), InsertionMode::InBody)?;
                    self.foster_parenting = false;
                }
            }
            // Batched characters in table: fall back to per-character processing
            // because the mode may switch between InTable and InTableText per char.
            HtmlToken::Characters(s) => {
                for c in s.chars() {
                    self.token_emitted(HtmlToken::Character(c))?;
                }
            }
            // A comment token
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            // A DOCTYPE token
            HtmlToken::DocType(_) => {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected DOCTYPE in table",
                )))?;
            }
            // A start tag whose tag name is "caption"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "caption" => {
                self.clear_the_stack_back_to_a_table_context();
                self.active_formatting_elements
                    .push(NodeOrMarker::Marker);
                self.insert_an_html_element(token)?;
                self.insertion_mode = InsertionMode::InCaption;
            }
            // A start tag whose tag name is "colgroup"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if token.tag_name == "colgroup" =>
            {
                self.clear_the_stack_back_to_a_table_context();
                self.insert_an_html_element(token)?;
                self.insertion_mode = InsertionMode::InColumnGroup;
            }
            // A start tag whose tag name is "col"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "col" => {
                self.clear_the_stack_back_to_a_table_context();
                self.insert_an_html_element(TagToken::new(String::from("colgroup")))?;
                self.insertion_mode = InsertionMode::InColumnGroup;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            // A start tag whose tag name is one of: "tbody", "tfoot", "thead"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["tbody", "tfoot", "thead"].contains(&token.tag_name.as_str()) =>
            {
                self.clear_the_stack_back_to_a_table_context();
                self.insert_an_html_element(token)?;
                self.insertion_mode = InsertionMode::InTableBody;
            }
            // A start tag whose tag name is one of: "td", "th", "tr"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["td", "th", "tr"].contains(&token.tag_name.as_str()) =>
            {
                self.clear_the_stack_back_to_a_table_context();
                self.insert_an_html_element(TagToken::new(String::from("tbody")))?;
                self.insertion_mode = InsertionMode::InTableBody;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            // A start tag whose tag name is "table"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "table" => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "nested table start tag",
                )))?;

                if !self.has_an_element_in_table_scope("table") {
                    // Ignore the token.
                } else {
                    self.pop_until_tag_name("table")?;
                    self.reset_the_insertion_mode_appropriately()?;
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
                }
            }
            // An end tag whose tag name is "table"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "table" => {
                if !self.has_an_element_in_table_scope("table") {
                    // Parse error. Ignore the token.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "table end tag without table in scope",
                    )))?;
                } else {
                    self.pop_until_tag_name("table")?;
                    self.reset_the_insertion_mode_appropriately()?;
                }
            }
            // An end tag whose tag name is one of: "body", "caption", "col", "colgroup", "html",
            // "tbody", "td", "tfoot", "th", "thead", "tr"
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if [
                    "body", "caption", "col", "colgroup", "html", "tbody", "td", "tfoot", "th",
                    "thead", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected end tag </{}> in table",
                    token.tag_name
                )))?;
            }
            // A start tag whose tag name is one of: "style", "script", "template"
            // An end tag whose tag name is "template"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["style", "script", "template"].contains(&token.tag_name.as_str()) =>
            {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::EndTag(token)));
            }
            // A start tag whose tag name is "input"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "input" => {
                let is_hidden_input = token.attributes.iter().any(|attr| {
                    attr.name.eq_ignore_ascii_case("type")
                        && attr.value.eq_ignore_ascii_case("hidden")
                });

                if !is_hidden_input {
                    // Anything else
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "unexpected input in table (not hidden)",
                    )))?;
                    self.foster_parenting = true;
                    self.using_the_rules_for(
                        HtmlToken::TagToken(TagTokenType::StartTag(token)),
                        InsertionMode::InBody,
                    )?;
                    self.foster_parenting = false;
                } else {
                    // Parse error.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "input type=hidden in table",
                    )))?;
                    self.insert_an_html_element(token)?;
                    self.open_elements.pop();
                    // Acknowledge the token's self-closing flag, if it is set.
                    return Ok(Acknowledgement::yes());
                }
            }
            // A start tag whose tag name is "form"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "form" => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "form start tag in table",
                )))?;

                if self.open_elements_has_element("template")
                    || self.form_element_pointer.is_some()
                {
                    // Ignore the token.
                } else {
                    let form_id = self.insert_an_html_element(token)?;
                    self.form_element_pointer = Some(form_id);
                    self.open_elements.pop();
                }
            }
            // An end-of-file token
            HtmlToken::EndOfFile => {
                return self.in_body_insertion_mode(HtmlToken::EndOfFile);
            }
            // Anything else
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token in table, foster parenting",
                )))?;
                self.foster_parenting = true;
                self.using_the_rules_for(token, InsertionMode::InBody)?;
                self.foster_parenting = false;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intabletext>
    pub(super) fn in_table_text_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A character token that is U+0000 NULL
            HtmlToken::Character('\0') => {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "null character in table text",
                )))?;
            }
            // Any other character token
            HtmlToken::Character(c) => {
                self.pending_table_character_tokens
                    .push(HtmlToken::Character(c));
            }
            // Batched characters: push each individually into pending tokens.
            HtmlToken::Characters(s) => {
                for c in s.chars() {
                    if c == '\0' {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "null character in table text",
                        )))?;
                    } else {
                        self.pending_table_character_tokens
                            .push(HtmlToken::Character(c));
                    }
                }
            }
            // Anything else
            _ => {
                // If any of the tokens in the pending table character tokens list
                // are character tokens that are not ASCII whitespace:
                let has_non_whitespace = self.pending_table_character_tokens.iter().any(|t| {
                    if let HtmlToken::Character(c) = t {
                        ![
                            chars::CHARACTER_TABULATION,
                            chars::LINE_FEED,
                            chars::FORM_FEED,
                            chars::CARRIAGE_RETURN,
                            chars::SPACE,
                        ]
                        .contains(c)
                    } else {
                        false
                    }
                });

                let pending_tokens = std::mem::take(&mut self.pending_table_character_tokens);

                if has_non_whitespace {
                    // This is a parse error. Reprocess the character tokens using
                    // the rules for the "anything else" entry in the "in table" insertion mode.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "non-whitespace character in table text",
                    )))?;
                    for pending_token in pending_tokens {
                        self.foster_parenting = true;
                        self.using_the_rules_for(pending_token, InsertionMode::InBody)?;
                        self.foster_parenting = false;
                    }
                } else {
                    // Otherwise, insert the characters given by the pending table character
                    // tokens list.
                    for pending_token in pending_tokens {
                        if let HtmlToken::Character(c) = pending_token {
                            self.insert_character(c)?;
                        }
                    }
                }

                // Switch the insertion mode to the original insertion mode and reprocess the token.
                self.insertion_mode = self
                    .original_insertion_mode
                    .expect("original insertion mode is None");
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// Close the caption element: generate implied end tags, pop until caption,
    /// clear active formatting elements, and switch to InTable.
    /// Returns true if the caption was closed, false if no caption was in table scope.
    fn close_the_caption(&mut self) -> Result<bool, HtmlParseError> {
        if !self.has_an_element_in_table_scope("caption") {
            self.handle_error(HtmlParserError::MinorError(String::from(
                "no caption in table scope",
            )))?;
            return Ok(false);
        }

        self.generate_implied_end_tags(None)?;

        if let Some(el) = self.current_node_as_element() {
            if el.name != "caption" {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "current node is not caption when closing caption",
                )))?;
            }
        }

        self.pop_until_tag_name("caption")?;
        self.clear_the_list_of_active_formatting_elements_up_to_the_last_marker()?;
        self.insertion_mode = InsertionMode::InTable;
        Ok(true)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incaption>
    pub(super) fn in_caption_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // An end tag whose tag name is "caption"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "caption" => {
                self.close_the_caption()?;
            }
            // A start tag whose tag name is one of: "caption", "col", "colgroup", "tbody", "td",
            // "tfoot", "th", "thead", "tr"
            // An end tag whose tag name is "table"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "caption", "col", "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                if self.close_the_caption()? {
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "table" => {
                if self.close_the_caption()? {
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
                }
            }
            // An end tag whose tag name is one of: "body", "col", "colgroup", "html", "tbody",
            // "td", "tfoot", "th", "thead", "tr"
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if [
                    "body", "col", "colgroup", "html", "tbody", "td", "tfoot", "th", "thead",
                    "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected end tag </{}> in caption",
                    token.tag_name
                )))?;
            }
            // Anything else: process using InBody rules
            _ => {
                return self.in_body_insertion_mode(token);
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incolumngroup>
    pub(super) fn in_column_group_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A character token that is whitespace
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
                self.insert_character(c)?;
            }
            // A comment token
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            // A DOCTYPE token
            HtmlToken::DocType(_) => {
                // Parse error. Ignore.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected DOCTYPE in column group",
                )))?;
            }
            // A start tag whose tag name is "html"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                return self
                    .in_body_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // A start tag whose tag name is "col"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "col" => {
                self.insert_an_html_element(token)?;
                self.open_elements.pop();
                return Ok(Acknowledgement::yes());
            }
            // An end tag whose tag name is "colgroup"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "colgroup" => {
                let is_colgroup = self
                    .current_node_as_element()
                    .map(|el| el.name == "colgroup")
                    .unwrap_or(false);

                if !is_colgroup {
                    // Parse error. Ignore.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "current node is not colgroup",
                    )))?;
                } else {
                    self.open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                }
            }
            // An end tag whose tag name is "col"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "col" => {
                // Parse error. Ignore.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected </col> end tag",
                )))?;
            }
            // A start tag whose tag name is "template"
            // An end tag whose tag name is "template"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if token.tag_name == "template" =>
            {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::EndTag(token)));
            }
            // An end-of-file token
            HtmlToken::EndOfFile => {
                return self.in_body_insertion_mode(HtmlToken::EndOfFile);
            }
            // Anything else
            _ => {
                let is_colgroup = self
                    .current_node_as_element()
                    .map(|el| el.name == "colgroup")
                    .unwrap_or(false);

                if !is_colgroup {
                    // Parse error. Ignore.
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "current node is not colgroup, ignoring token",
                    )))?;
                } else {
                    self.open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                    self.token_emitted(token)?;
                }
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intablebody>
    pub(super) fn in_table_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A start tag whose tag name is "tr"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "tr" => {
                self.clear_the_stack_back_to_a_table_body_context();
                self.insert_an_html_element(token)?;
                self.insertion_mode = InsertionMode::InRow;
            }
            // A start tag whose tag name is one of: "th", "td"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["th", "td"].contains(&token.tag_name.as_str()) =>
            {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "td/th start tag directly in table body",
                )))?;
                self.clear_the_stack_back_to_a_table_body_context();
                self.insert_an_html_element(TagToken::new(String::from("tr")))?;
                self.insertion_mode = InsertionMode::InRow;
                self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
            }
            // An end tag whose tag name is one of: "tbody", "tfoot", "thead"
            HtmlToken::TagToken(TagTokenType::EndTag(ref end_token))
                if ["tbody", "tfoot", "thead"].contains(&end_token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_table_scope(&end_token.tag_name) {
                    self.handle_error(HtmlParserError::MinorError(format!(
                        "no {} in table scope",
                        end_token.tag_name
                    )))?;
                } else {
                    self.clear_the_stack_back_to_a_table_body_context();
                    self.open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                }
            }
            // A start tag whose tag name is one of: "caption", "col", "colgroup", "tbody",
            // "tfoot", "thead"
            // An end tag whose tag name is "table"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["caption", "col", "colgroup", "tbody", "tfoot", "thead"]
                    .contains(&token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_table_scope("tbody")
                    && !self.has_an_element_in_table_scope("thead")
                    && !self.has_an_element_in_table_scope("tfoot")
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "no tbody/thead/tfoot in table scope",
                    )))?;
                } else {
                    self.clear_the_stack_back_to_a_table_body_context();
                    self.open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "table" => {
                if !self.has_an_element_in_table_scope("tbody")
                    && !self.has_an_element_in_table_scope("thead")
                    && !self.has_an_element_in_table_scope("tfoot")
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "no tbody/thead/tfoot in table scope",
                    )))?;
                } else {
                    self.clear_the_stack_back_to_a_table_body_context();
                    self.open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
                }
            }
            // An end tag whose tag name is one of: "body", "caption", "col", "colgroup", "html",
            // "td", "th", "tr"
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if [
                    "body", "caption", "col", "colgroup", "html", "td", "th", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected end tag </{}> in table body",
                    token.tag_name
                )))?;
            }
            // Anything else: process using InTable rules
            _ => {
                return self.in_table_insertion_mode(token);
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inrow>
    pub(super) fn in_row_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A start tag whose tag name is one of: "th", "td"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["th", "td"].contains(&token.tag_name.as_str()) =>
            {
                self.clear_the_stack_back_to_a_table_row_context();
                self.insert_an_html_element(token)?;
                self.insertion_mode = InsertionMode::InCell;
                self.active_formatting_elements
                    .push(NodeOrMarker::Marker);
            }
            // An end tag whose tag name is "tr"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "tr" => {
                if !self.has_an_element_in_table_scope("tr") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "no tr in table scope",
                    )))?;
                } else {
                    self.clear_the_stack_back_to_a_table_row_context();
                    self.open_elements.pop(); // pop the tr
                    self.insertion_mode = InsertionMode::InTableBody;
                }
            }
            // A start tag whose tag name is one of: "caption", "col", "colgroup", "tbody",
            // "tfoot", "thead", "tr"
            // An end tag whose tag name is "table"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "caption", "col", "colgroup", "tbody", "tfoot", "thead", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_table_scope("tr") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "no tr in table scope",
                    )))?;
                } else {
                    self.clear_the_stack_back_to_a_table_row_context();
                    self.open_elements.pop(); // pop the tr
                    self.insertion_mode = InsertionMode::InTableBody;
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "table" => {
                if !self.has_an_element_in_table_scope("tr") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "no tr in table scope",
                    )))?;
                } else {
                    self.clear_the_stack_back_to_a_table_row_context();
                    self.open_elements.pop(); // pop the tr
                    self.insertion_mode = InsertionMode::InTableBody;
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
                }
            }
            // An end tag whose tag name is one of: "tbody", "tfoot", "thead"
            HtmlToken::TagToken(TagTokenType::EndTag(ref end_token))
                if ["tbody", "tfoot", "thead"].contains(&end_token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_table_scope(&end_token.tag_name) {
                    self.handle_error(HtmlParserError::MinorError(format!(
                        "no {} in table scope",
                        end_token.tag_name
                    )))?;
                } else if !self.has_an_element_in_table_scope("tr") {
                    // Ignore the token.
                } else {
                    self.clear_the_stack_back_to_a_table_row_context();
                    self.open_elements.pop(); // pop the tr
                    self.insertion_mode = InsertionMode::InTableBody;
                    self.token_emitted(token)?;
                }
            }
            // An end tag whose tag name is one of: "body", "caption", "col", "colgroup", "html",
            // "td", "th"
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["body", "caption", "col", "colgroup", "html", "td", "th"]
                    .contains(&token.tag_name.as_str()) =>
            {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected end tag </{}> in row",
                    token.tag_name
                )))?;
            }
            // Anything else: process using InTable rules
            _ => {
                return self.in_table_insertion_mode(token);
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incell>
    pub(super) fn in_cell_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // An end tag whose tag name is one of: "td", "th"
            HtmlToken::TagToken(TagTokenType::EndTag(ref end_token))
                if ["td", "th"].contains(&end_token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_table_scope(&end_token.tag_name) {
                    self.handle_error(HtmlParserError::MinorError(format!(
                        "no {} in table scope",
                        end_token.tag_name
                    )))?;
                } else {
                    self.generate_implied_end_tags(None)?;

                    if let Some(el) = self.current_node_as_element() {
                        if el.name != end_token.tag_name {
                            self.handle_error(HtmlParserError::MinorError(format!(
                                "current node is not {}",
                                end_token.tag_name
                            )))?;
                        }
                    }

                    let tag_name = end_token.tag_name.clone();
                    self.pop_until_tag_name(&tag_name)?;
                    self.clear_the_list_of_active_formatting_elements_up_to_the_last_marker()?;
                    self.insertion_mode = InsertionMode::InRow;
                }
            }
            // A start tag whose tag name is one of: "caption", "col", "colgroup", "tbody", "td",
            // "tfoot", "th", "thead", "tr"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "caption", "col", "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                // Assert: stack has td or th in table scope
                if !self.has_an_element_in_table_scope("td")
                    && !self.has_an_element_in_table_scope("th")
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "no td or th in table scope",
                    )))?;
                } else {
                    self.close_the_cell()?;
                    self.token_emitted(HtmlToken::TagToken(TagTokenType::StartTag(token)))?;
                }
            }
            // An end tag whose tag name is one of: "body", "caption", "col", "colgroup", "html"
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["body", "caption", "col", "colgroup", "html"]
                    .contains(&token.tag_name.as_str()) =>
            {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected end tag </{}> in cell",
                    token.tag_name
                )))?;
            }
            // An end tag whose tag name is one of: "table", "tbody", "tfoot", "thead", "tr"
            HtmlToken::TagToken(TagTokenType::EndTag(ref end_token))
                if ["table", "tbody", "tfoot", "thead", "tr"]
                    .contains(&end_token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_table_scope(&end_token.tag_name) {
                    self.handle_error(HtmlParserError::MinorError(format!(
                        "no {} in table scope",
                        end_token.tag_name
                    )))?;
                } else {
                    self.close_the_cell()?;
                    self.token_emitted(token)?;
                }
            }
            // Anything else: process using InBody rules
            _ => {
                return self.in_body_insertion_mode(token);
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inselect>
    pub(super) fn in_select_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A character token that is U+0000 NULL
            HtmlToken::Character('\0') => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected null character in select",
                )))?;
            }
            // Any other character token
            HtmlToken::Character(c) => {
                self.insert_character(c)?;
            }
            // Batched characters in select
            HtmlToken::Characters(ref s) => {
                let filtered: String;
                let text = if s.contains('\0') {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "unexpected null character in select",
                    )))?;
                    filtered = s.replace('\0', "");
                    if filtered.is_empty() {
                        return Ok(Acknowledgement::no());
                    }
                    &filtered
                } else {
                    s
                };
                self.insert_characters(text)?;
            }
            // A comment token
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            // A DOCTYPE token
            HtmlToken::DocType(_) => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected DOCTYPE in select",
                )))?;
            }
            // A start tag whose tag name is "html"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                return self
                    .in_body_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // A start tag whose tag name is "option"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "option" => {
                // If the current node is an option element, pop that node
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "option" {
                        self.open_elements.pop();
                    }
                }
                self.insert_an_html_element(token)?;
            }
            // A start tag whose tag name is "optgroup"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if token.tag_name == "optgroup" =>
            {
                // If the current node is an option element, pop that node
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "option" {
                        self.open_elements.pop();
                    }
                }
                // If the current node is an optgroup element, pop that node
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "optgroup" {
                        self.open_elements.pop();
                    }
                }
                self.insert_an_html_element(token)?;
            }
            // A start tag whose tag name is "hr"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "hr" => {
                // If the current node is an option element, pop that node
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "option" {
                        self.open_elements.pop();
                    }
                }
                // If the current node is an optgroup element, pop that node
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "optgroup" {
                        self.open_elements.pop();
                    }
                }
                self.insert_an_html_element(token)?;
                // Pop the current node off the stack of open elements
                self.open_elements.pop();
                // Acknowledge the token's self-closing flag, if it is set
                return Ok(Acknowledgement::yes());
            }
            // An end tag whose tag name is "optgroup"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "optgroup" => {
                // First, if the current node is an option element, and the node immediately
                // before it in the stack of open elements is an optgroup element, then pop
                // the current node from the stack of open elements.
                let len = self.open_elements.len();
                if len >= 2 {
                    let is_current_option = self
                        .current_node_as_element()
                        .map(|el| el.name == "option")
                        .unwrap_or(false);
                    if is_current_option {
                        let prev_id = self.open_elements[len - 2];
                        let is_prev_optgroup = self
                            .arena
                            .get(prev_id)
                            .and_then(|node| match node.get() {
                                XpathItemTreeNode::ElementNode(el) => Some(el.name == "optgroup"),
                                _ => None,
                            })
                            .unwrap_or(false);
                        if is_prev_optgroup {
                            self.open_elements.pop();
                        }
                    }
                }

                // If the current node is an optgroup element, pop that node
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "optgroup" {
                        self.open_elements.pop();
                    } else {
                        // Otherwise, this is a parse error; ignore the token.
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "unexpected </optgroup> in select",
                        )))?;
                    }
                }
            }
            // An end tag whose tag name is "option"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "option" => {
                if let Some(el) = self.current_node_as_element() {
                    if el.name == "option" {
                        self.open_elements.pop();
                    } else {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "unexpected </option> in select",
                        )))?;
                    }
                }
            }
            // An end tag whose tag name is "select"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "select" => {
                if !self.has_an_element_in_select_scope("select") {
                    // Parse error. Ignore the token. (fragment case)
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "no select element in select scope",
                    )))?;
                } else {
                    self.pop_until_tag_name("select")?;
                    self.reset_the_insertion_mode_appropriately()?;
                }
            }
            // A start tag whose tag name is "select"
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "select" => {
                // Parse error.
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected <select> in select",
                )))?;
                if !self.has_an_element_in_select_scope("select") {
                    // Ignore the token. (fragment case)
                } else {
                    // Act as if </select> was seen
                    self.pop_until_tag_name("select")?;
                    self.reset_the_insertion_mode_appropriately()?;
                }
            }
            // A start tag whose tag name is one of: "input", "keygen", "textarea"
            HtmlToken::TagToken(TagTokenType::StartTag(ref start_token))
                if ["input", "keygen", "textarea"].contains(&start_token.tag_name.as_str()) =>
            {
                // Parse error.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected <{}> in select",
                    start_token.tag_name
                )))?;
                if !self.has_an_element_in_select_scope("select") {
                    // Ignore the token. (fragment case)
                } else {
                    self.pop_until_tag_name("select")?;
                    self.reset_the_insertion_mode_appropriately()?;
                    // Reprocess the token.
                    self.token_emitted(token)?;
                }
            }
            // A start tag whose tag name is one of: "script", "template"
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["script", "template"].contains(&token.tag_name.as_str()) =>
            {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::StartTag(token)));
            }
            // An end tag whose tag name is "template"
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                return self
                    .in_head_insertion_mode(HtmlToken::TagToken(TagTokenType::EndTag(token)));
            }
            // An end-of-file token
            HtmlToken::EndOfFile => {
                return self.in_body_insertion_mode(HtmlToken::EndOfFile);
            }
            // Anything else: parse error, ignore the token.
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token in select",
                )))?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inselectintable>
    pub(super) fn in_select_in_table_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A start tag whose tag name is one of: "caption", "table", "tbody", "tfoot",
            // "thead", "tr", "td", "th"
            HtmlToken::TagToken(TagTokenType::StartTag(ref start_token))
                if [
                    "caption", "table", "tbody", "tfoot", "thead", "tr", "td", "th",
                ]
                .contains(&start_token.tag_name.as_str()) =>
            {
                // Parse error.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected <{}> in select in table",
                    start_token.tag_name
                )))?;
                // Pop elements until a select element has been popped
                self.pop_until_tag_name("select")?;
                self.reset_the_insertion_mode_appropriately()?;
                // Reprocess the token.
                self.token_emitted(token)?;
            }
            // An end tag whose tag name is one of: "caption", "table", "tbody", "tfoot",
            // "thead", "tr", "td", "th"
            HtmlToken::TagToken(TagTokenType::EndTag(ref end_token))
                if [
                    "caption", "table", "tbody", "tfoot", "thead", "tr", "td", "th",
                ]
                .contains(&end_token.tag_name.as_str()) =>
            {
                // Parse error.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected </{}> in select in table",
                    end_token.tag_name
                )))?;
                if !self.has_an_element_in_table_scope(&end_token.tag_name) {
                    // Ignore the token.
                } else {
                    self.pop_until_tag_name("select")?;
                    self.reset_the_insertion_mode_appropriately()?;
                    // Reprocess the token.
                    self.token_emitted(token)?;
                }
            }
            // Anything else: process using the rules for the "in select" insertion mode
            _ => {
                return self.in_select_insertion_mode(token);
            }
        }

        Ok(Acknowledgement::no())
    }
}
