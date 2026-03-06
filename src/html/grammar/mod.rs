//! <https://html.spec.whatwg.org/multipage/parsing.html>

use indextree::{Arena, NodeId};
use thiserror::Error;
use tokenizer::{CommentToken, HtmlToken, Parser, TagToken, TagTokenType, TokenizerState};

use crate::{
    vecpointer::VecPointerRef,
    xpath::{
        grammar::{
            data_model::{
                AttributeNode, CommentNode, ElementNode, TextNode, XpathDocumentNode,
            },
            XpathItemTreeNode,
        },
        XpathItemTree,
    },
};

mod chars;
/// Builder API for programmatically constructing [`XpathItemTree`] documents.
pub mod document_builder;
mod insertion_mode_impls;
mod tokenizer;

/// The document's quirks mode, determined by the DOCTYPE (WHATWG 13.2.6.4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuirksMode {
    /// No quirks mode (standards mode).
    NoQuirks,
    /// Limited quirks mode (almost standards mode).
    LimitedQuirks,
    /// Quirks mode.
    Quirks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum InsertionMode {
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

/// An error that occurred during HTML parsing.
#[derive(Debug, Error)]
#[error("parse error: {message}")]
pub struct HtmlParseError {
    /// A human-readable description of the error.
    pub message: String,
}

impl HtmlParseError {
    /// Create a new parse error with the given message.
    pub fn new(message: &str) -> Self {
        HtmlParseError {
            message: message.to_string(),
        }
    }
}

/// Parse an HTML string into an [`XpathItemTree`].
///
/// This is the main entry point for parsing HTML documents. The returned tree
/// can be queried using XPath expressions via [`crate::xpath::parse`].
///
/// # Example
///
/// ```rust
/// use skyscraper::html;
///
/// let tree = html::parse("<html><body><p>Hello</p></body></html>").unwrap();
/// ```
pub fn parse(text: &str) -> Result<XpathItemTree, HtmlParseError> {
    let mut parser = HtmlParser::new();
    parser.parse(text)
}

/// Parse an HTML fragment as if it were the innerHTML of a context element.
///
/// This implements the HTML fragment parsing algorithm (WHATWG 13.4).
/// The `context_element_name` specifies the tag name of the context element
/// (e.g., `"div"`, `"table"`, `"title"`).
///
/// # Example
/// ```rust
/// use skyscraper::html::{self, grammar::HtmlParseError};
/// # fn main() -> Result<(), HtmlParseError> {
/// let tree = html::parse_fragment("div", "<p>Hello</p>")?;
/// # Ok(())
/// # }
/// ```
pub fn parse_fragment(
    context_element_name: &str,
    text: &str,
) -> Result<XpathItemTree, HtmlParseError> {
    let mut parser = HtmlParser::new();
    parser.parse_fragment(context_element_name, text)
}

/// <https://infra.spec.whatwg.org/#html-namespace>
pub(crate) const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// <https://infra.spec.whatwg.org/#svg-namespace>
pub(crate) const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// <https://infra.spec.whatwg.org/#mathml-namespace>
pub(crate) const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";

pub(crate) static ELEMENT_IN_SCOPE_TYPES: [&str; 18] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
    // MathML scope barriers
    "mi", "mo", "mn", "ms", "mtext", "annotation-xml",
    // SVG scope barriers
    "foreignObject", "desc", "title",
];
pub(crate) static GENERATE_IMPLIED_END_TAG_TYPES: [&str; 10] = [
    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
];

/// <https://html.spec.whatwg.org/multipage/parsing.html#special>
pub(crate) static SPECIAL_ELEMENTS: [&str; 83] = [
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "base",
    "basefont",
    "bgsound",
    "blockquote",
    "body",
    "br",
    "button",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dir",
    "div",
    "dl",
    "dt",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "iframe",
    "img",
    "input",
    "keygen",
    "li",
    "link",
    "listing",
    "main",
    "marquee",
    "menu",
    "meta",
    "nav",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "p",
    "param",
    "plaintext",
    "pre",
    "script",
    "search",
    "section",
    "select",
    "source",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
    "wbr",
    "xmp",
];

/// Returns the correctly-cased SVG element name for a lowercased tag name,
/// or `None` if the element name does not need adjustment.
///
/// The HTML tokenizer lowercases all tag names, but SVG uses camelCase for
/// many element names. This table restores the correct casing.
///
/// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign>
fn svg_element_name(lowered: &str) -> Option<&'static str> {
    match lowered {
        "altglyph" => Some("altGlyph"),
        "altglyphdef" => Some("altGlyphDef"),
        "altglyphitem" => Some("altGlyphItem"),
        "animatecolor" => Some("animateColor"),
        "animatemotion" => Some("animateMotion"),
        "animatetransform" => Some("animateTransform"),
        "clippath" => Some("clipPath"),
        "feblend" => Some("feBlend"),
        "fecolormatrix" => Some("feColorMatrix"),
        "fecomponenttransfer" => Some("feComponentTransfer"),
        "fecomposite" => Some("feComposite"),
        "feconvolvematrix" => Some("feConvolveMatrix"),
        "fediffuselighting" => Some("feDiffuseLighting"),
        "fedisplacementmap" => Some("feDisplacementMap"),
        "fedistantlight" => Some("feDistantLight"),
        "fedropshadow" => Some("feDropShadow"),
        "feflood" => Some("feFlood"),
        "fefunca" => Some("feFuncA"),
        "fefuncb" => Some("feFuncB"),
        "fefuncg" => Some("feFuncG"),
        "fefuncr" => Some("feFuncR"),
        "fegaussianblur" => Some("feGaussianBlur"),
        "feimage" => Some("feImage"),
        "femerge" => Some("feMerge"),
        "femergenode" => Some("feMergeNode"),
        "femorphology" => Some("feMorphology"),
        "feoffset" => Some("feOffset"),
        "fepointlight" => Some("fePointLight"),
        "fespecularlighting" => Some("feSpecularLighting"),
        "fespotlight" => Some("feSpotLight"),
        "fetile" => Some("feTile"),
        "feturbulence" => Some("feTurbulence"),
        "foreignobject" => Some("foreignObject"),
        "glyphref" => Some("glyphRef"),
        "lineargradient" => Some("linearGradient"),
        "radialgradient" => Some("radialGradient"),
        "textpath" => Some("textPath"),
        _ => None,
    }
}

/// Returns the correctly-cased SVG attribute name for a lowercased attribute,
/// or `None` if the attribute does not need adjustment.
///
/// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-svg-attributes>
fn svg_attribute_name(lowered: &str) -> Option<&'static str> {
    match lowered {
        "attributename" => Some("attributeName"),
        "attributetype" => Some("attributeType"),
        "basefrequency" => Some("baseFrequency"),
        "baseprofile" => Some("baseProfile"),
        "calcmode" => Some("calcMode"),
        "clippathunits" => Some("clipPathUnits"),
        "diffuseconstant" => Some("diffuseConstant"),
        "edgemode" => Some("edgeMode"),
        "filterunits" => Some("filterUnits"),
        "glyphref" => Some("glyphRef"),
        "gradienttransform" => Some("gradientTransform"),
        "gradientunits" => Some("gradientUnits"),
        "kernelmatrix" => Some("kernelMatrix"),
        "kernelunitlength" => Some("kernelUnitLength"),
        "keypoints" => Some("keyPoints"),
        "keysplines" => Some("keySplines"),
        "keytimes" => Some("keyTimes"),
        "lengthadjust" => Some("lengthAdjust"),
        "limitingconeangle" => Some("limitingConeAngle"),
        "markerheight" => Some("markerHeight"),
        "markerunits" => Some("markerUnits"),
        "markerwidth" => Some("markerWidth"),
        "maskcontentunits" => Some("maskContentUnits"),
        "maskunits" => Some("maskUnits"),
        "numoctaves" => Some("numOctaves"),
        "pathlength" => Some("pathLength"),
        "patterncontentunits" => Some("patternContentUnits"),
        "patterntransform" => Some("patternTransform"),
        "patternunits" => Some("patternUnits"),
        "pointsatx" => Some("pointsAtX"),
        "pointsaty" => Some("pointsAtY"),
        "pointsatz" => Some("pointsAtZ"),
        "preservealpha" => Some("preserveAlpha"),
        "preserveaspectratio" => Some("preserveAspectRatio"),
        "primitiveunits" => Some("primitiveUnits"),
        "refx" => Some("refX"),
        "refy" => Some("refY"),
        "repeatcount" => Some("repeatCount"),
        "repeatdur" => Some("repeatDur"),
        "requiredextensions" => Some("requiredExtensions"),
        "requiredfeatures" => Some("requiredFeatures"),
        "specularconstant" => Some("specularConstant"),
        "specularexponent" => Some("specularExponent"),
        "spreadmethod" => Some("spreadMethod"),
        "startoffset" => Some("startOffset"),
        "stddeviation" => Some("stdDeviation"),
        "stitchtiles" => Some("stitchTiles"),
        "surfacescale" => Some("surfaceScale"),
        "systemlanguage" => Some("systemLanguage"),
        "tablevalues" => Some("tableValues"),
        "targetx" => Some("targetX"),
        "targety" => Some("targetY"),
        "textlength" => Some("textLength"),
        "viewbox" => Some("viewBox"),
        "viewtarget" => Some("viewTarget"),
        "xchannelselector" => Some("xChannelSelector"),
        "ychannelselector" => Some("yChannelSelector"),
        "zoomandpan" => Some("zoomAndPan"),
        _ => None,
    }
}

/// The XLink namespace URI.
const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";

/// The XML namespace URI.
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

/// The XMLNS namespace URI.
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

/// Returns the namespace URI for a foreign attribute, if any.
///
/// Per WHATWG 13.2.6.3, certain attributes in foreign content (SVG, MathML)
/// are assigned to the xlink, xml, or xmlns namespaces.
///
/// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes>
fn foreign_attribute_namespace(name: &str) -> Option<&'static str> {
    match name {
        "xlink:actuate" | "xlink:arcrole" | "xlink:href" | "xlink:role" | "xlink:show"
        | "xlink:title" | "xlink:type" => Some(XLINK_NAMESPACE),
        "xml:lang" | "xml:space" => Some(XML_NAMESPACE),
        "xmlns" | "xmlns:xlink" => Some(XMLNS_NAMESPACE),
        _ => None,
    }
}

/// Represents a position in the tree where a new node should be inserted.
///
/// This is used by the "appropriate place for inserting a node" algorithm
/// to communicate the exact insertion point, which can be either appending
/// as the last child of a parent node or inserting before a specific sibling.
#[derive(Debug, Clone, Copy)]
pub(crate) enum InsertionPosition {
    /// Insert as the last child of the given node.
    LastChildOf(NodeId),
    /// Insert before the given sibling node (used in foster parenting).
    BeforeSibling(NodeId),
}

impl InsertionPosition {
    /// Insert a node at this insertion position.
    pub(crate) fn insert(self, new_node: NodeId, arena: &mut Arena<XpathItemTreeNode>) {
        match self {
            InsertionPosition::LastChildOf(parent) => {
                parent.append(new_node, arena);
            }
            InsertionPosition::BeforeSibling(sibling) => {
                sibling.insert_before(new_node, arena);
            }
        }
    }

    /// Get the parent node of this insertion position.
    pub(crate) fn parent(self, arena: &Arena<XpathItemTreeNode>) -> Option<NodeId> {
        match self {
            InsertionPosition::LastChildOf(parent) => Some(parent),
            InsertionPosition::BeforeSibling(sibling) => {
                arena.get(sibling).and_then(|node| node.parent())
            }
        }
    }

