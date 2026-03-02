//! Rules for parsing tokens in foreign content.
//!
//! <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign>

use super::super::{
    chars,
    tokenizer::{HtmlToken, TagTokenType},
    Acknowledgement, HtmlParseError, HtmlParser, HtmlParserError, HTML_NAMESPACE,
    MATHML_NAMESPACE, SVG_NAMESPACE,
};

/// HTML tag names that cause a breakout from foreign content.
///
/// When one of these tags is encountered as a start tag while parsing in
/// foreign content, the parser pops elements until the adjusted current node
/// is in the HTML namespace, then reprocesses the token using "in body" rules.
///
/// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign>
const FOREIGN_CONTENT_BREAKOUT_TAGS: [&str; 44] = [
    "b",
    "big",
    "blockquote",
    "body",
    "br",
    "center",
    "code",
    "dd",
    "div",
    "dl",
    "dt",
    "em",
    "embed",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "hr",
    "i",
    "img",
    "li",
    "listing",
    "menu",
    "meta",
    "nobr",
    "ol",
    "p",
    "pre",
    "ruby",
    "s",
    "small",
    "span",
    "strong",
    "strike",
    "sub",
    "sup",
    "table",
    "tt",
    "u",
    "ul",
    "var",
    // "font" is handled separately — only breaks out if it has color/face/size attributes.
];

impl HtmlParser {
    /// Process a token using the rules for parsing tokens in foreign content.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign>
    pub(crate) fn in_foreign_content(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            // A character token that is U+0000 NULL
            HtmlToken::Character('\0') => {
                // Parse error. Insert a U+FFFD REPLACEMENT CHARACTER.
                self.handle_error(HtmlParserError::MinorError(
                    "unexpected null character in foreign content".to_string(),
                ))?;
                self.insert_character('\u{FFFD}')?;
            }

            // A character token that is whitespace
            HtmlToken::Character(
                c @ (chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE),
            ) => {
                self.insert_character(c)?;
            }

            // Any other character token
            HtmlToken::Character(c) => {
                self.insert_character(c)?;
                self.frameset_ok = false;
            }

            // Batched characters in foreign content
            HtmlToken::Characters(ref s) => {
                // Replace NULLs with U+FFFD
                let filtered: String;
                let text = if s.contains('\0') {
                    self.handle_error(HtmlParserError::MinorError(
                        "unexpected null character in foreign content".to_string(),
                    ))?;
                    filtered = s.replace('\0', "\u{FFFD}");
                    &filtered
                } else {
                    s
                };
                self.insert_characters(text)?;
                // Set frameset_ok = false if batch contains any non-whitespace.
                if text
                    .bytes()
                    .any(|b| !matches!(b, b'\t' | b'\n' | b'\x0C' | b'\r' | b' '))
                {
                    self.frameset_ok = false;
                }
            }

            // A comment token
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }

            // A DOCTYPE token
            HtmlToken::DocType(_) => {
                // Parse error. Ignore the token.
                self.handle_error(HtmlParserError::MinorError(
                    "unexpected DOCTYPE in foreign content".to_string(),
                ))?;
            }

