use crate::xpath::grammar::{
    data_model::{AttributeNode, ElementNode},
    XpathItemTreeNode,
};

use super::{
    chars,
    tokenizer::{HtmlToken, TagToken, TokenizerObserver},
    HtmlParseError, HtmlParser, HtmlParserError, InsertionMode, HTML_NAMESPACE,
};

impl HtmlParser {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode>
    pub(super) fn initial_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<(), HtmlParseError> {
        match token {
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                // ignore
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType => todo!(),
            _ => {
                // TODO: If the document is not an iframe srcdoc document, then this is a parse error;
                //       if the parser cannot change the mode flag is false, set the Document to quirks mode.

                self.insertion_mode = InsertionMode::BeforeHtml;
                self.token_emitted(token)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode>
    pub(super) fn before_html_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<(), HtmlParseError> {
        match token {
            HtmlToken::DocType => todo!(),
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                // ignore
            }
            HtmlToken::StartTag(token) if token.tag_name == "html" => {
                let node_id = self.create_an_element_for_the_token(token, HTML_NAMESPACE, None)?;

                self.open_elements.push(node_id);

                // make it the document root
                self.root_node = Some(node_id);
            }
            HtmlToken::EndTag(TagToken { tag_name, .. })
                if ["head", "body", "html", "br"].contains(&tag_name.as_ref()) =>
            {
                todo!()
            }
            HtmlToken::EndTag(_) => {
                todo!()
            }
            _ => {
                let node_id =
                    self.create_element(String::from("html"), HTML_NAMESPACE, None, None)?;

                self.open_elements.push(node_id);

                // make it the document root
                self.root_node = Some(node_id);

                self.insertion_mode = InsertionMode::BeforeHead;
                self.token_emitted(token)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode>
    pub(super) fn before_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<(), HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            let node_id =
                parser.create_element(String::from("head"), HTML_NAMESPACE, None, None)?;

            parser.open_elements.push(node_id);

            parser.insertion_mode = InsertionMode::InHead;
            parser.token_emitted(token)?;

            Ok(())
        }

        match token {
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                // ignore
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType => todo!(),
            HtmlToken::StartTag(token) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::EndTag(token)
                if ["head", "body", "html", "br"].contains(&token.tag_name.as_ref()) =>
            {
                todo!()
            }
            HtmlToken::EndTag(_) => {
                todo!()
            }
            _ => anything_else(self, token)?,
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead>
    pub(super) fn in_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<(), HtmlParseError> {
        match token {
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                todo!()
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType => todo!(),
            HtmlToken::StartTag(token) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if ["base", "basefont", "bgsound", "link"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "meta" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "title" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if ["noframes", "style"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "noscript" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "script" => {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::EndTag(token)
                if ["body", "html", "br"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::EndTag(_) => {
                todo!()
            }
            _ => {
                self.open_elements.pop().expect("open elements is empty");

                self.insertion_mode = InsertionMode::AfterHead;

                self.token_emitted(token)?;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode>
    pub(super) fn after_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<(), HtmlParseError> {
        match token {
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                todo!()
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType => todo!(),
            HtmlToken::StartTag(token) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "body" => {
                self.insert_an_html_element(token)?;

                self.frameset_ok = false;

                self.insertion_mode = InsertionMode::InBody;
            }
            HtmlToken::StartTag(token) if token.tag_name == "frameset" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::EndTag(token)
                if ["body", "html", "br"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::EndTag(_) => {
                todo!()
            }
            _ => {
                todo!()
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody>
    pub(super) fn in_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<(), HtmlParseError> {
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
                todo!()
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
            HtmlToken::Comment(_) => {
                todo!()
            }
            HtmlToken::DocType => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "html" => {
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
                    return Ok(());
                }

                // for each attribute, check if the attribute is already present on top element of the stack
                let top_element_res = self.top_node().unwrap().as_element_node();

                let top_element = match top_element_res {
                    Ok(node) => node,
                    Err(_) => {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "top element is not an element node",
                        )))?;
                        return Ok(());
                    }
                };

                let top_element_attrs = top_element
                    .attributes_arena(&self.arena)
                    .into_iter()
                    .map(|attr| attr.name.to_string())
                    .collect::<Vec<String>>();

                for (name, value) in token.attributes.into_iter() {
                    // if the element doesn't already have the attribute, add it
                    if !top_element_attrs.contains(&name) {
                        let top_node_id = self.open_elements.first().unwrap();

                        let attr_node_id = self.arena.new_node(XpathItemTreeNode::AttributeNode(
                            AttributeNode::new(name, value),
                        ));
                        top_node_id.append(attr_node_id, &mut self.arena);
                    }
                }
            }
            HtmlToken::StartTag(token)
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "body" => {
                if !self.has_an_element_in_scope("body") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no body element in scope",
                    )))?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                }

                self.insertion_mode = InsertionMode::AfterBody;
            }
            HtmlToken::StartTag(token) if token.tag_name == "frameset" => {
                todo!()
            }
            HtmlToken::EndOfFile => {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "body" => {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::StartTag(token)
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
            HtmlToken::StartTag(token)
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if ["pre", "listing"].contains(&token.tag_name.as_str()) => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "form" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "li" => {
                todo!()
            }
            HtmlToken::StartTag(token) if ["dd", "dt"].contains(&token.tag_name.as_str()) => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "plaintext" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "button" => {
                todo!()
            }
            HtmlToken::EndTag(token)
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
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "form" => {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "p" => {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "li" => {
                todo!()
            }
            HtmlToken::EndTag(token) if ["dd", "dt"].contains(&token.tag_name.as_str()) => {
                todo!()
            }
            HtmlToken::EndTag(token)
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "sarcasm" => {
                // "Take a deep breath, then act as described in the 'any other end tag' entry below." lol
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "a" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if [
                    "b", "big", "code", "em", "font", "i", "s", "small", "strike", "strong", "tt",
                    "u",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "nobr" => {
                todo!()
            }
            HtmlToken::EndTag(token)
                if [
                    "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                    "strong", "tt", "u",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token)
                if ["applet", "marquee", "object"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::EndTag(token)
                if ["applet", "marquee", "object"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "table" => {
                todo!()
            }
            HtmlToken::EndTag(token) if token.tag_name == "br" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if ["area", "br", "embed", "img", "keygen", "wbr"]
                    .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "input" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if ["param", "source", "track"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "hr" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "image" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "textarea" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "xmp" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "iframe" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if ["noembed", "noscript"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "select" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if ["optgroup", "option"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) if ["rb", "rtc"].contains(&token.tag_name.as_str()) => {
                todo!()
            }
            HtmlToken::StartTag(token) if ["rp", "rt"].contains(&token.tag_name.as_str()) => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "math" => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "svg" => {
                todo!()
            }
            HtmlToken::StartTag(token)
                if [
                    "caption", "col", "colgroup", "frame", "head", "tbody", "td", "tfoot", "th",
                    "thead", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::StartTag(token) => {
                todo!()
            }
            HtmlToken::EndTag(token) => {
                todo!()
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody>
    pub(super) fn after_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<(), HtmlParseError> {
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
                self.using_the_rules_for(token, InsertionMode::InBody)?;
            }
            HtmlToken::Comment(_) => {
                todo!()
            }
            HtmlToken::DocType => {
                todo!()
            }
            HtmlToken::StartTag(token) if token.tag_name == "html" => {
                self.using_the_rules_for(HtmlToken::StartTag(token), InsertionMode::InBody)?;
            }
            HtmlToken::EndTag(token) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::EndOfFile => {
                self.stop_parsing()?;
            }
            _ => {
                todo!()
            }
        }

        Ok(())
    }
}