    /// Get the node that would be the previous sibling of a newly inserted node.
    ///
    /// For `LastChildOf`, this is the current last child of the parent.
    /// For `BeforeSibling`, this is the node before the sibling.
    pub(crate) fn previous_sibling(self, arena: &Arena<XpathItemTreeNode>) -> Option<NodeId> {
        match self {
            InsertionPosition::LastChildOf(parent) => {
                arena.get(parent).and_then(|node| node.last_child())
            }
            InsertionPosition::BeforeSibling(sibling) => {
                arena.get(sibling).and_then(|node| node.previous_sibling())
            }
        }
    }
}

pub(crate) struct CreateAnElementForTheTokenResult {
    element: ElementNode,
    attributes: Vec<AttributeNode>,
}

#[derive(Debug, Clone)]
pub(crate) enum NodeOrMarker {
    Node(NodeEntry),
    Marker,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeEntry {
    pub(crate) node_id: NodeId,
    pub(crate) token: TagToken,
}

/// A stateful HTML parser implementing the WHATWG parsing algorithm.
///
/// For most use cases, prefer the [`parse`] free function which creates a parser
/// internally. Use `HtmlParser` directly only if you need to configure the
/// error handler or reuse the parser across multiple inputs.
pub struct HtmlParser {
    error_handler: Box<dyn ParseErrorHandler>,
    insertion_mode: InsertionMode,
    template_insertion_modes: Vec<InsertionMode>,
    original_insertion_mode: Option<InsertionMode>,
    open_elements: Vec<NodeId>,
    context_element: Option<NodeId>,
    arena: Arena<XpathItemTreeNode>,
    root_node: Option<NodeId>,
    foster_parenting: bool,
    frameset_ok: bool,
    active_formatting_elements: Vec<NodeOrMarker>,
    head_element_pointer: Option<NodeId>,
    form_element_pointer: Option<NodeId>,
    pending_table_character_tokens: Vec<HtmlToken>,
    skip_next_line_feed: bool,
    quirks_mode: QuirksMode,
    /// Cached text node for consecutive character insertions.
    /// When characters are being appended to a text node, this caches the NodeId
    /// and the tree_generation at the time. Subsequent characters can skip the
    /// full `insert_character` path if the generation hasn't changed.
    active_text_node: Option<(NodeId, u32)>,
    /// Monotonically increasing counter, bumped on every non-character token.
    /// Used to invalidate the active_text_node cache.
    tree_generation: u32,
}

impl HtmlParser {
    /// Create a new parser with default settings.
    pub fn new() -> Self {
        HtmlParser {
            error_handler: Box::new(DefaultParseErrorHandler),
            insertion_mode: InsertionMode::Initial,
            template_insertion_modes: Vec::new(),
            original_insertion_mode: None,
            open_elements: Vec::new(),
            context_element: None,
            arena: Arena::new(),
            root_node: None,
            foster_parenting: false,
            frameset_ok: true,
            active_formatting_elements: Vec::new(),
            head_element_pointer: None,
            form_element_pointer: None,
            pending_table_character_tokens: Vec::new(),
            skip_next_line_feed: false,
            quirks_mode: QuirksMode::NoQuirks,
            active_text_node: None,
            tree_generation: 0,
        }
    }

    /// Parse an HTML string into an [`XpathItemTree`].
    pub fn parse(&mut self, text: &str) -> Result<XpathItemTree, HtmlParseError> {
        // set document node as the root node
        let document_node_id = self
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));

        self.root_node = Some(document_node_id);

        let chars: Vec<char> = text.chars().collect();
        let input_stream = VecPointerRef::new(&chars);
        let mut tokenizer = tokenizer::Tokenizer::new(input_stream, self);
        let tokenizer_error_handler = tokenizer::DefaultTokenizerErrorHandler;

        tokenizer.set_error_handler(&tokenizer_error_handler);

        while !tokenizer.is_terminated() {
            tokenizer.step()?;
        }

