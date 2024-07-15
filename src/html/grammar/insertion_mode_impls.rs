use crate::xpath::grammar::{data_model::ElementNode, XpathItemTreeNode};

use super::{
    chars,
    tokenizer::{HtmlToken, TagToken, TokenizerObserver},
    HtmlParseError, HtmlParser, InsertionMode, HTML_NAMESPACE,
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
}