            // Start tag: one of the breakout tags, or "font" with color/face/size attributes
            HtmlToken::TagToken(TagTokenType::StartTag(ref tag))
                if FOREIGN_CONTENT_BREAKOUT_TAGS.contains(&tag.tag_name.as_str())
                    || (tag.tag_name == "font"
                        && tag
                            .attributes
                            .iter()
                            .any(|a| a.name == "color" || a.name == "face" || a.name == "size")) =>
            {
                // Parse error.
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected HTML start tag <{}> in foreign content",
                    tag.tag_name
                )))?;

                // Pop elements from the stack of open elements until the adjusted
                // current node is an element in the HTML namespace.
                // (For fragment parsing: also stop if the adjusted current node is
                // not in the HTML namespace and is the bottom of the stack.)
                loop {
                    let acn_id = match self.adjusted_current_node_id_opt() {
                        Some(id) => id,
                        None => break,
                    };
                    let ns = self.element_namespace(acn_id).unwrap_or(HTML_NAMESPACE);
                    if ns == HTML_NAMESPACE {
                        break;
                    }
                    // For fragment parsers: if we're down to one element left on the
                    // stack and the context element is set, stop to avoid popping the
                    // root html element.
                    if self.is_fragment_parser() && self.open_elements.len() == 1 {
                        break;
                    }
                    self.open_elements.pop();
                }

                // Process the token using the rules for the current insertion mode in HTML content.
                return self.handle_token(token, self.insertion_mode);
            }

            // Any other start tag
            HtmlToken::TagToken(TagTokenType::StartTag(mut tag)) => {
                let acn_id = self
                    .adjusted_current_node_id_opt()
                    .ok_or(HtmlParseError::new(
                        "no adjusted current node for foreign content start tag",
                    ))?;
                let acn_ns = self
                    .element_namespace(acn_id)
                    .unwrap_or(HTML_NAMESPACE)
                    .to_string();

                // If the adjusted current node is in the MathML namespace,
                // adjust MathML attributes.
                if acn_ns == MATHML_NAMESPACE {
                    Self::adjust_mathml_attributes(&mut tag);
                }

                // If the adjusted current node is in the SVG namespace,
                // adjust SVG tag names and SVG attributes.
                if acn_ns == SVG_NAMESPACE {
                    Self::adjust_svg_tag_names(&mut tag);
                    Self::adjust_svg_attributes(&mut tag);
                }

                // Adjust foreign attributes.
                Self::adjust_foreign_attributes(&mut tag);

                // Insert a foreign element for the token, in the same namespace
                // as the adjusted current node.
                let self_closing = tag.self_closing;
                self.insert_foreign_element(tag, &acn_ns, false)?;

                // If the token has its self-closing flag set:
                if self_closing {
                    // Pop the current node off the stack of open elements.
                    self.open_elements.pop();
                    // Acknowledge the token's self-closing flag.
                    return Ok(Acknowledgement::yes());
                }
            }

            // An end tag whose tag name is "script", if the current node is
            // an SVG script element.
            HtmlToken::TagToken(TagTokenType::EndTag(ref tag))
                if tag.tag_name == "script"
                    && self.current_node_id().map_or(false, |id| {
                        self.element_namespace(id) == Some(SVG_NAMESPACE)
                            && self.element_name(id) == Some("script")
                    }) =>
            {
                // Pop the current node off the stack of open elements.
                self.open_elements.pop();
                // Script execution is not supported; done.
            }

            // Any other end tag
            HtmlToken::TagToken(TagTokenType::EndTag(ref tag)) => {
                let tag_name = tag.tag_name.clone();
                return self.foreign_content_end_tag(&tag_name, token);
            }

            HtmlToken::EndOfFile => {
                // Should not reach here (dispatcher handles EOF), but just in case
                // process via the current insertion mode.
                return self.handle_token(token, self.insertion_mode);
            }
        }

        Ok(Acknowledgement::no())
    }

    /// Handle an end tag in foreign content using the WHATWG algorithm.
    ///
    /// This walks up the stack of open elements to find a matching element,
    /// popping elements along the way. If it reaches an HTML-namespace element
    /// without finding a match, it falls through to the current insertion mode.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign>
    fn foreign_content_end_tag(
        &mut self,
        tag_name: &str,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        // 1. Initialize node to be the current node (the bottommost node of the stack).
        let mut node_index = self.open_elements.len() - 1;

        // 2. If node's tag name, converted to ASCII lowercase, is not the same
        //    as the tag name of the token, then this is a parse error.
        if let Some(name) = self.element_name(self.open_elements[node_index]) {
            if name.to_ascii_lowercase() != tag_name {
                self.handle_error(HtmlParserError::MinorError(format!(
                    "unexpected end tag </{}> in foreign content (current node is <{}>)",
                    tag_name, name
                )))?;
            }
        }

        // 3. Loop:
        loop {
            // If node is the topmost element in the stack of open elements, return.
            // (fragment case)
            if node_index == 0 {
                return Ok(Acknowledgement::no());
            }

            let node_id = self.open_elements[node_index];

            // If node's tag name, converted to ASCII lowercase, is the same as
            // the tag name of the token, pop elements from the stack until node
            // has been popped, and then return.
            if let Some(name) = self.element_name(node_id) {
                if name.to_ascii_lowercase() == tag_name {
                    // Pop until (and including) node_index
                    self.open_elements.truncate(node_index);
                    return Ok(Acknowledgement::no());
                }
            }

            // Set node to the previous entry in the stack of open elements.
            node_index -= 1;
            let node_id = self.open_elements[node_index];

            // If node is not an element in the HTML namespace, return to the
            // step labeled loop.
            let ns = self.element_namespace(node_id).unwrap_or(HTML_NAMESPACE);
            if ns != HTML_NAMESPACE {
                continue;
            }

            // Otherwise, process the token according to the rules given in the
            // section corresponding to the current insertion mode in HTML content.
            return self.handle_token(token, self.insertion_mode);
        }
    }
}