        let arena = std::mem::replace(&mut self.arena, Arena::new());
        let document = XpathItemTree::new_with_quirks_mode(arena, document_node_id, self.quirks_mode);
        Ok(document)
    }

    /// Parse an HTML fragment using the HTML fragment parsing algorithm.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-html-fragments>
    pub fn parse_fragment(
        &mut self,
        context_element_name: &str,
        text: &str,
    ) -> Result<XpathItemTree, HtmlParseError> {
        // 1. Create a new Document node.
        let document_node_id = self
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        self.root_node = Some(document_node_id);

        // 2. Create the context element as a detached node in the arena
        //    (used by adjusted_current_node when open_elements has only one entry).
        let context_element = ElementNode::new(context_element_name.to_string());
        let context_element_id =
            self.new_node(XpathItemTreeNode::ElementNode(context_element));
        self.context_element = Some(context_element_id);

        // WHATWG 13.4 step 8: set form element pointer if context is a form.
        if context_element_name == "form" {
            self.form_element_pointer = Some(context_element_id);
        }

        // 3. Create an html element, append to document, push onto open_elements.
        let html_element = ElementNode::new("html".to_string());
        let html_id = self.new_node(XpathItemTreeNode::ElementNode(html_element));
        document_node_id.append(html_id, &mut self.arena);
        self.open_elements.push(html_id);

        // 4. If context is "template", push InTemplate onto template_insertion_modes.
        if context_element_name == "template" {
            self.template_insertion_modes.push(InsertionMode::InTemplate);
        }

        // 5. Reset the insertion mode appropriately.
        self.reset_the_insertion_mode_appropriately()?;

        // 6. Determine initial tokenizer state based on context element name.
        // Note: "noscript" would be RAWTEXT if scripting were enabled, but
        // Skyscraper does not support scripting, so it falls through to Data.
        let initial_state = match context_element_name {
            "title" | "textarea" => TokenizerState::RCDATA,
            "style" | "xmp" | "iframe" | "noembed" | "noframes" => TokenizerState::RAWTEXT,
            "script" => TokenizerState::ScriptData,
            "plaintext" => TokenizerState::PLAINTEXT,
            _ => TokenizerState::Data,
        };

        // 7. Create tokenizer, set initial state, and run.
        let chars: Vec<char> = text.chars().collect();
        let input_stream = VecPointerRef::new(&chars);
        let mut tokenizer = tokenizer::Tokenizer::new(input_stream, self);
        let tokenizer_error_handler = tokenizer::DefaultTokenizerErrorHandler;
        tokenizer.set_error_handler(&tokenizer_error_handler);
        tokenizer.set_state(initial_state);

        while !tokenizer.is_terminated() {
            tokenizer.step()?;
        }

        let arena = std::mem::replace(&mut self.arena, Arena::new());
        let document = XpathItemTree::new_with_quirks_mode(arena, document_node_id, self.quirks_mode);
        Ok(document)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-node>
    pub(crate) fn current_node(&self) -> Option<&XpathItemTreeNode> {
        self.open_elements
            .last()
            .and_then(|id| self.arena.get(*id).map(|node| node.get()))
    }

    pub(crate) fn current_node_id(&self) -> Option<NodeId> {
        self.open_elements.last().map(|id| *id)
    }

    pub(crate) fn current_node_id_result(&self) -> Result<NodeId, HtmlParseError> {
        self.current_node_id()
            .ok_or(HtmlParseError::new("no current node"))
    }

    pub(crate) fn current_node_as_element(&self) -> Option<&ElementNode> {
        self.current_node().and_then(|node| match node {
            XpathItemTreeNode::ElementNode(element) => Some(element),
            _ => None,
        })
    }

    pub(crate) fn current_node_as_element_result(&self) -> Result<&ElementNode, HtmlParseError> {
        self.current_node_as_element()
            .ok_or(HtmlParseError::new("current node is not an element"))
    }

    pub(crate) fn is_fragment_parser(&self) -> bool {
        self.context_element.is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-template-insertion-mode>
    pub(crate) fn current_template_insertion_mode(&self) -> Option<InsertionMode> {
        self.template_insertion_modes.last().map(|mode| *mode)
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

    pub(crate) fn new_node(&mut self, node: XpathItemTreeNode) -> NodeId {
        let id = self.arena.new_node(node);

        let node: &mut XpathItemTreeNode = self.arena.get_mut(id).unwrap().get_mut();

        if let XpathItemTreeNode::ElementNode(element) = node {
            element.set_id(id);
        } else if let XpathItemTreeNode::AttributeNode(attribute) = node {
            attribute.set_id(id);
        }

        id
    }

    pub(crate) fn open_elements_as_nodes(&self) -> Vec<&XpathItemTreeNode> {
        self.open_elements
            .iter()
            .map(|id| self.arena.get(*id).unwrap().get())
            .collect()
    }

    pub(crate) fn open_elements_has_element(&self, tag_name: &str) -> bool {
        self.open_elements
            .iter()
            .any(|id| match self.arena.get(*id).unwrap().get() {
                XpathItemTreeNode::ElementNode(element) => element.name == tag_name,
                _ => false,
            })
    }

    pub(crate) fn handle_error(&self, error: HtmlParserError) -> Result<(), HtmlParseError> {
        match error {
            HtmlParserError::MinorError(_err) => {
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

    /// Adjust SVG tag names on a token.
    ///
    /// The HTML tokenizer lowercases all tag names, but SVG uses camelCase for
    /// many element names. This restores the correct casing.
    ///
    /// Called on start tag tokens in the "in foreign content" rules before
    /// element insertion (WHATWG 13.2.6.5). Also applied as a safety net in
    /// `create_an_element_for_the_token` when the namespace is SVG.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign>
    pub(crate) fn adjust_svg_tag_names(token: &mut TagToken) {
        if let Some(corrected) = svg_element_name(&token.tag_name) {
            token.tag_name = String::from(corrected);
        }
    }

    /// Adjust MathML attributes on a token.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-mathml-attributes>
    pub(crate) fn adjust_mathml_attributes(token: &mut TagToken) {
        for attr in &mut token.attributes {
            if attr.name == "definitionurl" {
                attr.name = String::from("definitionURL");
            }
        }
    }

    /// Adjust SVG attributes on a token.
    ///
    /// The HTML tokenizer lowercases all attribute names, but SVG uses camelCase
    /// for many attributes. This restores the correct casing.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-svg-attributes>
    pub(crate) fn adjust_svg_attributes(token: &mut TagToken) {
        for attr in &mut token.attributes {
            if let Some(corrected) = svg_attribute_name(&attr.name) {
                attr.name = String::from(corrected);
            }
        }
    }

    /// Adjust foreign attributes on a token.
    ///
    /// Sets the namespace on attributes like `xlink:href`, `xml:lang`, and
    /// `xmlns` per the WHATWG spec table.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes>
    pub(crate) fn adjust_foreign_attributes(token: &mut TagToken) {
        for attr in &mut token.attributes {
            attr.namespace = foreign_attribute_namespace(&attr.name).map(String::from);
        }
    }

    /// Returns the namespace of the given element node, or `None` if the node
    /// is not an element.
    ///
    /// HTML elements return `HTML_NAMESPACE`, MathML elements return
    /// `MATHML_NAMESPACE`, SVG elements return `SVG_NAMESPACE`.
    pub(crate) fn element_namespace(&self, node_id: NodeId) -> Option<&str> {
        if let Some(node) = self.arena.get(node_id) {
            if let XpathItemTreeNode::ElementNode(element) = node.get() {
                return Some(
                    element
                        .namespace
                        .as_deref()
                        .unwrap_or(HTML_NAMESPACE),
                );
            }
        }
        None
    }

    /// Returns the tag name of the given element node, or `None` if the node
    /// is not an element.
    pub(crate) fn element_name(&self, node_id: NodeId) -> Option<&str> {
        if let Some(node) = self.arena.get(node_id) {
            if let XpathItemTreeNode::ElementNode(element) = node.get() {
                return Some(&element.name);
            }
        }
        None
    }

    /// An element is a MathML text integration point if it is one of the
    /// MathML elements `mi`, `mo`, `mn`, `ms`, or `mtext`.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#mathml-text-integration-point>
    pub(crate) fn is_mathml_text_integration_point(&self, node_id: NodeId) -> bool {
        if let Some(ns) = self.element_namespace(node_id) {
            if ns == MATHML_NAMESPACE {
                if let Some(name) = self.element_name(node_id) {
                    return matches!(name, "mi" | "mo" | "mn" | "ms" | "mtext");
                }
            }
        }
        false
    }

    /// An element is an HTML integration point if it is one of the SVG
    /// elements `foreignObject`, `desc`, or `title`, or a MathML
    /// `annotation-xml` element whose start tag token had an `encoding`
    /// attribute with the value `text/html` or `application/xhtml+xml`
    /// (ASCII case-insensitive).
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#html-integration-point>
    pub(crate) fn is_html_integration_point(&self, node_id: NodeId) -> bool {
        let ns = match self.element_namespace(node_id) {
            Some(ns) => ns,
            None => return false,
        };
        let name = match self.element_name(node_id) {
            Some(name) => name,
            None => return false,
        };

        if ns == SVG_NAMESPACE {
            return matches!(name, "foreignObject" | "desc" | "title");
        }

        if ns == MATHML_NAMESPACE && name == "annotation-xml" {
            // Check for encoding attribute among children
            for child_id in node_id.children(&self.arena) {
                if let Some(child) = self.arena.get(child_id) {
                    if let XpathItemTreeNode::AttributeNode(attr) = child.get() {
                        if attr.name == "encoding" {
                            let val = attr.value.to_ascii_lowercase();
                            if val == "text/html" || val == "application/xhtml+xml" {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// The adjusted current node's ID, or `None` if the stack is empty.
    pub(crate) fn adjusted_current_node_id_opt(&self) -> Option<NodeId> {
        if let Some(context_element) = self.context_element {
            if self.open_elements.len() == 1 {
                return Some(context_element);
            }
        }
        self.current_node_id()
    }

    /// Determines whether the given token should be processed using the rules
    /// for parsing tokens in foreign content rather than the current insertion
    /// mode.
    ///
    /// This implements the tree construction dispatcher check from WHATWG 13.2.6.
    ///
    /// <https://html.spec.whatwg.org/multipage/parsing.html#tree-construction-dispatcher>
    pub(crate) fn should_process_as_foreign_content(&self, token: &HtmlToken) -> bool {
        // If the stack of open elements is empty, use HTML rules.
        if self.open_elements.is_empty() {
            return false;
        }

        let acn_id = match self.adjusted_current_node_id_opt() {
            Some(id) => id,
            None => return false,
        };

        let acn_ns = match self.element_namespace(acn_id) {
            Some(ns) => ns,
            None => return false,
        };

        // If the adjusted current node is in the HTML namespace, use HTML rules.
        if acn_ns == HTML_NAMESPACE {
            return false;
        }

        // If the adjusted current node is a MathML text integration point and
        // the token is a start tag whose tag name is neither "mglyph" nor "malignmark":
        if self.is_mathml_text_integration_point(acn_id) {
            match token {
                HtmlToken::TagToken(TagTokenType::StartTag(tag))
                    if tag.tag_name != "mglyph" && tag.tag_name != "malignmark" =>
                {
                    return false;
                }
                // MathML text integration point + character token: use HTML rules.
                HtmlToken::Character(_) | HtmlToken::Characters(_) => return false,
                _ => {}
            }
        }

        // If the adjusted current node is a MathML annotation-xml element and
        // the token is a start tag whose tag name is "svg":
        if acn_ns == MATHML_NAMESPACE {
            if let Some(name) = self.element_name(acn_id) {
                if name == "annotation-xml" {
                    if let HtmlToken::TagToken(TagTokenType::StartTag(tag)) = token {
                        if tag.tag_name == "svg" {
                            return false;
                        }
                    }
                }
            }
        }

        // If the adjusted current node is an HTML integration point and the
        // token is a start tag or character token:
        if self.is_html_integration_point(acn_id) {
            match token {
                HtmlToken::TagToken(TagTokenType::StartTag(_))
                | HtmlToken::Character(_)
                | HtmlToken::Characters(_) => {
                    return false;
                }
                _ => {}
            }
        }

        // If the token is an end-of-file token, use HTML rules.
        if matches!(token, HtmlToken::EndOfFile) {
            return false;
        }

        // Otherwise, use foreign content rules.
        true
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
            #[cfg(feature = "debug_prints")]
            {
                if let Some(parent_id) = adjusted_insertion_location.parent(&self.arena) {
                    let element = self.arena.get(parent_id).unwrap().get();
                    println!("child of: {:?}", element);
                }
            }
            adjusted_insertion_location.insert(element_id, &mut self.arena);
        }

        Ok(element_id)
    }

    /// Creates an element node and its attributes in the arena without modifying
    /// the stack of open elements. Used by the adoption agency algorithm which
    /// manages the stack position manually.
    pub(crate) fn create_element_node_from_token_result(
        &mut self,
        result: CreateAnElementForTheTokenResult,
    ) -> NodeId {
        #[cfg(feature = "debug_prints")]
        println!("inserting element: {:?}", result.element);
        let element_id = self.new_node(XpathItemTreeNode::ElementNode(result.element));

        for attribute in result.attributes {
            let item_id = self.new_node(XpathItemTreeNode::AttributeNode(attribute));
            element_id.append(item_id, &mut self.arena);
        }

        element_id
    }

    pub(crate) fn insert_create_an_element_for_the_token_result(
        &mut self,
        result: CreateAnElementForTheTokenResult,
    ) -> Result<NodeId, HtmlParseError> {
        let element_id = self.create_element_node_from_token_result(result);
        self.open_elements.push(element_id);
        Ok(element_id)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-element-at-the-adjusted-insertion-location>
    pub(crate) fn insert_element_at_adjusted_insertion_location(
        &mut self,
        element_id: NodeId,
    ) -> Result<(), HtmlParseError> {
        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node(None)?;

        adjusted_insertion_location.insert(element_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    pub(crate) fn insert_a_comment(
        &mut self,
        comment: CommentToken,
        parent_override: Option<NodeId>,
    ) -> Result<(), HtmlParseError> {
        let comment_id = CommentNode::create(comment.data, &mut self.arena);

        let adjusted_insertion_location = if let Some(parent) = parent_override {
            InsertionPosition::LastChildOf(parent)
        } else {
            self.appropriate_place_for_inserting_a_node(None)?
        };

        adjusted_insertion_location.insert(comment_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    pub(crate) fn appropriate_place_for_inserting_a_node(
        &self,
        override_target: Option<NodeId>,
    ) -> Result<InsertionPosition, HtmlParseError> {
        // 1. If there was an override target specified, then let target be the override target.
        //    Otherwise, let target be the current node.
        let target = if let Some(override_target) = override_target {
            override_target
        } else {
            #[cfg(feature = "debug_prints")]
            {
                let open_elements: Vec<&XpathItemTreeNode> = self
                    .open_elements
                    .iter()
                    .map(|id| self.arena.get(*id).unwrap().get())
                    .collect();

                println!("open elements: {:?}", open_elements);
            }

            self.open_elements
                .last()
                .cloned()
                .ok_or(HtmlParseError::new("no current node to insert a node into"))?
        };

        // 2. Determine the adjusted insertion location using the first matching steps.
        let adjusted_insertion_location = if self.foster_parenting
            && self.is_foster_parenting_target(target)
        {
            // If foster parenting is enabled and target is a table, tbody, tfoot, thead, or tr element:
            let last_template = self.get_last_element_by_tag_name("template");
            let last_table = self.get_last_element_by_tag_name("table");

            match (last_template, last_table) {
                // If there is a last template and either there is no last table, or there is one,
                // but last template is lower in the stack (i.e., was inserted more recently):
                (Some((_template_idx, template_id)), None) => {
                    // No last table — template wins.
                    // Adjusted insertion location is inside last template's template contents.
                    InsertionPosition::LastChildOf(template_id)
                }
                (Some((template_idx, template_id)), Some((table_idx, _)))
                    if template_idx > table_idx =>
                {
                    // Last template is lower (more recent) than last table.
                    InsertionPosition::LastChildOf(template_id)
                }
                (_, None) => {
                    // No last template AND no last table.
                    // Fragment case: insert inside the first element in the stack (the html element).
                    let first_element = self.open_elements.first().cloned().ok_or(
                        HtmlParseError::new("foster parenting: stack of open elements is empty"),
                    )?;
                    InsertionPosition::LastChildOf(first_element)
                }
                (_, Some((table_idx, table_id))) => {
                    // There is a last table.
                    if self.arena.get(table_id).and_then(|n| n.parent()).is_some() {
                        // If last table has a parent node:
                        // Insert inside last table's parent, immediately before last table.
                        InsertionPosition::BeforeSibling(table_id)
                    } else {
                        // Let previous element be the element immediately above last table
                        // in the stack of open elements.
                        let previous_element = if table_idx > 0 {
                            self.open_elements[table_idx - 1]
                        } else {
                            return Err(HtmlParseError::new(
                                "foster parenting: no element above last table in stack",
                            ));
                        };
                        InsertionPosition::LastChildOf(previous_element)
                    }
                }
            }
        } else {
            // Otherwise: adjusted insertion location is inside target, after its last child.
            InsertionPosition::LastChildOf(target)
        };

        // 3. WHATWG says to redirect into the template element's "template contents"
        //    (a DocumentFragment). We intentionally keep children directly under the
        //    template element: Skyscraper has no DOM API (no `template.content`), the
        //    inertness guarantees are already satisfied (no script execution), and
        //    direct children are more useful for XPath queries (e.g. `//template/div`).
        //    This matches html5ever's rcdom behavior.

        Ok(adjusted_insertion_location)
    }

    /// Returns true if the node is a table, tbody, tfoot, thead, or tr element.
    ///
    /// These are the element types that trigger foster parenting when they are the
    /// target in the "appropriate place for inserting a node" algorithm.
    /// See: <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    fn is_foster_parenting_target(&self, node_id: NodeId) -> bool {
        if let Some(node) = self.arena.get(node_id) {
            if let XpathItemTreeNode::ElementNode(element) = node.get() {
                return matches!(
                    element.name.as_str(),
                    "table" | "tbody" | "tfoot" | "thead" | "tr"
                );
            }
        }
        false
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
        let mut local_name = token.tag_name;

        // Adjust SVG element names to their correctly-cased forms.
        // The tokenizer lowercases all tag names, but SVG uses camelCase
        // (e.g. "foreignobject" → "foreignObject").
        if namespace == SVG_NAMESPACE {
            if let Some(corrected) = svg_element_name(&local_name) {
                local_name = String::from(corrected);
            }
        }

        let element = self.create_element(local_name, namespace, None, None)?;

        // add the attributes
        let attributes: Vec<AttributeNode> = token
            .attributes
            .into_iter()
            .map(|attribute| AttributeNode::with_prefix(attribute.name, attribute.value, attribute.prefix, attribute.original_name, attribute.namespace))
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
        let mut element = ElementNode::new(local_name);

        // Set namespace for non-HTML elements (MathML, SVG).
        // HTML elements keep namespace as None since it is the default.
        if namespace != HTML_NAMESPACE {
            element.namespace = Some(namespace.to_string());
        }

        Ok(element)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements>
    pub(crate) fn reconstruct_the_active_formatting_elements(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        fn step_4_rewind(
            parser: &mut HtmlParser,
            entry: &NodeEntry,
            entry_index: usize,
        ) -> Result<(), HtmlParseError> {
            let new_entry_index = parser
                .active_formatting_elements
                .iter()
                .position(|e| {
                    if let NodeOrMarker::Node(NodeEntry { node_id, .. }) = e {
                        return *node_id == entry.node_id;
                    } else {
                        return false;
                    }
                })
                .and_then(|i| if i == 0 { None } else { Some(i - 1) });

            // if there are no entries before entry_id in the list of active formatting elements, then jump to step 8 (create)
            match new_entry_index {
                None => return step_8_create(parser, entry, entry_index),
                Some(new_entry_index) => {
                    let new_entry = parser
                        .active_formatting_elements
                        .get(new_entry_index)
                        .expect("could not get new entry")
                        .clone();

                    if let NodeOrMarker::Node(new_entry) = new_entry {
                        // if new_entry is not a marker and is not in the list of open elements, then jump to step 4 (rewind)
                        if !parser.open_elements.contains(&new_entry.node_id) {
                            return step_4_rewind(parser, &new_entry, new_entry_index);
                        }
                    }

                    return step_7_advance(parser, new_entry_index);
                }
            }
        }

        fn step_7_advance(
            parser: &mut HtmlParser,
            entry_index: usize,
        ) -> Result<(), HtmlParseError> {
            let (new_index, new_entry) = parser
                .active_formatting_elements
                .iter()
                .enumerate()
                .skip(entry_index + 1)
                .find_map(|(i, e)| {
                    if let NodeOrMarker::Node(entry) = e {
                        return Some((i, entry));
                    }

                    None
                })
                .map(|(i, e)| (i, e.clone()))
                .expect("could not get new entry");

            return step_8_create(parser, &new_entry, new_index);
        }

        fn step_8_create(
            parser: &mut HtmlParser,
            entry: &NodeEntry,
            index: usize,
        ) -> Result<(), HtmlParseError> {
            let element = parser.insert_an_html_element(entry.token.clone())?;

            // replace the entry
            let new_entry = NodeEntry {
                node_id: element,
                token: entry.token.clone(),
            };

            // replace the entry in the list of active formatting elements
            parser.active_formatting_elements[index] = NodeOrMarker::Node(new_entry.clone());

            // if the entry was not the last entry in the list of active formatting elements, then jump to advance
            if index != parser.active_formatting_elements.len() - 1 {
                return step_7_advance(parser, index);
            }

            return Ok(());
        }

        if self.active_formatting_elements.is_empty() {
            return Ok(());
        }

        // Check early-exit conditions without cloning the entry.
        // The clone is expensive (TagToken contains String + Vec<Attribute>)
        // and the common case is that the entry is already in open_elements.
        let (node_id, needs_reconstruct) = match self.active_formatting_elements.last().unwrap() {
            NodeOrMarker::Marker => return Ok(()),
            NodeOrMarker::Node(entry) => (entry.node_id, !self.open_elements.contains(&entry.node_id)),
        };

        if !needs_reconstruct {
            return Ok(());
        }

        // Only clone when we actually need to reconstruct (rare path).
        let entry = match self.active_formatting_elements.last().unwrap() {
            NodeOrMarker::Node(entry) => entry.clone(),
            _ => unreachable!(),
        };

        step_4_rewind(self, &entry, self.active_formatting_elements.len() - 1)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character>
    pub(crate) fn insert_character(&mut self, c: char) -> Result<(), HtmlParseError> {
        // Fast path: if we have a cached text node and no non-character tokens
        // have been processed since, append directly.
        if let Some((text_id, gen)) = self.active_text_node {
            if self.tree_generation == gen {
                if let Some(node) = self.arena.get_mut(text_id) {
                    if let XpathItemTreeNode::TextNode(ref mut text) = node.get_mut() {
                        text.content.push(c);
                        return Ok(());
                    }
                }
            }
            // Cache was stale, clear it.
            self.active_text_node = None;
        }

        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node(None)?;

        // Check if the parent is a Document node — DOM doesn't allow Document nodes to have Text children.
        if let Some(parent_id) = adjusted_insertion_location.parent(&self.arena) {
            let node = self.arena.get(parent_id).unwrap().get();
            if let XpathItemTreeNode::DocumentNode(_) = node {
                return Ok(());
            }
        }

        // Get the previous sibling at the insertion point to check for text node merging.
        let prev_sibling_id = adjusted_insertion_location.previous_sibling(&self.arena);

        let prev_sibling: Option<&mut XpathItemTreeNode> =
            prev_sibling_id.map(|id| self.arena.get_mut(id).unwrap().get_mut());

        if let Some(&mut XpathItemTreeNode::TextNode(ref mut text)) = prev_sibling {
            // If the previous sibling is a Text node, append the character to that Text node.
            text.content.push(c);
            self.active_text_node = prev_sibling_id.map(|id| (id, self.tree_generation));
        } else {
            // Otherwise, insert a new Text node with the character as its data.
            let string = c.to_string();
            let text = XpathItemTreeNode::TextNode(TextNode::new(string));
            let text_id = self.new_node(text);

            self.arena
                .get_mut(text_id)
                .unwrap()
                .get_mut()
                .as_text_node_mut()
                .unwrap()
                .set_id(text_id);
            self.active_text_node = Some((text_id, self.tree_generation));

            adjusted_insertion_location.insert(text_id, &mut self.arena);
        }

        Ok(())
    }

    /// Bulk insert a string of characters at the appropriate place.
    ///
    /// This is the batched equivalent of calling [`insert_character`] for each
    /// character individually, but avoids repeated insertion-point lookups.
    pub(crate) fn insert_characters(&mut self, s: &str) -> Result<(), HtmlParseError> {
        // Fast path: active_text_node cache
        if let Some((text_id, gen)) = self.active_text_node {
            if self.tree_generation == gen {
                if let Some(node) = self.arena.get_mut(text_id) {
                    if let XpathItemTreeNode::TextNode(ref mut text) = node.get_mut() {
                        text.content.push_str(s);
                        return Ok(());
                    }
                }
            }
            self.active_text_node = None;
        }

        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node(None)?;

        // Check if the parent is a Document node — DOM doesn't allow Document nodes to have Text children.
        if let Some(parent_id) = adjusted_insertion_location.parent(&self.arena) {
            let node = self.arena.get(parent_id).unwrap().get();
            if let XpathItemTreeNode::DocumentNode(_) = node {
                return Ok(());
            }
        }

        // Get the previous sibling at the insertion point to check for text node merging.
        let prev_sibling_id = adjusted_insertion_location.previous_sibling(&self.arena);

        let prev_sibling: Option<&mut XpathItemTreeNode> =
            prev_sibling_id.map(|id| self.arena.get_mut(id).unwrap().get_mut());

        if let Some(&mut XpathItemTreeNode::TextNode(ref mut text)) = prev_sibling {
            text.content.push_str(s);
            self.active_text_node = prev_sibling_id.map(|id| (id, self.tree_generation));
        } else {
            let text = XpathItemTreeNode::TextNode(TextNode::new(String::from(s)));
            let text_id = self.new_node(text);

            self.arena
                .get_mut(text_id)
                .unwrap()
                .get_mut()
                .as_text_node_mut()
                .unwrap()
                .set_id(text_id);
            self.active_text_node = Some((text_id, self.tree_generation));

            adjusted_insertion_location.insert(text_id, &mut self.arena);
        }

        Ok(())
    }

    /// Insert a character as a text node directly on the document root.
    ///
    /// Used in initial/before_html insertion modes where the open elements stack
    /// may be empty and `insert_character` would fail or drop the character.
    pub(crate) fn insert_character_at_document_level(
        &mut self,
        c: char,
    ) -> Result<(), HtmlParseError> {
        let root = self
            .root_node
            .ok_or(HtmlParseError::new("root node is None"))?;

        let prev_sibling_id = self.arena.get(root).unwrap().last_child();

        if let Some(prev_id) = prev_sibling_id {
            let prev = self.arena.get_mut(prev_id).unwrap().get_mut();
            if let XpathItemTreeNode::TextNode(ref mut text) = prev {
                text.content.push(c);
                return Ok(());
            }
        }

        let text = XpathItemTreeNode::TextNode(TextNode::new(c.to_string()));
        let text_id = self.new_node(text);
        self.arena
            .get_mut(text_id)
            .unwrap()
            .get_mut()
            .as_text_node_mut()
            .unwrap()
            .set_id(text_id);
        root.append(text_id, &mut self.arena);

        Ok(())
    }

    /// Insert a character as a child of the given node, coalescing with
    /// an existing trailing text node if possible. Used for preserving
    /// whitespace at specific tree locations for round-trip fidelity.
    pub(crate) fn insert_character_at_node(
        &mut self,
        parent: NodeId,
        c: char,
    ) -> Result<(), HtmlParseError> {
        let prev_child_id = self.arena.get(parent).unwrap().last_child();

        if let Some(prev_id) = prev_child_id {
            let prev = self.arena.get_mut(prev_id).unwrap().get_mut();
            if let XpathItemTreeNode::TextNode(ref mut text) = prev {
                text.content.push(c);
                return Ok(());
            }
        }

        let text = XpathItemTreeNode::TextNode(TextNode::new(c.to_string()));
        let text_id = self.new_node(text);
        self.arena
            .get_mut(text_id)
            .unwrap()
            .get_mut()
            .as_text_node_mut()
            .unwrap()
            .set_id(text_id);
        parent.append(text_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-the-specific-scope>
    pub(crate) fn has_an_element_in_the_specific_scope(
        &self,
        tag_names: &[&str],
        element_types: &[&str],
    ) -> bool {
        for node_id in self.open_elements.iter().rev() {
            if let Some(node) = self.arena.get(*node_id) {
                if let XpathItemTreeNode::ElementNode(element) = node.get() {
                    if tag_names.contains(&element.name.as_str()) {
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
        self.has_an_element_in_the_specific_scope(&[tag_name], &ELEMENT_IN_SCOPE_TYPES)
    }

    pub(crate) fn has_an_element_in_scope_by_tag_names(&self, tag_names: &[&str]) -> bool {
        self.has_an_element_in_the_specific_scope(tag_names, &ELEMENT_IN_SCOPE_TYPES)
    }

    /// Check whether a specific node (by [`NodeId`]) is in scope.
    ///
    /// NodeId-based variant of
    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope>.
    ///
    /// Walks the stack of open elements from the top. Returns `true` if the
    /// target node is found before any scope-barrier element
    /// ([`ELEMENT_IN_SCOPE_TYPES`]).
    pub(crate) fn has_node_in_scope(&self, target: NodeId) -> bool {
        for node_id in self.open_elements.iter().rev() {
            if *node_id == target {
                return true;
            }
            if let Some(node) = self.arena.get(*node_id) {
                if let XpathItemTreeNode::ElementNode(element) = node.get() {
                    if ELEMENT_IN_SCOPE_TYPES.contains(&element.name.as_str()) {
                        return false;
                    }
                }
            }
        }
        false
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope>
    pub(crate) fn has_an_element_in_button_scope(&self, tag_name: &str) -> bool {
        static BUTTON_SCOPE_TYPES: [&str; 19] = [
            "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
            "mi", "mo", "mn", "ms", "mtext", "annotation-xml",
            "foreignObject", "desc", "title",
            "button",
        ];
        self.has_an_element_in_the_specific_scope(&[tag_name], &BUTTON_SCOPE_TYPES)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-list-item-scope>
    pub(crate) fn has_an_element_in_list_item_scope(&self, tag_name: &str) -> bool {
        static LIST_ITEM_SCOPE_TYPES: [&str; 20] = [
            "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
            "mi", "mo", "mn", "ms", "mtext", "annotation-xml",
            "foreignObject", "desc", "title",
            "ol", "ul",
        ];
        self.has_an_element_in_the_specific_scope(&[tag_name], &LIST_ITEM_SCOPE_TYPES)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-table-scope>
    pub(crate) fn has_an_element_in_table_scope(&self, tag_name: &str) -> bool {
        static TABLE_SCOPE_TYPES: [&str; 3] = ["html", "table", "template"];
        self.has_an_element_in_the_specific_scope(&[tag_name], &TABLE_SCOPE_TYPES)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-select-scope>
    ///
    /// The select scope is inverted from other scopes: all elements are barriers
    /// EXCEPT optgroup and option.
    pub(crate) fn has_an_element_in_select_scope(&self, tag_name: &str) -> bool {
        for node_id in self.open_elements.iter().rev() {
            if let Some(node) = self.arena.get(*node_id) {
                if let XpathItemTreeNode::ElementNode(element) = node.get() {
                    if element.name == tag_name {
                        return true;
                    }

                    // Everything except optgroup and option is a barrier
                    if element.name != "optgroup" && element.name != "option" {
                        return false;
                    }
                }
            }
        }

        false
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#clear-the-stack-back-to-a-table-context>
    pub(crate) fn clear_the_stack_back_to_a_table_context(&mut self) {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if matches!(element.name.as_str(), "table" | "template" | "html") {
                    break;
                }
            }
            self.open_elements.pop();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#clear-the-stack-back-to-a-table-body-context>
    pub(crate) fn clear_the_stack_back_to_a_table_body_context(&mut self) {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if matches!(
                    element.name.as_str(),
                    "tbody" | "tfoot" | "thead" | "template" | "html"
                ) {
                    break;
                }
            }
            self.open_elements.pop();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#clear-the-stack-back-to-a-table-row-context>
    pub(crate) fn clear_the_stack_back_to_a_table_row_context(&mut self) {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if matches!(element.name.as_str(), "tr" | "template" | "html") {
                    break;
                }
            }
            self.open_elements.pop();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#close-the-cell>
    pub(crate) fn close_the_cell(&mut self) -> Result<(), HtmlParseError> {
        self.generate_implied_end_tags(None)?;

        if let Some(node) = self.current_node_as_element() {
            if !matches!(node.name.as_str(), "td" | "th") {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "expected td or th as current node when closing cell",
                )))?;
            }
        }

        self.pop_until_tag_name_one_of(vec!["td", "th"])?;
        self.clear_the_list_of_active_formatting_elements_up_to_the_last_marker()?;
        self.insertion_mode = InsertionMode::InRow;

        Ok(())
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
        self.pop_until_tag_name_one_of(vec![tag_name])
    }
    pub(crate) fn pop_until_tag_name_one_of(
        &mut self,
        tag_names: Vec<&str>,
    ) -> Result<(), HtmlParseError> {
        while let Some(node_id) = self.open_elements.pop() {
            let node = self.arena.get(node_id).unwrap().get();
            if let XpathItemTreeNode::ElementNode(element) = node {
                if tag_names.contains(&element.name.as_str()) {
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
        self.handle_token(token, insertion_mode)?;

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generic-rcdata-element-parsing-algorithm>
    pub(crate) fn generic_rcdata_element_parsing_algorithm(
        &mut self,
        token: TagToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        self.insert_an_html_element(token)?;

        self.original_insertion_mode = Some(self.insertion_mode);
        self.insertion_mode = InsertionMode::Text;

        Ok(Acknowledgement {
            self_closed: false,
            tokenizer_state: Some(TokenizerState::RCDATA),
        })
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements>
    pub(crate) fn push_onto_the_list_of_active_formatting_elements(
        &mut self,
        element_id: NodeId,
        token: TagToken,
    ) -> Result<(), HtmlParseError> {
        let element = self
            .arena
            .get(element_id)
            .unwrap()
            .get()
            .as_element_node()
            .map_err(|_| HtmlParseError::new("node is not an element node"))?;

        let elements_since_marker = self.active_formatting_elements.iter().rev().map_while(
            |node_or_marker| match node_or_marker {
                NodeOrMarker::Node(entry) => {
                    let node = self.arena.get(entry.node_id).unwrap().get();
                    match node {
                        XpathItemTreeNode::ElementNode(element) => Some(element),
                        _ => None,
                    }
                }
                _ => None,
            },
        );

        let element_attributes = element.attributes_arena(&self.arena);
        let matching_elements = elements_since_marker
            .filter(|e| {
                if e.name != element.name || e.namespace != element.namespace {
                    return false;
                }

                let e_attributes = e.attributes_arena(&self.arena);
                if e_attributes.len() != element_attributes.len() {
                    return false;
                }

                for attribute in e_attributes.iter() {
                    let has_match = element_attributes.iter().any(|ea| {
                        ea.name == attribute.name && ea.value == attribute.value
                    });
                    if !has_match {
                        return false;
                    }
                }

                true
            })
            .collect::<Vec<&ElementNode>>();

        if matching_elements.len() >= 3 {
            // remove the earliest matching element from the list
            // matching_elements were collected from a .rev() iterator, so the last entry is the earliest
            let earliest_element = matching_elements.last().unwrap();
            let earliest_element_id = earliest_element.id();
            self.active_formatting_elements.retain(|node_or_marker| {
                if let NodeOrMarker::Node(entry) = node_or_marker {
                    return entry.node_id != earliest_element_id;
                }

                true
            });
        }

        self.active_formatting_elements
            .push(NodeOrMarker::Node(NodeEntry {
                node_id: element_id,
                token,
            }));

        Ok(())
    }

    /// Gets active formatting elements between the end of the list and the last marker,
    /// if there is a marker, or the start of the list otherwise.
    pub(crate) fn active_formatting_elements_until_marker(
        &self,
    ) -> impl Iterator<Item = &ElementNode> {
        self.active_formatting_elements
            .iter()
            .rev()
            .map_while(|node_or_marker| {
                if let NodeOrMarker::Node(entry) = node_or_marker {
                    let node = self.arena.get(entry.node_id).unwrap().get();
                    if let XpathItemTreeNode::ElementNode(element) = node {
                        return Some(element);
                    }
                }

                None
            })
    }

    pub(crate) fn remove_from_active_formatting_elements(
        &mut self,
        element_id: NodeId,
    ) -> Result<(), HtmlParseError> {
        self.active_formatting_elements.retain(|node_or_marker| {
            if let NodeOrMarker::Node(entry) = node_or_marker {
                return entry.node_id != element_id;
            }

            true
        });

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#clear-the-list-of-active-formatting-elements-up-to-the-last-marker>
    pub(crate) fn clear_the_list_of_active_formatting_elements_up_to_the_last_marker(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        // WHATWG: Pop entries from the end until a marker is popped (or the list is empty).
        while let Some(entry) = self.active_formatting_elements.pop() {
            if matches!(entry, NodeOrMarker::Marker) {
                break;
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reset-the-insertion-mode-appropriately>
    pub(crate) fn reset_the_insertion_mode_appropriately(&mut self) -> Result<(), HtmlParseError> {
        fn step_3_loop(
            parser: &mut HtmlParser,
            node_id: NodeId,
            last: bool,
        ) -> Result<(), HtmlParseError> {
            let mut last = last;
            let mut node_id = node_id;
            if node_id == parser.open_elements[0] {
                last = true;

                if let Some(context_element) = parser.context_element {
                    node_id = context_element;
                }
            }

            return step_4(parser, node_id, last);
        }

        fn step_4(
            parser: &mut HtmlParser,
            node_id: NodeId,
            last: bool,
        ) -> Result<(), HtmlParseError> {
            fn step_4_3_loop(
                parser: &mut HtmlParser,
                ancestor_id: NodeId,
                last: bool,
            ) -> Result<(), HtmlParseError> {
                if ancestor_id == parser.open_elements[0] {
                    return step_4_8_done(parser);
                }

                // let ancestor be the node before ancestor in the stack of open elements
                let ancestor_position = parser
                    .open_elements
                    .iter()
                    .position(|id| *id == ancestor_id)
                    .ok_or(HtmlParseError::new(
                        "ancestor id not found in open elements",
                    ))?;
                let ancestor_id =
                    parser
                        .open_elements
                        .get(ancestor_position - 1)
                        .ok_or(HtmlParseError::new(
                            "ancestor id not found in open elements",
                        ))?;

                let ancestor = parser
                    .arena
                    .get(*ancestor_id)
                    .unwrap()
                    .get()
                    .as_element_node()
                    .map_err(|_| HtmlParseError::new("ancestor is not an element node"))?;

                if ancestor.name == "template" {
                    return step_4_8_done(parser);
                }

                if ancestor.name == "table" {
                    parser.insertion_mode = InsertionMode::InSelectInTable;
                    return Ok(());
                }

                return step_4_3_loop(parser, *ancestor_id, last);
            }

            fn step_4_8_done(parser: &mut HtmlParser) -> Result<(), HtmlParseError> {
                parser.insertion_mode = InsertionMode::InSelect;
                Ok(())
            }
            let node = parser
                .arena
                .get(node_id)
                .unwrap()
                .get()
                .as_element_node()
                .map_err(|_| HtmlParseError::new("node is not an element node"))?;

            if node.name == "select" {
                if last {
                    return step_4_8_done(parser);
                }
                return step_4_3_loop(parser, node_id, last);
            }

            if (node.name == "td" || node.name == "th") && !last {
                parser.insertion_mode = InsertionMode::InCell;
                return Ok(());
            }

            if node.name == "tr" {
                parser.insertion_mode = InsertionMode::InRow;
                return Ok(());
            }

            if node.name == "tbody" || node.name == "thead" || node.name == "tfoot" {
                parser.insertion_mode = InsertionMode::InTableBody;
                return Ok(());
            }

            if node.name == "caption" {
                parser.insertion_mode = InsertionMode::InCaption;
                return Ok(());
            }

            if node.name == "colgroup" {
                parser.insertion_mode = InsertionMode::InColumnGroup;
                return Ok(());
            }

            if node.name == "table" {
                parser.insertion_mode = InsertionMode::InTable;
                return Ok(());
            }

            if node.name == "template" {
                parser.insertion_mode = parser
                    .current_template_insertion_mode()
                    .ok_or(HtmlParseError::new("no current template insertion mode"))?;
                return Ok(());
            }

            if node.name == "head" && !last {
                parser.insertion_mode = InsertionMode::InHead;
                return Ok(());
            }

            if node.name == "body" {
                parser.insertion_mode = InsertionMode::InBody;
                return Ok(());
            }

            if node.name == "frameset" {
                parser.insertion_mode = InsertionMode::InFrameset;
                return Ok(());
            }

            if node.name == "html" {
                if parser.head_element_pointer.is_none() {
                    parser.insertion_mode = InsertionMode::BeforeHead;
                } else {
                    parser.insertion_mode = InsertionMode::AfterHead;
                }
                return Ok(());
            }

            if last {
                parser.insertion_mode = InsertionMode::InBody;
                return Ok(());
            }

            // let node be the node before node in the stack of open elements
            let node_position = parser
                .open_elements
                .iter()
                .position(|id| *id == node_id)
                .ok_or(HtmlParseError::new(
                    "ancestor id not found in open elements",
                ))?;
            let node_id =
                parser
                    .open_elements
                    .get(node_position - 1)
                    .ok_or(HtmlParseError::new(
                        "ancestor id not found in open elements",
                    ))?;

            return step_3_loop(parser, *node_id, last);
        }

        let last = false;
        let node_id = self.current_node_id_result()?;

        step_3_loop(self, node_id, last)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generic-raw-text-element-parsing-algorithm>
    pub(crate) fn generic_raw_text_element_parsing_algorithm(
        &mut self,
        token: TagToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        self.insert_an_html_element(token)?;

        self.original_insertion_mode = Some(self.insertion_mode);
        self.insertion_mode = InsertionMode::Text;

        Ok(Acknowledgement {
            self_closed: false,
            tokenizer_state: Some(TokenizerState::RAWTEXT),
        })
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stop-parsing>
    pub(crate) fn stop_parsing(&mut self) -> Result<(), HtmlParseError> {
        // mostly scripting stuff that is unsupported by Skyscraper
        Ok(())
    }

    fn adjusted_current_node_id(&self) -> Result<NodeId, HtmlParseError> {
        if let Some(context_element) = self.context_element {
            if self.open_elements.len() == 1 {
                return Ok(context_element);
            }
        }

        self.current_node_id_result()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generate-all-implied-end-tags-thoroughly>
    pub(crate) fn generate_all_implied_end_tags_thoroughly(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if ![
                    "caption", "colgroup", "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp",
                    "rt", "rtc", "tbody", "td", "tfoot", "th", "thead", "tr",
                ]
                .contains(&element.name.as_str())
                {
                    break;
                }
            }

            self.open_elements.pop();
        }

        Ok(())
    }

    pub(crate) fn handle_token(
        &mut self,
        token: HtmlToken,
        insertion_mode: InsertionMode,
    ) -> Result<Acknowledgement, HtmlParseError> {
        let self_closing = match &token {
            HtmlToken::TagToken(tag) => tag.self_closing(),
            _ => false,
        };

        #[cfg(feature = "debug_prints")]
        {
            println!(
                "insertion mode: {:?}; token: {:?}",
                self.insertion_mode, token
            );
            if let HtmlToken::TagToken(TagTokenType::StartTag(token)) = &token {
                println!("start tag: {}", token.tag_name);
            }

            if let HtmlToken::TagToken(TagTokenType::EndTag(token)) = &token {
                println!("end tag: {}", token.tag_name);
            }
        }

        let acknowledgement = match insertion_mode {
            InsertionMode::Initial => self.initial_insertion_mode(token),
            InsertionMode::BeforeHtml => self.before_html_insertion_mode(token),
            InsertionMode::BeforeHead => self.before_head_insertion_mode(token),
            InsertionMode::InHead => self.in_head_insertion_mode(token),
            InsertionMode::InHeadNoscript => self.in_head_noscript_insertion_mode(token),
            InsertionMode::AfterHead => self.after_head_insertion_mode(token),
            InsertionMode::InBody => self.in_body_insertion_mode(token),
            InsertionMode::Text => self.text_insertion_mode(token),
            InsertionMode::InTable => self.in_table_insertion_mode(token),
            InsertionMode::InTableText => self.in_table_text_insertion_mode(token),
            InsertionMode::InCaption => self.in_caption_insertion_mode(token),
            InsertionMode::InColumnGroup => self.in_column_group_insertion_mode(token),
            InsertionMode::InTableBody => self.in_table_body_insertion_mode(token),
            InsertionMode::InRow => self.in_row_insertion_mode(token),
            InsertionMode::InCell => self.in_cell_insertion_mode(token),
            InsertionMode::InSelect => self.in_select_insertion_mode(token),
            InsertionMode::InSelectInTable => self.in_select_in_table_insertion_mode(token),
            InsertionMode::InTemplate => self.in_template_insertion_mode(token),
            InsertionMode::AfterBody => self.after_body_insertion_mode(token),
            InsertionMode::InFrameset => self.in_frameset_insertion_mode(token),
            InsertionMode::AfterFrameset => self.after_frameset_insertion_mode(token),
            InsertionMode::AfterAfterBody => self.after_after_body_insertion_mode(token),
            InsertionMode::AfterAfterFrameset => self.after_after_frameset_insertion_mode(token),
        }?;

        if self_closing && !acknowledgement.self_closed {
            self.error_handler
                .error_emitted(HtmlParseErrorType::NonVoidHtmlElementStartTagWithTrailingSolidus)?;
        }

        Ok(acknowledgement)
    }
}

/// Errors produced by the HTML parser during tree construction.
#[derive(Debug, Error)]
pub enum HtmlParserError {
    /// A non-fatal parse error (e.g. a mismatched end tag that can be recovered from).
    #[error("minor error: {0}")]
    MinorError(String),
    /// A fatal parse error that prevents the document from being constructed.
    #[error("fatal error: {0}")]
    FatalError(String),
}

pub(crate) struct Acknowledgement {
    pub self_closed: bool,
    pub tokenizer_state: Option<tokenizer::TokenizerState>,
}

impl Acknowledgement {
    fn no() -> Self {
        Acknowledgement {
            self_closed: false,
            tokenizer_state: None,
        }
    }

    fn yes() -> Self {
        Acknowledgement {
            self_closed: true,
            tokenizer_state: None,
        }
    }
}

impl Parser for HtmlParser {
    fn token_emitted(&mut self, token: HtmlToken) -> Result<Acknowledgement, HtmlParseError> {
        // Bump the tree generation counter for any non-character token.
        // This invalidates the active_text_node cache so that structural
        // changes (new elements, end tags, etc.) force insert_character to
        // re-derive the correct insertion point.
        if !matches!(token, HtmlToken::Character(_) | HtmlToken::Characters(_)) {
            self.tree_generation = self.tree_generation.wrapping_add(1);
        }

        // If the skip_next_line_feed flag is set and the next token is a LF character,
        // ignore it (used by <pre>, <listing>, <textarea> per WHATWG spec).
        if self.skip_next_line_feed {
            self.skip_next_line_feed = false;
            if matches!(token, HtmlToken::Character(chars::LINE_FEED)) {
                return Ok(Acknowledgement::no());
            }
            // For a batched Characters token, strip a leading LF if present.
            if let HtmlToken::Characters(ref s) = token {
                if s.starts_with(chars::LINE_FEED) {
                    let rest = &s[1..];
                    if rest.is_empty() {
                        return Ok(Acknowledgement::no());
                    }
                    // Re-emit the remainder as a new token.
                    let remaining = rest.to_string();
                    if remaining.len() == 1 {
                        return self.token_emitted(HtmlToken::Character(
                            remaining.chars().next().unwrap(),
                        ));
                    } else {
                        return self.token_emitted(HtmlToken::Characters(remaining));
                    }
                }
            }
        }

        // Tree construction dispatcher (WHATWG 13.2.6):
        // Process the token using the rules for the current insertion mode in
        // HTML content, UNLESS all of the following conditions are met, in
        // which case process using the rules for parsing tokens in foreign
        // content:
        //   - the stack of open elements is not empty
        //   - the adjusted current node is not in the HTML namespace
        //   - it's not a MathML text integration point receiving a non-mglyph/malignmark start tag or character
        //   - it's not a MathML annotation-xml receiving an "svg" start tag
        //   - it's not an HTML integration point receiving a start tag or character
        //   - it's not an EOF token
        if self.should_process_as_foreign_content(&token) {
            return self.in_foreign_content(token);
        }

        // For insertion modes that don't have native batch-Characters handling,
        // decompose into per-character processing. The hot-path modes (InBody,
        // Text, InTable, InTableText, InTemplate, InSelect) handle Characters
        // directly; all other modes are rare and per-char is both correct and
        // has negligible cost.
        //
        // Optimization: after per-char processing triggers mode transitions
        // (e.g. Initial → BeforeHtml → ... → InBody), re-batch the remaining
        // characters to avoid O(n) individual token_emitted + insert_character
        // calls.
        if let HtmlToken::Characters(ref s) = token {
            if !matches!(
                self.insertion_mode,
                InsertionMode::InBody
                    | InsertionMode::Text
                    | InsertionMode::InTable
                    | InsertionMode::InTableText
                    | InsertionMode::InTemplate
                    | InsertionMode::InSelect
            ) {
                let chars: Vec<char> = s.chars().collect();
                let mut consumed = 0;
                for &c in &chars {
                    self.token_emitted(HtmlToken::Character(c))?;
                    consumed += 1;
                    if matches!(
                        self.insertion_mode,
                        InsertionMode::InBody
                            | InsertionMode::Text
                            | InsertionMode::InTable
                            | InsertionMode::InTableText
                            | InsertionMode::InTemplate
                            | InsertionMode::InSelect
                    ) {
                        break;
                    }
                }
                if consumed < chars.len() {
                    let remaining: String = chars[consumed..].iter().collect();
                    return self.token_emitted(HtmlToken::Characters(remaining));
                }
                return Ok(Acknowledgement::no());
            }
        }

        self.handle_token(token, self.insertion_mode)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjusted-current-node>
    fn adjusted_current_node(&self) -> Option<&XpathItemTreeNode> {
        if let Some(context_element) = self.context_element {
            if self.open_elements.len() == 1 {
                return Some(
                    self.arena
                        .get(context_element)
                        .expect("context element not in arena")
                        .get(),
                );
            }
        }

        self.current_node()
    }
}

/// Handler for parse errors emitted during HTML tokenization and tree construction.
///
/// Implement this trait to customize how parse errors are handled (e.g. to
/// collect warnings instead of failing immediately).
pub trait ParseErrorHandler {
    /// Called when a parse error is encountered.
    ///
    /// Return `Ok(())` to continue parsing, or `Err(...)` to abort.
    fn error_emitted(&self, error: HtmlParseErrorType) -> Result<(), HtmlParseError>;
}

/// The default error handler, which swallows all parse errors and continues parsing.
pub struct DefaultParseErrorHandler;

impl ParseErrorHandler for DefaultParseErrorHandler {
    fn error_emitted(&self, _error: HtmlParseErrorType) -> Result<(), HtmlParseError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xpath::grammar::data_model::ElementNode;

    /// Helper to create a parser with a tree structure suitable for foster parenting tests.
    /// Returns the parser and node IDs for the elements.
    fn setup_parser_with_table() -> (HtmlParser, NodeId, NodeId, NodeId, NodeId) {
        let mut parser = HtmlParser::new();

        // Create root document node
        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        // Create html element
        let html_node = XpathItemTreeNode::ElementNode(ElementNode::new("html".to_string()));
        let html_id = parser.new_node(html_node);
        doc_id.append(html_id, &mut parser.arena);

        // Create body element
        let body_node = XpathItemTreeNode::ElementNode(ElementNode::new("body".to_string()));
        let body_id = parser.new_node(body_node);
        html_id.append(body_id, &mut parser.arena);

        // Create table element
        let table_node = XpathItemTreeNode::ElementNode(ElementNode::new("table".to_string()));
        let table_id = parser.new_node(table_node);
        body_id.append(table_id, &mut parser.arena);

        // Push open elements: html, body, table
        parser.open_elements.push(html_id);
        parser.open_elements.push(body_id);
        parser.open_elements.push(table_id);

        (parser, html_id, body_id, table_id, doc_id)
    }

    #[test]
    fn foster_parenting_disabled_returns_last_child_of_current_node() {
        let (parser, _, _, table_id, _) = setup_parser_with_table();
        // foster_parenting is false by default

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // Should return LastChildOf(table_id) since table is the current node (last on stack)
        match result {
            InsertionPosition::LastChildOf(id) => assert_eq!(id, table_id),
            InsertionPosition::BeforeSibling(_) => {
                panic!("Expected LastChildOf, got BeforeSibling")
            }
        }
    }

    #[test]
    fn foster_parenting_with_table_parent_inserts_before_table() {
        let (mut parser, _, body_id, table_id, _) = setup_parser_with_table();
        parser.foster_parenting = true;

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // Table has a parent (body), so insert before table
        match result {
            InsertionPosition::BeforeSibling(id) => assert_eq!(id, table_id),
            InsertionPosition::LastChildOf(_) => {
                panic!("Expected BeforeSibling, got LastChildOf")
            }
        }

        // Verify that inserting works correctly
        let new_node = XpathItemTreeNode::ElementNode(ElementNode::new("span".to_string()));
        let new_id = parser.new_node(new_node);
        result.insert(new_id, &mut parser.arena);

        // The span should now be before the table, as a child of body
        let body_children: Vec<NodeId> = body_id.children(&parser.arena).collect();
        assert_eq!(body_children.len(), 2);
        assert_eq!(body_children[0], new_id); // span is before table
        assert_eq!(body_children[1], table_id); // table is after span
    }

    #[test]
    fn foster_parenting_table_without_parent_uses_element_above() {
        let mut parser = HtmlParser::new();

        // Create root document node
        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        // Create html element directly in arena (not attached to document)
        let html_node = XpathItemTreeNode::ElementNode(ElementNode::new("html".to_string()));
        let html_id = parser.new_node(html_node);

        // Create table element (also not attached to a parent in the tree)
        let table_node = XpathItemTreeNode::ElementNode(ElementNode::new("table".to_string()));
        let table_id = parser.new_node(table_node);

        // Push open elements: html, table (table has no parent in the tree)
        parser.open_elements.push(html_id);
        parser.open_elements.push(table_id);
        parser.foster_parenting = true;

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // Table has no parent, so use the element above table in the stack (html)
        match result {
            InsertionPosition::LastChildOf(id) => assert_eq!(id, html_id),
            InsertionPosition::BeforeSibling(_) => {
                panic!("Expected LastChildOf, got BeforeSibling")
            }
        }
    }

    #[test]
    fn foster_parenting_with_template_and_no_table() {
        let mut parser = HtmlParser::new();

        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        let html_node = XpathItemTreeNode::ElementNode(ElementNode::new("html".to_string()));
        let html_id = parser.new_node(html_node);
        doc_id.append(html_id, &mut parser.arena);

        let body_node = XpathItemTreeNode::ElementNode(ElementNode::new("body".to_string()));
        let body_id = parser.new_node(body_node);
        html_id.append(body_id, &mut parser.arena);

        let template_node =
            XpathItemTreeNode::ElementNode(ElementNode::new("template".to_string()));
        let template_id = parser.new_node(template_node);
        body_id.append(template_id, &mut parser.arena);

        // Create a tbody inside template (this will be the current node)
        let tbody_node = XpathItemTreeNode::ElementNode(ElementNode::new("tbody".to_string()));
        let tbody_id = parser.new_node(tbody_node);
        template_id.append(tbody_id, &mut parser.arena);

        parser.open_elements.push(html_id);
        parser.open_elements.push(body_id);
        parser.open_elements.push(template_id);
        parser.open_elements.push(tbody_id);
        parser.foster_parenting = true;

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // There's a template but no table, so insert inside template
        match result {
            InsertionPosition::LastChildOf(id) => assert_eq!(id, template_id),
            InsertionPosition::BeforeSibling(_) => {
                panic!("Expected LastChildOf(template), got BeforeSibling")
            }
        }
    }

    #[test]
    fn foster_parenting_template_more_recent_than_table() {
        let mut parser = HtmlParser::new();

        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        let html_node = XpathItemTreeNode::ElementNode(ElementNode::new("html".to_string()));
        let html_id = parser.new_node(html_node);
        doc_id.append(html_id, &mut parser.arena);

        let body_node = XpathItemTreeNode::ElementNode(ElementNode::new("body".to_string()));
        let body_id = parser.new_node(body_node);
        html_id.append(body_id, &mut parser.arena);

        let table_node = XpathItemTreeNode::ElementNode(ElementNode::new("table".to_string()));
        let table_id = parser.new_node(table_node);
        body_id.append(table_id, &mut parser.arena);

        // Template is more recent (pushed after table)
        let template_node =
            XpathItemTreeNode::ElementNode(ElementNode::new("template".to_string()));
        let template_id = parser.new_node(template_node);
        table_id.append(template_id, &mut parser.arena);

        // Create a tr inside template (this is the current node)
        let tr_node = XpathItemTreeNode::ElementNode(ElementNode::new("tr".to_string()));
        let tr_id = parser.new_node(tr_node);
        template_id.append(tr_id, &mut parser.arena);

        parser.open_elements.push(html_id);
        parser.open_elements.push(body_id);
        parser.open_elements.push(table_id);
        parser.open_elements.push(template_id);
        parser.open_elements.push(tr_id);
        parser.foster_parenting = true;

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // Template is lower/more recent than table, so insert inside template
        match result {
            InsertionPosition::LastChildOf(id) => assert_eq!(id, template_id),
            InsertionPosition::BeforeSibling(_) => {
                panic!("Expected LastChildOf(template), got BeforeSibling")
            }
        }
    }

    #[test]
    fn foster_parenting_table_more_recent_than_template() {
        let mut parser = HtmlParser::new();

        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        let html_node = XpathItemTreeNode::ElementNode(ElementNode::new("html".to_string()));
        let html_id = parser.new_node(html_node);
        doc_id.append(html_id, &mut parser.arena);

        let body_node = XpathItemTreeNode::ElementNode(ElementNode::new("body".to_string()));
        let body_id = parser.new_node(body_node);
        html_id.append(body_id, &mut parser.arena);

        // Template pushed first (higher in stack)
        let template_node =
            XpathItemTreeNode::ElementNode(ElementNode::new("template".to_string()));
        let template_id = parser.new_node(template_node);
        body_id.append(template_id, &mut parser.arena);

        // Table pushed second (lower in stack, more recent)
        let table_node = XpathItemTreeNode::ElementNode(ElementNode::new("table".to_string()));
        let table_id = parser.new_node(table_node);
        template_id.append(table_id, &mut parser.arena);

        parser.open_elements.push(html_id);
        parser.open_elements.push(body_id);
        parser.open_elements.push(template_id);
        parser.open_elements.push(table_id);
        parser.foster_parenting = true;

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // Table is more recent than template and has a parent (template),
        // so insert before table
        match result {
            InsertionPosition::BeforeSibling(id) => assert_eq!(id, table_id),
            InsertionPosition::LastChildOf(_) => {
                panic!("Expected BeforeSibling(table), got LastChildOf")
            }
        }
    }

    #[test]
    fn foster_parenting_not_triggered_for_non_table_elements() {
        let mut parser = HtmlParser::new();

        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        let html_node = XpathItemTreeNode::ElementNode(ElementNode::new("html".to_string()));
        let html_id = parser.new_node(html_node);
        doc_id.append(html_id, &mut parser.arena);

        let div_node = XpathItemTreeNode::ElementNode(ElementNode::new("div".to_string()));
        let div_id = parser.new_node(div_node);
        html_id.append(div_id, &mut parser.arena);

        parser.open_elements.push(html_id);
        parser.open_elements.push(div_id);
        parser.foster_parenting = true;

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // div is not a table-scope element, so foster parenting should NOT activate
        match result {
            InsertionPosition::LastChildOf(id) => assert_eq!(id, div_id),
            InsertionPosition::BeforeSibling(_) => {
                panic!("Expected LastChildOf(div), got BeforeSibling")
            }
        }
    }

    #[test]
    fn foster_parenting_no_template_no_table_uses_first_element() {
        let mut parser = HtmlParser::new();

        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        let html_node = XpathItemTreeNode::ElementNode(ElementNode::new("html".to_string()));
        let html_id = parser.new_node(html_node);
        doc_id.append(html_id, &mut parser.arena);

        // Create a tbody directly (simulating a fragment case with no table/template)
        let tbody_node = XpathItemTreeNode::ElementNode(ElementNode::new("tbody".to_string()));
        let tbody_id = parser.new_node(tbody_node);
        html_id.append(tbody_id, &mut parser.arena);

        parser.open_elements.push(html_id);
        parser.open_elements.push(tbody_id);
        parser.foster_parenting = true;

        let result = parser.appropriate_place_for_inserting_a_node(None).unwrap();
        // No template and no table — fragment case: use first element in stack (html)
        match result {
            InsertionPosition::LastChildOf(id) => assert_eq!(id, html_id),
            InsertionPosition::BeforeSibling(_) => {
                panic!("Expected LastChildOf(html), got BeforeSibling")
            }
        }
    }

    #[test]
    fn foster_parenting_with_override_target() {
        let (mut parser, _, body_id, _table_id, _) = setup_parser_with_table();
        parser.foster_parenting = true;

        // Override target is body (not a table-scope element)
        let result = parser
            .appropriate_place_for_inserting_a_node(Some(body_id))
            .unwrap();
        // body is not a table-scope element, so foster parenting should NOT activate
        match result {
            InsertionPosition::LastChildOf(id) => assert_eq!(id, body_id),
            InsertionPosition::BeforeSibling(_) => {
                panic!("Expected LastChildOf(body), got BeforeSibling")
            }
        }
    }

    #[test]
    fn foster_parenting_with_override_target_table_element() {
        let (mut parser, _, _body_id, table_id, _) = setup_parser_with_table();
        parser.foster_parenting = true;

        // Override target is the table element itself
        let result = parser
            .appropriate_place_for_inserting_a_node(Some(table_id))
            .unwrap();
        // table IS a table-scope element AND foster_parenting is true,
        // so foster parenting should activate and insert before table (since it has body as parent)
        match result {
            InsertionPosition::BeforeSibling(id) => assert_eq!(id, table_id),
            InsertionPosition::LastChildOf(_) => {
                panic!("Expected BeforeSibling(table), got LastChildOf")
            }
        }
    }

    #[test]
    fn insertion_position_insert_last_child_of() {
        let mut arena: Arena<XpathItemTreeNode> = Arena::new();
        let parent_node = XpathItemTreeNode::ElementNode(ElementNode::new("div".to_string()));
        let parent_id = arena.new_node(parent_node);

        let child1_node = XpathItemTreeNode::ElementNode(ElementNode::new("p".to_string()));
        let child1_id = arena.new_node(child1_node);
        parent_id.append(child1_id, &mut arena);

        let child2_node = XpathItemTreeNode::ElementNode(ElementNode::new("span".to_string()));
        let child2_id = arena.new_node(child2_node);

        let pos = InsertionPosition::LastChildOf(parent_id);
        pos.insert(child2_id, &mut arena);

        let children: Vec<NodeId> = parent_id.children(&arena).collect();
        assert_eq!(children, vec![child1_id, child2_id]);
    }

    #[test]
    fn insertion_position_insert_before_sibling() {
        let mut arena: Arena<XpathItemTreeNode> = Arena::new();
        let parent_node = XpathItemTreeNode::ElementNode(ElementNode::new("div".to_string()));
        let parent_id = arena.new_node(parent_node);

        let child1_node = XpathItemTreeNode::ElementNode(ElementNode::new("p".to_string()));
        let child1_id = arena.new_node(child1_node);
        parent_id.append(child1_id, &mut arena);

        let child2_node = XpathItemTreeNode::ElementNode(ElementNode::new("span".to_string()));
        let child2_id = arena.new_node(child2_node);

        let pos = InsertionPosition::BeforeSibling(child1_id);
        pos.insert(child2_id, &mut arena);

        let children: Vec<NodeId> = parent_id.children(&arena).collect();
        assert_eq!(children, vec![child2_id, child1_id]);
    }

    #[test]
    fn insertion_position_previous_sibling_last_child_of() {
        let mut arena: Arena<XpathItemTreeNode> = Arena::new();
        let parent_node = XpathItemTreeNode::ElementNode(ElementNode::new("div".to_string()));
        let parent_id = arena.new_node(parent_node);

        let child1_node = XpathItemTreeNode::ElementNode(ElementNode::new("p".to_string()));
        let child1_id = arena.new_node(child1_node);
        parent_id.append(child1_id, &mut arena);

        let pos = InsertionPosition::LastChildOf(parent_id);
        assert_eq!(pos.previous_sibling(&arena), Some(child1_id));
    }

    #[test]
    fn insertion_position_previous_sibling_before_sibling() {
        let mut arena: Arena<XpathItemTreeNode> = Arena::new();
        let parent_node = XpathItemTreeNode::ElementNode(ElementNode::new("div".to_string()));
        let parent_id = arena.new_node(parent_node);

        let child1_node = XpathItemTreeNode::ElementNode(ElementNode::new("p".to_string()));
        let child1_id = arena.new_node(child1_node);
        parent_id.append(child1_id, &mut arena);

        let child2_node = XpathItemTreeNode::ElementNode(ElementNode::new("span".to_string()));
        let child2_id = arena.new_node(child2_node);
        parent_id.append(child2_id, &mut arena);

        let pos = InsertionPosition::BeforeSibling(child2_id);
        assert_eq!(pos.previous_sibling(&arena), Some(child1_id));
    }

    #[test]
    fn insertion_position_parent_last_child_of() {
        let mut arena: Arena<XpathItemTreeNode> = Arena::new();
        let parent_node = XpathItemTreeNode::ElementNode(ElementNode::new("div".to_string()));
        let parent_id = arena.new_node(parent_node);

        let pos = InsertionPosition::LastChildOf(parent_id);
        assert_eq!(pos.parent(&arena), Some(parent_id));
    }

    #[test]
    fn insertion_position_parent_before_sibling() {
        let mut arena: Arena<XpathItemTreeNode> = Arena::new();
        let parent_node = XpathItemTreeNode::ElementNode(ElementNode::new("div".to_string()));
        let parent_id = arena.new_node(parent_node);

        let child_node = XpathItemTreeNode::ElementNode(ElementNode::new("p".to_string()));
        let child_id = arena.new_node(child_node);
        parent_id.append(child_id, &mut arena);

        let pos = InsertionPosition::BeforeSibling(child_id);
        assert_eq!(pos.parent(&arena), Some(parent_id));
    }

    #[test]
    fn is_foster_parenting_target_returns_true_for_table_elements() {
        let mut parser = HtmlParser::new();

        for tag in &["table", "tbody", "tfoot", "thead", "tr"] {
            let node =
                XpathItemTreeNode::ElementNode(ElementNode::new(tag.to_string()));
            let id = parser.new_node(node);
            assert!(
                parser.is_foster_parenting_target(id),
                "{} should be a table scope element",
                tag
            );
        }
    }

    #[test]
    fn is_foster_parenting_target_returns_false_for_non_table_elements() {
        let mut parser = HtmlParser::new();

        for tag in &["div", "p", "span", "body", "html", "template"] {
            let node =
                XpathItemTreeNode::ElementNode(ElementNode::new(tag.to_string()));
            let id = parser.new_node(node);
            assert!(
                !parser.is_foster_parenting_target(id),
                "{} should NOT be a table scope element",
                tag
            );
        }
    }

    #[test]
    fn svg_attribute_name_returns_correct_casing() {
        assert_eq!(svg_attribute_name("viewbox"), Some("viewBox"));
        assert_eq!(svg_attribute_name("preserveaspectratio"), Some("preserveAspectRatio"));
        assert_eq!(svg_attribute_name("attributename"), Some("attributeName"));
        assert_eq!(svg_attribute_name("gradientunits"), Some("gradientUnits"));
        assert_eq!(svg_attribute_name("stddeviation"), Some("stdDeviation"));
        assert_eq!(svg_attribute_name("xchannelselector"), Some("xChannelSelector"));
    }

    #[test]
    fn svg_attribute_name_returns_none_for_non_adjusted() {
        assert_eq!(svg_attribute_name("width"), None);
        assert_eq!(svg_attribute_name("height"), None);
        assert_eq!(svg_attribute_name("fill"), None);
        assert_eq!(svg_attribute_name("class"), None);
        assert_eq!(svg_attribute_name("nonexistent"), None);
    }

    #[test]
    fn adjust_mathml_attributes_renames_definitionurl() {
        let mut token = TagToken::new(String::from("math"));
        token.attributes.push(tokenizer::Attribute {
            name: String::from("definitionurl"),
            value: String::from("http://example.com"),
            prefix: String::new(),
            original_name: None,
            namespace: None,
        });
        HtmlParser::adjust_mathml_attributes(&mut token);
        assert_eq!(token.attributes[0].name, "definitionURL");
    }

    #[test]
    fn adjust_mathml_attributes_does_not_change_other_attributes() {
        let mut token = TagToken::new(String::from("math"));
        token.attributes.push(tokenizer::Attribute {
            name: String::from("display"),
            value: String::from("block"),
            prefix: String::new(),
            original_name: None,
            namespace: None,
        });
        HtmlParser::adjust_mathml_attributes(&mut token);
        assert_eq!(token.attributes[0].name, "display");
    }

    #[test]
    fn adjust_svg_attributes_fixes_casing() {
        let mut token = TagToken::new(String::from("svg"));
        token.attributes.push(tokenizer::Attribute {
            name: String::from("viewbox"),
            value: String::from("0 0 100 100"),
            prefix: String::new(),
            original_name: None,
            namespace: None,
        });
        token.attributes.push(tokenizer::Attribute {
            name: String::from("preserveaspectratio"),
            value: String::from("xMidYMid"),
            prefix: String::new(),
            original_name: None,
            namespace: None,
        });
        token.attributes.push(tokenizer::Attribute {
            name: String::from("width"),
            value: String::from("100"),
            prefix: String::new(),
            original_name: None,
            namespace: None,
        });
        HtmlParser::adjust_svg_attributes(&mut token);
        assert_eq!(token.attributes[0].name, "viewBox");
        assert_eq!(token.attributes[1].name, "preserveAspectRatio");
        assert_eq!(token.attributes[2].name, "width"); // unchanged
    }

    #[test]
    fn svg_element_name_correction_foreignobject() {
        assert_eq!(svg_element_name("foreignobject"), Some("foreignObject"));
    }

    #[test]
    fn svg_element_name_correction_clippath() {
        assert_eq!(svg_element_name("clippath"), Some("clipPath"));
    }

    #[test]
    fn svg_element_name_correction_all_entries() {
        // Verify all 37 entries in the table return correct values.
        let cases = [
            ("altglyph", "altGlyph"),
            ("altglyphdef", "altGlyphDef"),
            ("altglyphitem", "altGlyphItem"),
            ("animatecolor", "animateColor"),
            ("animatemotion", "animateMotion"),
            ("animatetransform", "animateTransform"),
            ("clippath", "clipPath"),
            ("feblend", "feBlend"),
            ("fecolormatrix", "feColorMatrix"),
            ("fecomponenttransfer", "feComponentTransfer"),
            ("fecomposite", "feComposite"),
            ("feconvolvematrix", "feConvolveMatrix"),
            ("fediffuselighting", "feDiffuseLighting"),
            ("fedisplacementmap", "feDisplacementMap"),
            ("fedistantlight", "feDistantLight"),
            ("fedropshadow", "feDropShadow"),
            ("feflood", "feFlood"),
            ("fefunca", "feFuncA"),
            ("fefuncb", "feFuncB"),
            ("fefuncg", "feFuncG"),
            ("fefuncr", "feFuncR"),
            ("fegaussianblur", "feGaussianBlur"),
            ("feimage", "feImage"),
            ("femerge", "feMerge"),
            ("femergenode", "feMergeNode"),
            ("femorphology", "feMorphology"),
            ("feoffset", "feOffset"),
            ("fepointlight", "fePointLight"),
            ("fespecularlighting", "feSpecularLighting"),
            ("fespotlight", "feSpotLight"),
            ("fetile", "feTile"),
            ("feturbulence", "feTurbulence"),
            ("foreignobject", "foreignObject"),
            ("glyphref", "glyphRef"),
            ("lineargradient", "linearGradient"),
            ("radialgradient", "radialGradient"),
            ("textpath", "textPath"),
        ];
        for (lowered, expected) in cases {
            assert_eq!(
                svg_element_name(lowered),
                Some(expected),
                "svg_element_name({lowered:?}) should return {expected:?}"
            );
        }
    }

    #[test]
    fn svg_element_name_returns_none_for_non_special_names() {
        assert_eq!(svg_element_name("svg"), None);
        assert_eq!(svg_element_name("rect"), None);
        assert_eq!(svg_element_name("circle"), None);
        assert_eq!(svg_element_name("g"), None);
        assert_eq!(svg_element_name("path"), None);
        assert_eq!(svg_element_name("text"), None);
    }

    #[test]
    fn adjust_svg_tag_names_corrects_token() {
        let mut token = TagToken {
            tag_name: String::from("foreignobject"),
            self_closing: false,
            attributes: vec![],
        };
        HtmlParser::adjust_svg_tag_names(&mut token);
        assert_eq!(token.tag_name, "foreignObject");
    }

    #[test]
    fn adjust_svg_tag_names_no_op_for_regular_svg_element() {
        let mut token = TagToken {
            tag_name: String::from("rect"),
            self_closing: false,
            attributes: vec![],
        };
        HtmlParser::adjust_svg_tag_names(&mut token);
        assert_eq!(token.tag_name, "rect");
    }

    #[test]
    fn create_element_for_token_corrects_svg_element_name() {
        let mut parser = HtmlParser::new();
        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        let token = TagToken {
            tag_name: String::from("foreignobject"),
            self_closing: false,
            attributes: vec![],
        };
        let result = parser
            .create_an_element_for_the_token(token, SVG_NAMESPACE)
            .unwrap();
        assert_eq!(result.element.name, "foreignObject");
        assert_eq!(
            result.element.namespace.as_deref(),
            Some(SVG_NAMESPACE)
        );
    }

    #[test]
    fn create_element_for_token_does_not_correct_html_namespace() {
        let mut parser = HtmlParser::new();
        let doc_id = parser
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));
        parser.root_node = Some(doc_id);

        // Even though "foreignobject" matches the SVG table, it should NOT be
        // corrected when created in the HTML namespace.
        let token = TagToken {
            tag_name: String::from("foreignobject"),
            self_closing: false,
            attributes: vec![],
        };
        let result = parser
            .create_an_element_for_the_token(token, HTML_NAMESPACE)
            .unwrap();
        assert_eq!(result.element.name, "foreignobject");
        assert_eq!(result.element.namespace, None);
    }

    #[test]
    fn foreign_attribute_namespace_xlink() {
        let xlink_ns = "http://www.w3.org/1999/xlink";
        assert_eq!(foreign_attribute_namespace("xlink:actuate"), Some(xlink_ns));
        assert_eq!(foreign_attribute_namespace("xlink:arcrole"), Some(xlink_ns));
        assert_eq!(foreign_attribute_namespace("xlink:href"), Some(xlink_ns));
        assert_eq!(foreign_attribute_namespace("xlink:role"), Some(xlink_ns));
        assert_eq!(foreign_attribute_namespace("xlink:show"), Some(xlink_ns));
        assert_eq!(foreign_attribute_namespace("xlink:title"), Some(xlink_ns));
        assert_eq!(foreign_attribute_namespace("xlink:type"), Some(xlink_ns));
    }

    #[test]
    fn foreign_attribute_namespace_xml() {
        let xml_ns = "http://www.w3.org/XML/1998/namespace";
        assert_eq!(foreign_attribute_namespace("xml:lang"), Some(xml_ns));
        assert_eq!(foreign_attribute_namespace("xml:space"), Some(xml_ns));
    }

    #[test]
    fn foreign_attribute_namespace_xmlns() {
        let xmlns_ns = "http://www.w3.org/2000/xmlns/";
        assert_eq!(foreign_attribute_namespace("xmlns"), Some(xmlns_ns));
        assert_eq!(foreign_attribute_namespace("xmlns:xlink"), Some(xmlns_ns));
    }

    #[test]
    fn foreign_attribute_namespace_returns_none_for_regular() {
        assert_eq!(foreign_attribute_namespace("class"), None);
        assert_eq!(foreign_attribute_namespace("id"), None);
        assert_eq!(foreign_attribute_namespace("href"), None);
        assert_eq!(foreign_attribute_namespace("viewBox"), None);
        assert_eq!(foreign_attribute_namespace("width"), None);
    }

    #[test]
    fn adjust_foreign_attributes_sets_namespace_on_xlink_href() {
        let mut token = TagToken::new(String::from("use"));
        token.attributes.push(tokenizer::Attribute {
            name: String::from("xlink:href"),
            value: String::from("#icon"),
            prefix: String::new(),
            original_name: None,
            namespace: None,
        });
        HtmlParser::adjust_foreign_attributes(&mut token);
        assert_eq!(
            token.attributes[0].namespace.as_deref(),
            Some("http://www.w3.org/1999/xlink")
        );
    }

    #[test]
    fn adjust_foreign_attributes_does_not_set_namespace_on_regular_attributes() {
        let mut token = TagToken::new(String::from("rect"));
        token.attributes.push(tokenizer::Attribute {
            name: String::from("width"),
            value: String::from("100"),
            prefix: String::new(),
            original_name: None,
            namespace: None,
        });
        HtmlParser::adjust_foreign_attributes(&mut token);
        assert_eq!(token.attributes[0].namespace, None);
    }
}
