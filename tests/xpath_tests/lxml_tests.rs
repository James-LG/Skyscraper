use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use itertools::Itertools;
use serde::Deserialize;
use skyscraper::{
    html,
    xpath::{self, xpath_item_set::XpathItemSet, XpathItemTree},
};

#[derive(Deserialize, Debug, PartialEq)]
struct LxmlElement {
    pub tag: String,
    pub text: Option<String>,
    pub text_content: String,
    pub attrib: HashMap<String, String>,
    pub itertext: Vec<String>,
}

fn get_lxml_output(xpath: &str, html_text: String, count_only: bool) -> std::process::Output {
    let mut lxml_python_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lxml_python_path.push("tests/lxml_tests/xpath.py");

    // Use the uv-managed venv Python so lxml/jsons deps are available.
    let mut venv_python = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    venv_python.push("tests/lxml_tests/.venv/bin/python");

    let python = if venv_python.exists() {
        venv_python.into_os_string().into_string().unwrap()
    } else {
        "python3".to_string()
    };

    let mut cmd = Command::new(python);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg(
            lxml_python_path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .arg(xpath);

    if count_only {
        cmd.arg("--count-only");
    }

    let mut process = cmd.spawn().expect("failed to spawn process");

    let mut stdin = process.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(html_text.as_bytes())
            .expect("Failed to write to stdin");
    });

    let output = process
        .wait_with_output()
        .expect("failed to execute stack overflow tests");

    output
}

fn get_lxml_elements(xpath: &str, html_text: String) -> Vec<LxmlElement> {
    let output = get_lxml_output(xpath, html_text, false);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{}\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );

    let lxml_elements: Vec<LxmlElement> = serde_json::from_str(&stdout).unwrap();
    return lxml_elements;
}

fn skyscraper_to_lxml_elements(
    xpath_tree: &XpathItemTree,
    item_set: XpathItemSet,
) -> Vec<LxmlElement> {
    let mut lxml_elements = Vec::new();
    for item in item_set.into_iter() {
        let node = item.extract_into_node();
        let element = node.extract_as_element_node();
        let text = element.text(&xpath_tree);
        let text_content = element.text_content(&xpath_tree);
        let itertext = element.itertext(&xpath_tree).collect();

        lxml_elements.push(LxmlElement {
            tag: element.name.to_string(),
            text,
            text_content,
            attrib: element
                .attributes(&xpath_tree)
                .iter()
                .map(|x| (x.name.clone(), x.value.clone()))
                .collect(),
            itertext,
        });
    }
    return lxml_elements;
}

fn compare_skyscraper_to_lxml(
    lxml_elements: Vec<LxmlElement>,
    converted_skyscraper_elems: Vec<LxmlElement>,
) {
    for (i, eb) in lxml_elements
        .iter()
        .zip_longest(converted_skyscraper_elems.iter())
        .enumerate()
    {
        let (lxml_elem, skyscraper_elem) = eb.left_and_right();

        if let (Some(lxml_elem), Some(skyscraper_elem)) = (lxml_elem, skyscraper_elem) {
            assert_eq!(
                lxml_elem.tag, skyscraper_elem.tag,
                "Tag mismatch at index {}",
                i
            );
            assert_eq!(
                lxml_elem.text, skyscraper_elem.text,
                "Text mismatch at index {}",
                i
            );
            // Compare attributes case-insensitively for keys because lxml
            // lowercases all HTML attribute names, while skyscraper correctly
            // preserves SVG attribute casing per the WHATWG spec (e.g.
            // "viewBox" vs lxml's "viewbox").
            let lxml_lower: HashMap<String, String> = lxml_elem
                .attrib
                .iter()
                .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
                .collect();
            let sky_lower: HashMap<String, String> = skyscraper_elem
                .attrib
                .iter()
                .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
                .collect();
            assert_eq!(
                lxml_lower, sky_lower,
                "Attribute mismatch at index {}",
                i
            );
            compare_itertext(&lxml_elem.itertext, &skyscraper_elem.itertext);
        } else {
            assert_eq!(
                lxml_elem, skyscraper_elem,
                "Element mismatch at index {}",
                i
            );
        }
    }
    assert_eq!(converted_skyscraper_elems.len(), lxml_elements.len());
}

fn compare_itertext(first: &Vec<String>, second: &Vec<String>) {
    for (i, eb) in first.iter().zip_longest(second.iter()).enumerate() {
        let (first, second) = eb.left_and_right();
        assert_eq!(first, second, "Itertext mismatch at index {}", i);
    }
}

static GITHUB_HTML: &'static str = include_str!("../samples/James-LG_Skyscraper.html");

/// This test is a sanity check of the lxml output.
#[test]
fn test_lxml_output() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//a[@rel='author']"#;

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);

    // assert
    assert_eq!(lxml_elements.len(), 1);

    let mut lxml_elements = lxml_elements.into_iter();
    let lxml_element = lxml_elements.next().unwrap();
    assert_eq!(lxml_element.tag, "a");
    assert_eq!(lxml_element.text, Some("James-LG".to_string()));
    assert_eq!(lxml_element.attrib["rel"], "author");
}

/// Selects a large block of text and checks that Skyscraper handles text the same as lxml.
#[test]
fn test_text_handling() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//div[@role='tabpanel']"#;

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

#[test]
fn test_text_handling2() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//h2"#;

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

#[test]
fn test_text_handling3() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//div"#;

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

#[test]
fn test_item_count1() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = "//div[@class='flex-auto min-width-0 width-fit mr-3']";

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_output = get_lxml_output(xpath, html_text, true);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let output = String::from_utf8_lossy(&lxml_output.stdout);
    let lxml_count = output.trim().parse::<usize>().unwrap();
    assert_eq!(lxml_count, skyscraper_elements.len());
}

/// Helper: run a full comparison test between skyscraper and lxml for a given XPath.
fn run_lxml_comparison(xpath: &str) {
    run_lxml_comparison_with_html(xpath, GITHUB_HTML);
}

/// Helper: run a full comparison test with custom HTML.
fn run_lxml_comparison_with_html(xpath: &str, html_text: &str) {
    let html_document = html::parse(html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    let lxml_elements = get_lxml_elements(xpath, html_text.to_string());
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

/// Helper: run a count-only comparison test between skyscraper and lxml for a given XPath.
fn run_lxml_count_comparison(xpath: &str) {
    run_lxml_count_comparison_with_html(xpath, GITHUB_HTML);
}

/// Helper: run a count-only comparison test with custom HTML.
fn run_lxml_count_comparison_with_html(xpath: &str, html_text: &str) {
    let html_document = html::parse(html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    let lxml_output = get_lxml_output(xpath, html_text.to_string(), true);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    let output = String::from_utf8_lossy(&lxml_output.stdout);
    let lxml_count = output.trim().parse::<usize>().unwrap();
    assert!(lxml_count > 0, "lxml returned 0 results for '{}'", xpath);
    assert_eq!(lxml_count, skyscraper_elements.len());
}

// ===== Simple element selection =====

/// Select all span elements.
#[test]
fn test_select_all_spans() {
    run_lxml_comparison("//span");
}

/// Select all anchor elements.
#[test]
fn test_select_all_anchors() {
    run_lxml_comparison("//a");
}

/// Select all list items.
#[test]
fn test_select_all_li() {
    run_lxml_comparison("//li");
}

/// Select all paragraph elements.
#[test]
fn test_select_all_p() {
    run_lxml_comparison("//p");
}

/// Select all h1 elements.
#[test]
fn test_select_all_h1() {
    run_lxml_comparison("//h1");
}

/// Select all button elements.
#[test]
fn test_select_all_buttons() {
    run_lxml_comparison("//button");
}

/// Select all summary elements.
#[test]
fn test_select_all_summary() {
    run_lxml_comparison("//summary");
}

/// Select all img elements (void element).
#[test]
fn test_select_all_img() {
    run_lxml_comparison("//img");
}

/// Select all meta elements (void element).
#[test]
fn test_select_all_meta() {
    run_lxml_comparison("//meta");
}

/// Select all link elements (void element).
#[test]
fn test_select_all_link() {
    run_lxml_comparison("//link");
}

/// Select all input elements (void element).
#[test]
fn test_select_all_input() {
    run_lxml_comparison("//input");
}

/// Select all form elements.
#[test]
fn test_select_all_form() {
    run_lxml_comparison("//form");
}

/// Select all svg elements.
#[test]
fn test_select_all_svg() {
    run_lxml_comparison("//svg");
}

// ===== Child axis / multi-step paths =====

/// Select anchors that are direct children of list items.
#[test]
fn test_child_li_a() {
    run_lxml_comparison("//li/a");
}

/// Select list items that are direct children of unordered lists.
#[test]
fn test_child_ul_li() {
    run_lxml_comparison("//ul/li");
}

/// Select spans that are direct children of divs.
#[test]
fn test_child_div_span() {
    run_lxml_comparison("//div/span");
}

/// Select divs that are direct children of a specific div.
#[test]
fn test_child_of_specific_div() {
    run_lxml_comparison("//div[@class='position-relative']/div");
}

// ===== Descendant axis =====

/// Select anchors that are descendants of nav elements.
#[test]
fn test_descendant_nav_a() {
    run_lxml_comparison("//nav//a");
}

/// Select anchors descended from header > nav chain.
#[test]
fn test_descendant_header_nav_a() {
    run_lxml_comparison("//header//nav//a");
}

/// Select summary elements descended from details.
#[test]
fn test_descendant_details_summary() {
    run_lxml_comparison("//details//summary");
}

/// Select spans descended from a specific div.
#[test]
fn test_descendant_specific_div_span() {
    run_lxml_comparison("//div[@class='position-relative']//span");
}

// ===== Parent axis =====

/// Select parent elements of all anchors.
#[test]
fn test_parent_axis() {
    run_lxml_comparison("//a/..");
}

// ===== Attribute predicates =====

/// Select anchors with contains() on href attribute.
#[test]
fn test_attr_contains_href() {
    run_lxml_comparison("//a[contains(@href, 'github')]");
}

/// Select anchors with contains() on class attribute.
#[test]
fn test_attr_contains_class() {
    run_lxml_comparison("//a[contains(@class, 'Link')]");
}

/// Select spans that have a class attribute.
#[test]
fn test_attr_has_class() {
    run_lxml_comparison("//span[@class]");
}

/// Select divs that have an id attribute.
#[test]
fn test_attr_has_id() {
    run_lxml_comparison("//div[@id]");
}

/// Select anchors that have both class and href attributes.
#[test]
fn test_attr_multiple_existence() {
    run_lxml_comparison("//a[@class and @href]");
}

/// Select divs with an exact class match.
#[test]
fn test_attr_exact_class() {
    run_lxml_comparison("//div[@class='position-relative']");
}

/// Select spans with a data-content attribute.
#[test]
fn test_attr_data_content() {
    run_lxml_comparison("//span[@data-content]");
}

/// Select anchors with aria-label attribute.
#[test]
fn test_attr_aria_label() {
    run_lxml_comparison("//a[@aria-label]");
}

/// Select anchors with data-analytics-event attribute.
#[test]
fn test_attr_data_analytics() {
    run_lxml_comparison("//a[@data-analytics-event]");
}

/// Select images with alt attribute.
#[test]
fn test_attr_img_alt() {
    run_lxml_comparison("//img[@alt]");
}

/// Select images with src attribute.
#[test]
fn test_attr_img_src() {
    run_lxml_comparison("//img[@src]");
}

// ===== Negation predicate =====

/// Select anchors that do NOT have a class attribute.
#[test]
fn test_not_attr() {
    run_lxml_comparison("//a[not(@class)]");
}

/// Select anchors with href but without class.
#[test]
fn test_attr_and_not() {
    run_lxml_comparison("//a[@href and not(@class)]");
}

// ===== Wildcard =====

/// Select all elements (wildcard) with a specific attribute.
#[test]
fn test_wildcard_with_attr() {
    run_lxml_count_comparison("//*[@role]");
}

// ===== Positional predicates =====

/// Select the first anchor element using a filter expression.
#[test]
fn test_positional_first() {
    run_lxml_comparison("(//a)[1]");
}

/// Select the last span element using a filter expression.
#[test]
fn test_positional_last() {
    run_lxml_comparison("(//span)[last()]");
}

/// Select list items in the first 3 positions.
#[test]
fn test_positional_lte() {
    run_lxml_comparison("//li[position() <= 3]");
}

// ===== String function predicates =====

/// Select spans where string-length of class attribute is greater than 20.
#[test]
fn test_string_length_predicate() {
    run_lxml_comparison("//span[string-length(@class) > 20]");
}

// ===== Existential element predicates =====

/// Select divs that contain at least one svg descendant.
#[test]
fn test_element_exists_descendant() {
    run_lxml_count_comparison("//div[.//svg]");
}

/// Select divs that contain a direct child paragraph.
#[test]
fn test_element_exists_child() {
    run_lxml_comparison("//div[p]");
}

/// Select unordered lists that contain li > a chains.
#[test]
fn test_element_exists_nested() {
    run_lxml_comparison("//ul[li/a]");
}

// ===== count() function in predicates =====

/// Select divs with more than zero direct anchor children.
#[test]
fn test_count_predicate() {
    run_lxml_count_comparison("//div[count(a) > 0]");
}

// ===== Union expressions =====

/// Select both anchors and spans via union operator.
#[test]
fn test_union_a_or_span() {
    run_lxml_count_comparison("//a | //span");
}

// ===== Or predicate =====

/// Select anchors matching either of two class substrings.
#[test]
fn test_or_predicate() {
    run_lxml_comparison("//a[contains(@class, 'Link') or contains(@class, 'btn')]");
}

/// Select list items containing either anchors or spans.
#[test]
fn test_or_predicate_child_element() {
    run_lxml_comparison("//li[a or span]");
}

// ===== Not-equal comparison =====

/// Select divs whose class is not a specific value.
#[test]
fn test_not_equal_attr() {
    run_lxml_count_comparison("//div[@class != 'position-relative']");
}

// ===== Ancestor axis =====

/// Select ancestor divs of a deeply nested element.
#[test]
fn test_ancestor_axis() {
    run_lxml_comparison("//a[@rel='author']/ancestor::div");
}

/// Select all ancestor divs of a specific class of div.
#[test]
fn test_ancestor_axis_from_class() {
    run_lxml_comparison("//div[@class='position-relative']/ancestor::div");
}

/// Select the nearest ancestor div.
#[test]
fn test_ancestor_axis_positional() {
    run_lxml_comparison("//a[@rel='author']/ancestor::div[1]");
}

/// Select the outermost ancestor div.
#[test]
fn test_ancestor_axis_last() {
    run_lxml_comparison("//a[@rel='author']/ancestor::div[last()]");
}

/// Select nearest ancestor of any type.
#[test]
fn test_ancestor_wildcard() {
    run_lxml_comparison("//a[@rel='author']/ancestor::*[1]");
}

// ===== Ancestor-or-self axis =====

/// Select self and all ancestor divs.
#[test]
fn test_ancestor_or_self_axis() {
    run_lxml_comparison("//div[@class='position-relative']/ancestor-or-self::div");
}

// ===== Self axis =====

/// Select self node via explicit self axis.
#[test]
fn test_self_axis() {
    run_lxml_comparison("//a[@rel='author']/self::a");
}

// ===== Following-sibling axis =====

/// Select following sibling list items.
#[test]
fn test_following_sibling() {
    run_lxml_comparison("//li/following-sibling::li");
}

/// Select divs with a class that have a following sibling div.
#[test]
fn test_following_sibling_predicate() {
    run_lxml_count_comparison("//div[@class][following-sibling::div]");
}

// ===== Preceding-sibling axis =====

/// Select preceding sibling list items.
#[test]
fn test_preceding_sibling() {
    run_lxml_comparison("//li/preceding-sibling::li");
}

/// Select divs with a class that have a preceding sibling div.
#[test]
fn test_preceding_sibling_predicate() {
    run_lxml_count_comparison("//div[@class][preceding-sibling::div]");
}

// ===== Following axis =====

/// Select h2 elements following any h1.
#[test]
fn test_following_axis() {
    run_lxml_comparison("//h1/following::h2");
}

// ===== Preceding axis =====

/// Select h1 elements preceding any h2.
#[test]
fn test_preceding_axis() {
    run_lxml_comparison("//h2/preceding::h1");
}

// ===== Multiple chained predicates =====

/// Chain two attribute predicates plus positional.
#[test]
fn test_chained_predicates() {
    run_lxml_count_comparison("//a[@class][contains(@href, 'github')][1]");
}

/// Two attribute-existence predicates.
#[test]
fn test_two_attr_predicates() {
    run_lxml_comparison("//div[@class][span]");
}

// ===== Nested predicates =====

/// Select divs that contain a div that contains an anchor.
#[test]
fn test_nested_predicate() {
    run_lxml_comparison("//div[div[a]]");
}

/// Select list items containing anchors with href.
#[test]
fn test_nested_attr_predicate() {
    run_lxml_comparison("//li[a[@href]]");
}

/// Select divs containing anchors with class.
#[test]
fn test_nested_attr_predicate2() {
    run_lxml_comparison("//div[a[@class]]");
}

// ===== Compound negation predicates =====

/// Select anchors with href that do not start with #.
#[test]
fn test_compound_negation() {
    run_lxml_count_comparison("//a[@href][not(starts-with(@href, '#'))]");
}

// ===== Positional in context =====

/// Select positional range of divs.
#[test]
fn test_positional_range() {
    run_lxml_comparison("(//div)[position() >= 3 and position() <= 5]");
}

/// Select first li child of each ul.
#[test]
fn test_positional_first_child() {
    run_lxml_comparison("//ul/li[1]");
}

/// Select last li child of each ul.
#[test]
fn test_positional_last_child() {
    run_lxml_comparison("//ul/li[last()]");
}

/// Select the last div among siblings at each level.
#[test]
fn test_positional_last_sibling() {
    run_lxml_count_comparison("//div[last()]");
}

/// Select the first div among siblings at each level.
#[test]
fn test_positional_first_sibling() {
    run_lxml_count_comparison("//div[1]");
}

// ===== String functions in predicates =====

/// Select anchors with long href values.
#[test]
fn test_string_length_href() {
    run_lxml_comparison("//a[string-length(@href) > 50]");
}

/// Select anchors where normalize-space matches exactly.
#[test]
fn test_normalize_space_predicate() {
    run_lxml_comparison("//a[normalize-space(@class) = 'Link--secondary']");
}

/// Select anchors whose text content contains a specific string.
#[test]
fn test_contains_text_content() {
    run_lxml_comparison("//a[contains(., 'James')]");
}

/// Select divs whose text content contains a specific string.
#[test]
fn test_contains_dot_text() {
    run_lxml_count_comparison("//div[contains(., 'Skyscraper')]");
}

/// Select anchors with non-empty normalized text.
#[test]
fn test_string_length_normalize_space_dot() {
    run_lxml_count_comparison("//a[string-length(normalize-space(.)) > 0]");
}

// ===== Absolute paths =====

/// Select divs with id via absolute path from root.
#[test]
fn test_absolute_path() {
    run_lxml_comparison("/html/body//div[@id]");
}

/// Select direct child divs of body.
#[test]
fn test_absolute_path_direct_children() {
    run_lxml_comparison("/html/body/div");
}

// ===== Parent axis combined with child step =====

/// Navigate up to parent then back down to sibling span.
#[test]
fn test_parent_then_child() {
    run_lxml_comparison("//a[@href]/../span");
}

// ===== count() with higher thresholds =====

/// Select divs with many child divs.
#[test]
fn test_count_many_children() {
    run_lxml_count_comparison("//div[count(div) > 3]");
}

/// Select divs with many children of any kind.
#[test]
fn test_count_wildcard_children() {
    run_lxml_count_comparison("//div[count(*) > 10]");
}

// ===== Custom HTML tests for targeted feature coverage =====

static CUSTOM_HTML: &str = r#"<html><body>
<div id="root">
  <ul class="list">
    <li class="item first">Alpha</li>
    <li class="item">Beta</li>
    <li class="item last">Gamma</li>
  </ul>
  <div class="content">
    <p>Hello <strong>bold</strong> world</p>
    <p class="intro">Second <em>emphasized</em> paragraph</p>
  </div>
  <!-- Explicit tbody so Skyscraper (WHATWG) and lxml produce the same tree. -->
  <table><tbody>
    <tr><td class="c1">A1</td><td class="c2">A2</td></tr>
    <tr><td class="c1">B1</td><td class="c2">B2</td></tr>
  </tbody></table>
  <div class="nested">
    <div class="inner"><span data-x="1">deep</span></div>
  </div>
  <div class="siblings">
    <span class="a">first</span>
    <span class="b">second</span>
    <span class="c">third</span>
  </div>
</div>
</body></html>"#;

/// following-sibling on custom HTML with known structure.
#[test]
fn test_custom_following_sibling() {
    run_lxml_comparison_with_html("//li[@class='item first']/following-sibling::li", CUSTOM_HTML);
}

/// preceding-sibling on custom HTML with known structure.
#[test]
fn test_custom_preceding_sibling() {
    run_lxml_comparison_with_html("//li[@class='item last']/preceding-sibling::li", CUSTOM_HTML);
}

/// ancestor axis on custom HTML.
#[test]
fn test_custom_ancestor() {
    run_lxml_comparison_with_html("//strong/ancestor::div", CUSTOM_HTML);
}

/// ancestor-or-self axis on custom HTML.
#[test]
fn test_custom_ancestor_or_self() {
    run_lxml_comparison_with_html("//div[@class='inner']/ancestor-or-self::div", CUSTOM_HTML);
}

/// following axis on custom HTML.
#[test]
fn test_custom_following() {
    run_lxml_comparison_with_html("//ul/following::div", CUSTOM_HTML);
}

/// preceding axis on custom HTML.
#[test]
fn test_custom_preceding() {
    run_lxml_comparison_with_html("//table/preceding::div", CUSTOM_HTML);
}

/// self axis with type test on custom HTML.
#[test]
fn test_custom_self_axis() {
    run_lxml_comparison_with_html("//p/self::p", CUSTOM_HTML);
}

/// union expression on custom HTML.
#[test]
fn test_custom_union() {
    run_lxml_count_comparison_with_html("//strong | //em", CUSTOM_HTML);
}

/// Navigate parent then into sibling on custom HTML.
#[test]
fn test_custom_parent_then_sibling() {
    run_lxml_comparison_with_html("//strong/..", CUSTOM_HTML);
}

/// Multiple steps with mixed axes on custom HTML.
#[test]
fn test_custom_child_descendant_mix() {
    run_lxml_comparison_with_html("//div[@id='root']/div//span", CUSTOM_HTML);
}

/// Nested predicates on custom HTML.
#[test]
fn test_custom_nested_predicate() {
    run_lxml_comparison_with_html("//div[p[strong]]", CUSTOM_HTML);
}

/// or predicate on custom HTML.
#[test]
fn test_custom_or_predicate() {
    run_lxml_comparison_with_html("//li[@class='item first' or @class='item last']", CUSTOM_HTML);
}

/// contains on text content on custom HTML.
#[test]
fn test_custom_contains_dot() {
    run_lxml_comparison_with_html("//p[contains(., 'bold')]", CUSTOM_HTML);
}

/// Positional predicate [1] in context of each parent on custom HTML.
#[test]
fn test_custom_positional_first_td() {
    run_lxml_comparison_with_html("//tr/td[1]", CUSTOM_HTML);
}

/// Positional predicate [last()] in context of each parent on custom HTML.
#[test]
fn test_custom_positional_last_td() {
    run_lxml_comparison_with_html("//tr/td[last()]", CUSTOM_HTML);
}

/// Wildcard descendant selection on custom HTML.
#[test]
fn test_custom_wildcard_descendants() {
    run_lxml_count_comparison_with_html("//div[@class='content']//*", CUSTOM_HTML);
}

/// Chain: descendant -> attribute predicate -> child on custom HTML.
#[test]
fn test_custom_multi_step_chain() {
    run_lxml_comparison_with_html("//div[@class='nested']//span[@data-x]", CUSTOM_HTML);
}

/// Select siblings following the first span in siblings div.
#[test]
fn test_custom_following_sibling_span() {
    run_lxml_comparison_with_html(
        "//div[@class='siblings']/span[@class='a']/following-sibling::span",
        CUSTOM_HTML,
    );
}

/// not() combined with contains() on custom HTML.
#[test]
fn test_custom_not_contains() {
    run_lxml_comparison_with_html("//li[not(contains(@class, 'first'))]", CUSTOM_HTML);
}

/// String comparison with normalize-space on custom HTML.
#[test]
fn test_custom_normalize_space() {
    run_lxml_comparison_with_html("//li[normalize-space(.) = 'Beta']", CUSTOM_HTML);
}

// ===== Additional complex expressions targeting different failure modes =====

/// Select script elements with a type attribute.
#[test]
fn test_select_script_with_type() {
    run_lxml_comparison("//script[@type]");
}

/// Select script with a specific type.
#[test]
fn test_select_script_exact_type() {
    run_lxml_comparison("//script[@type='application/json']");
}

/// Select label elements.
#[test]
fn test_select_all_label() {
    run_lxml_comparison("//label");
}

/// Select header elements.
#[test]
fn test_select_all_header() {
    run_lxml_comparison("//header");
}

/// Select footer elements.
#[test]
fn test_select_all_footer() {
    run_lxml_comparison("//footer");
}

/// Select nav elements.
#[test]
fn test_select_all_nav() {
    run_lxml_comparison("//nav");
}

/// Select the single main element.
#[test]
fn test_select_main() {
    run_lxml_comparison("//main");
}

/// Select the article element.
#[test]
fn test_select_article() {
    run_lxml_comparison("//article");
}

/// Select the style element.
#[test]
fn test_select_style() {
    run_lxml_comparison("//style");
}

// ===== substring() function =====

/// Select anchors whose href starts with 'https' using substring.
#[test]
fn test_substring_predicate() {
    run_lxml_count_comparison("//a[substring(@href, 1, 5) = 'https']");
}

// ===== translate() function =====

/// Use translate to do case-insensitive comparison.
#[test]
fn test_translate_predicate() {
    run_lxml_comparison("//a[translate(@rel, 'AUTHOR', 'author') = 'author']");
}

// ===== string() function =====

/// Select anchors whose string value exactly matches.
#[test]
fn test_string_function_exact() {
    run_lxml_comparison("//a[string(.) = 'James-LG']");
}

/// Select divs where string(@class) is truthy (non-empty).
#[test]
fn test_string_function_truthy() {
    run_lxml_count_comparison("//div[string(@class)]");
}

// ===== boolean() function =====

/// Select divs where boolean(@class) is true.
#[test]
fn test_boolean_function() {
    run_lxml_count_comparison("//div[boolean(@class)]");
}

// ===== Leaf node selection =====

/// Select divs with no child elements.
#[test]
fn test_leaf_div() {
    run_lxml_count_comparison("//div[count(child::*) = 0]");
}

/// Select spans with no child elements.
#[test]
fn test_leaf_span() {
    run_lxml_count_comparison("//span[not(child::*)]");
}

// ===== Modular arithmetic =====

/// Select odd-positioned divs with [1] predicate.
#[test]
fn test_mod_positional() {
    run_lxml_count_comparison("//div[position() mod 2 = 1][1]");
}

// ===== starts-with on class =====

/// Select divs whose class starts with a prefix.
#[test]
fn test_starts_with_class() {
    run_lxml_comparison("//div[starts-with(@class, 'position')]");
}

// ===== Following-sibling with wildcard =====

/// Select the immediate following sibling of any type for divs with class.
#[test]
fn test_following_sibling_wildcard() {
    run_lxml_count_comparison("//div[@class]/following-sibling::*[1]");
}

// ===== Preceding axis with positional =====

/// Select the first preceding anchor before a specific element.
#[test]
fn test_preceding_with_positional() {
    run_lxml_comparison("//a[@rel='author']/preceding::a[1]");
}

// ===== Following-sibling with positional =====

/// Select first following-sibling after each h2[1].
#[test]
fn test_following_sibling_after_h2() {
    run_lxml_count_comparison("//h2[1]/following-sibling::*[1]");
}

// ===== Large text content predicate =====

/// Select divs with very long text content.
#[test]
fn test_string_length_dot() {
    run_lxml_count_comparison("//div[string-length(.) > 1000]");
}

// ===== Deep nesting with attribute paths on custom HTML =====

static CUSTOM_HTML_DEEP: &str = r#"<html><body>
<div id="a">
  <div id="b">
    <div id="c">
      <span class="deep">found</span>
    </div>
  </div>
</div>
<div id="flat">
  <span class="x">one</span>
  <span class="y">two</span>
  <span class="z">three</span>
</div>
<ul>
  <li>1<ul><li>1.1</li><li>1.2</li></ul></li>
  <li>2<ul><li>2.1</li><li>2.2</li></ul></li>
</ul>
</body></html>"#;

/// Ancestor chain on deeply nested elements.
#[test]
fn test_custom_deep_ancestor_chain() {
    run_lxml_comparison_with_html("//span[@class='deep']/ancestor::div", CUSTOM_HTML_DEEP);
}

/// Nested list: select inner list items.
#[test]
fn test_custom_nested_list_items() {
    run_lxml_comparison_with_html("//ul/li/ul/li", CUSTOM_HTML_DEEP);
}

/// Nested list: select outer list items (which contain inner text too).
#[test]
fn test_custom_outer_list_items() {
    run_lxml_comparison_with_html("//body/ul/li", CUSTOM_HTML_DEEP);
}

/// Preceding-sibling with positional on flat structure.
#[test]
fn test_custom_preceding_sibling_positional() {
    run_lxml_comparison_with_html(
        "//span[@class='z']/preceding-sibling::span[1]",
        CUSTOM_HTML_DEEP,
    );
}

/// Following with nested lists.
#[test]
fn test_custom_nested_list_following() {
    run_lxml_comparison_with_html("//div[@id='a']/following::div", CUSTOM_HTML_DEEP);
}

/// Absolute path into deeply nested element.
#[test]
fn test_custom_deep_absolute_path() {
    run_lxml_comparison_with_html("/html/body/div/div/div/span", CUSTOM_HTML_DEEP);
}

// =====================================================================
// Additional lxml tests — simple to complex
// =====================================================================

// ===== Explicit axis syntax =====

/// Select child elements using explicit child:: axis.
#[test]
fn test_explicit_child_axis() {
    run_lxml_comparison_with_html("//div[@id='root']/child::ul", CUSTOM_HTML);
}

/// Select descendants using explicit descendant:: axis.
#[test]
fn test_explicit_descendant_axis() {
    run_lxml_comparison_with_html("//div[@id='root']/descendant::span", CUSTOM_HTML);
}

/// Select using explicit descendant-or-self:: axis.
#[test]
fn test_explicit_descendant_or_self_axis() {
    run_lxml_comparison_with_html(
        "//div[@class='nested']/descendant-or-self::div",
        CUSTOM_HTML,
    );
}

// ===== More element selection patterns =====

/// Select all table data cells.
#[test]
fn test_select_all_td() {
    run_lxml_comparison_with_html("//td", CUSTOM_HTML);
}

/// Select all table rows.
#[test]
fn test_select_all_tr() {
    run_lxml_comparison_with_html("//tr", CUSTOM_HTML);
}

/// Select all em elements.
#[test]
fn test_select_all_em() {
    run_lxml_comparison_with_html("//em", CUSTOM_HTML);
}

/// Select all strong elements.
#[test]
fn test_select_all_strong() {
    run_lxml_comparison_with_html("//strong", CUSTOM_HTML);
}

// ===== Positional — last() arithmetic =====

/// Select the second to last li in each ul.
#[test]
fn test_positional_last_minus_one() {
    run_lxml_comparison_with_html("//ul/li[last() - 1]", CUSTOM_HTML);
}

/// Select the element at position = last (i.e. only the last).
#[test]
fn test_positional_position_eq_last() {
    run_lxml_comparison_with_html("//ul/li[position() = last()]", CUSTOM_HTML);
}

/// Select all li elements except the last in each ul.
#[test]
fn test_positional_not_last() {
    run_lxml_comparison_with_html("//ul/li[position() != last()]", CUSTOM_HTML);
}

// ===== Arithmetic in predicates =====

/// Select divs where count of children is greater than 2.
#[test]
fn test_count_gt_two() {
    run_lxml_count_comparison_with_html("//div[count(*) > 2]", CUSTOM_HTML);
}

/// Select divs where count of children equals 1.
#[test]
fn test_count_eq_one() {
    run_lxml_comparison_with_html("//div[count(*) = 1]", CUSTOM_HTML);
}

/// Arithmetic: count(child) + 1 > 3.
#[test]
fn test_arithmetic_count_plus() {
    run_lxml_count_comparison_with_html("//div[count(*) + 1 > 3]", CUSTOM_HTML);
}

/// Arithmetic: string-length comparison with subtraction.
#[test]
fn test_arithmetic_string_length_sub() {
    run_lxml_comparison_with_html("//li[string-length(@class) - 4 > 0]", CUSTOM_HTML);
}

/// Position mod 2 = 0 (even positioned elements).
#[test]
fn test_position_mod_even() {
    run_lxml_comparison_with_html("//ul/li[position() mod 2 = 0]", CUSTOM_HTML);
}

// ===== String functions: concat() =====

/// Select elements where concat of two attributes matches.
#[test]
fn test_concat_function() {
    run_lxml_comparison_with_html(
        "//td[concat(@class, '-extra') = 'c1-extra']",
        CUSTOM_HTML,
    );
}

/// concat() in a contains() predicate.
#[test]
fn test_concat_in_contains() {
    run_lxml_comparison_with_html("//li[contains(concat(' ', @class, ' '), ' item ')]", CUSTOM_HTML);
}

// ===== String functions: substring-before() / substring-after() =====

/// Select elements where substring-before class at space matches.
#[test]
fn test_substring_before() {
    run_lxml_comparison_with_html(
        "//li[substring-before(@class, ' ') = 'item']",
        CUSTOM_HTML,
    );
}

/// Select elements where substring-after class at space matches.
#[test]
fn test_substring_after() {
    run_lxml_comparison_with_html(
        "//li[substring-after(@class, 'item ') = 'first']",
        CUSTOM_HTML,
    );
}

// ===== Number functions: floor(), ceiling(), round() =====

/// floor() function in a predicate.
#[test]
fn test_floor_function() {
    run_lxml_comparison_with_html("//ul/li[floor(last() div 2) = 1]", CUSTOM_HTML);
}

/// ceiling() function in a predicate.
#[test]
fn test_ceiling_function() {
    run_lxml_comparison_with_html("//ul/li[ceiling(last() div 2) >= 2]", CUSTOM_HTML);
}

/// round() function in a predicate.
#[test]
fn test_round_function() {
    run_lxml_comparison_with_html("//ul/li[position() = round(1.5)]", CUSTOM_HTML);
}

// ===== Boolean functions: true(), false() =====

/// true() function as a predicate (selects all).
#[test]
fn test_true_function() {
    run_lxml_comparison_with_html("//li[true()]", CUSTOM_HTML);
}

/// not(false()) as a predicate (selects all).
#[test]
fn test_not_false_function() {
    run_lxml_comparison_with_html("//li[not(false())]", CUSTOM_HTML);
}

// ===== Node-set functions: name(), local-name() =====

/// Select elements where name() matches.
#[test]
fn test_name_function() {
    run_lxml_comparison_with_html("//*[name() = 'strong']", CUSTOM_HTML);
}

/// Select elements where local-name() matches.
#[test]
fn test_local_name_function() {
    run_lxml_comparison_with_html("//*[local-name() = 'em']", CUSTOM_HTML);
}

// ===== Double negation =====

/// Double not() — should be equivalent to boolean.
#[test]
fn test_double_negation() {
    run_lxml_comparison_with_html("//li[not(not(@class))]", CUSTOM_HTML);
}

// ===== Complex predicate combinations =====

/// and/or combined: select li with specific class combos.
#[test]
fn test_and_or_combined() {
    run_lxml_comparison_with_html(
        "//li[contains(@class, 'item') and (contains(@class, 'first') or contains(@class, 'last'))]",
        CUSTOM_HTML,
    );
}

/// Predicate with comparison and string function.
#[test]
fn test_predicate_mixed_functions() {
    run_lxml_comparison_with_html(
        "//td[string-length(@class) = 2 and starts-with(@class, 'c')]",
        CUSTOM_HTML,
    );
}

/// Negation combined with positional.
#[test]
fn test_not_with_positional() {
    run_lxml_comparison_with_html("//ul/li[not(position() = 1)]", CUSTOM_HTML);
}

// ===== Multiple union expressions =====

/// Union of three element types.
#[test]
fn test_triple_union() {
    run_lxml_count_comparison_with_html("//strong | //em | //p", CUSTOM_HTML);
}

/// Union with predicates on each operand.
#[test]
fn test_union_with_predicates() {
    run_lxml_count_comparison_with_html(
        "//li[@class='item first'] | //td[@class='c1']",
        CUSTOM_HTML,
    );
}

// ===== Complex multi-step navigation =====

/// Navigate up and then to a specific descendant.
#[test]
fn test_parent_then_descendant() {
    run_lxml_comparison_with_html("//strong/..//em", CUSTOM_HTML);
}

/// Navigate: descendant, then parent, then child of a different type.
#[test]
fn test_complex_navigation() {
    run_lxml_comparison_with_html("//div[@class='content']//strong/../em", CUSTOM_HTML);
}

/// Multi-step: ancestor then descendant.
#[test]
fn test_ancestor_then_descendant() {
    run_lxml_comparison_with_html(
        "//span[@class='deep']/ancestor::div[@id='a']//span",
        CUSTOM_HTML_DEEP,
    );
}

/// Navigate from table cell up to row then to sibling cell.
#[test]
fn test_table_cell_to_sibling() {
    run_lxml_comparison_with_html("//td[@class='c1']/../td[@class='c2']", CUSTOM_HTML);
}

// ===== Deeply nested predicates =====

/// Three levels of nested predicates.
#[test]
fn test_triple_nested_predicate() {
    run_lxml_comparison_with_html("//div[div[div[span]]]", CUSTOM_HTML_DEEP);
}

/// Nested predicate with attribute check at leaf.
#[test]
fn test_nested_predicate_attr_at_leaf() {
    run_lxml_comparison_with_html("//div[div[span[@class='deep']]]", CUSTOM_HTML_DEEP);
}

// ===== Wildcard patterns =====

/// Select all direct children of body using wildcard.
#[test]
fn test_wildcard_body_children() {
    run_lxml_comparison_with_html("/html/body/*", CUSTOM_HTML);
}

/// Wildcard at intermediate step.
#[test]
fn test_wildcard_intermediate_step() {
    run_lxml_comparison_with_html("//div[@id='root']/*/li", CUSTOM_HTML);
}

/// Double wildcard descent.
#[test]
fn test_double_wildcard() {
    run_lxml_count_comparison_with_html("//table//*", CUSTOM_HTML);
}

// ===== Attribute value comparisons =====

/// Less-than comparison on string-length.
#[test]
fn test_string_length_lt() {
    run_lxml_comparison_with_html("//span[string-length(@class) < 2]", CUSTOM_HTML);
}

/// Greater-or-equal on count.
#[test]
fn test_count_gte() {
    run_lxml_count_comparison_with_html("//div[count(*) >= 3]", CUSTOM_HTML);
}

/// Less-or-equal on position.
#[test]
fn test_position_lte() {
    run_lxml_comparison_with_html("//div[@class='siblings']/span[position() <= 2]", CUSTOM_HTML);
}

// ===== Combining axes with predicates =====

/// Following-sibling with attribute predicate.
#[test]
fn test_following_sibling_with_attr() {
    run_lxml_comparison_with_html(
        "//span[@class='a']/following-sibling::span[@class='c']",
        CUSTOM_HTML,
    );
}

/// Preceding-sibling with attribute predicate.
#[test]
fn test_preceding_sibling_with_attr() {
    run_lxml_comparison_with_html(
        "//span[@class='c']/preceding-sibling::span[@class='a']",
        CUSTOM_HTML,
    );
}

/// Ancestor with predicate.
#[test]
fn test_ancestor_with_predicate() {
    run_lxml_comparison_with_html("//span[@data-x]/ancestor::div[@id]", CUSTOM_HTML);
}

// ===== Complex real-world patterns on GitHub HTML =====

/// Select divs with role attribute via descendant of specific element.
#[test]
fn test_github_role_descendant() {
    run_lxml_count_comparison("//main//div[@role]");
}

/// Select anchors in nav with specific attribute.
#[test]
fn test_github_nav_anchors_with_attr() {
    run_lxml_comparison("//nav//a[@data-analytics-event]");
}

/// Multi-step: header -> nav -> ul -> li -> a.
#[test]
fn test_github_deep_nav_path() {
    run_lxml_comparison("//header//nav//ul//li//a");
}

/// Select elements with multiple attribute predicates.
#[test]
fn test_github_multi_attr_predicate() {
    run_lxml_count_comparison("//a[@href and @class and @data-analytics-event]");
}

/// Complex: anchors in nav that contain text.
#[test]
fn test_github_nav_anchors_with_text() {
    run_lxml_count_comparison("//nav//a[string-length(normalize-space(.)) > 0]");
}

/// Select divs with class containing multiple substrings.
#[test]
fn test_github_class_multi_contains() {
    run_lxml_comparison("//div[contains(@class, 'position') and contains(@class, 'relative')]");
}

/// Ancestor of img elements.
#[test]
fn test_github_img_ancestor() {
    run_lxml_comparison("//img[@alt]/ancestor::a");
}

/// Select elements at a specific depth from body.
#[test]
fn test_github_depth_path() {
    run_lxml_count_comparison("/html/body/div/div");
}

// ===== Custom HTML for numeric and edge-case patterns =====

static CUSTOM_HTML_NUMERIC: &str = r#"<html><body>
<div class="prices">
  <span class="price" data-value="10">$10</span>
  <span class="price" data-value="25">$25</span>
  <span class="price" data-value="5">$5</span>
  <span class="price" data-value="100">$100</span>
</div>
<div class="mixed">
  <p class="a b c">multi-class</p>
  <p class="x">single-class</p>
  <p>no-class</p>
  <p class="">empty-class</p>
</div>
<div class="empty-children">
  <div></div>
  <div><span>has child</span></div>
  <div></div>
</div>
</body></html>"#;

/// Select spans with numeric attribute comparison.
#[test]
fn test_numeric_attr_gt() {
    run_lxml_comparison_with_html("//span[@data-value > 10]", CUSTOM_HTML_NUMERIC);
}

/// Select spans with numeric attribute less-than.
#[test]
fn test_numeric_attr_lt() {
    run_lxml_comparison_with_html("//span[@data-value < 25]", CUSTOM_HTML_NUMERIC);
}

/// Select spans with numeric attribute equality.
#[test]
fn test_numeric_attr_eq() {
    run_lxml_comparison_with_html("//span[@data-value = 100]", CUSTOM_HTML_NUMERIC);
}

/// Select p elements that have a class attribute (including empty).
#[test]
fn test_has_class_including_empty() {
    run_lxml_comparison_with_html("//p[@class]", CUSTOM_HTML_NUMERIC);
}

/// Select p elements with non-empty class.
#[test]
fn test_non_empty_class() {
    run_lxml_comparison_with_html("//p[string-length(@class) > 0]", CUSTOM_HTML_NUMERIC);
}

/// Select p elements without class attribute.
#[test]
fn test_no_class_attr() {
    run_lxml_comparison_with_html("//p[not(@class)]", CUSTOM_HTML_NUMERIC);
}

/// Select empty divs (no element children).
#[test]
fn test_empty_divs() {
    run_lxml_comparison_with_html(
        "//div[@class='empty-children']/div[not(*)]",
        CUSTOM_HTML_NUMERIC,
    );
}

/// Select non-empty divs (with element children).
#[test]
fn test_non_empty_divs() {
    run_lxml_comparison_with_html(
        "//div[@class='empty-children']/div[*]",
        CUSTOM_HTML_NUMERIC,
    );
}

/// Predicate combining numeric comparison and string function.
#[test]
fn test_numeric_and_string_predicate() {
    run_lxml_comparison_with_html(
        "//span[@data-value > 5 and contains(., '$')]",
        CUSTOM_HTML_NUMERIC,
    );
}

/// sum() function on numeric attributes.
#[test]
fn test_sum_function() {
    run_lxml_comparison_with_html(
        "//div[sum(span/@data-value) > 50]",
        CUSTOM_HTML_NUMERIC,
    );
}

// ===== Multiple predicates (chained) =====

/// Three chained predicates.
#[test]
fn test_triple_chained_predicates() {
    run_lxml_comparison_with_html(
        "//span[@class][contains(@class, 'price')][@data-value]",
        CUSTOM_HTML_NUMERIC,
    );
}

/// Chained predicates: attribute existence + positional.
#[test]
fn test_chained_attr_then_positional() {
    run_lxml_comparison_with_html("//span[@class='price'][2]", CUSTOM_HTML_NUMERIC);
}

/// Chained predicates: positional + attribute.
#[test]
fn test_chained_positional_then_attr() {
    run_lxml_comparison_with_html("//span[2][@class='price']", CUSTOM_HTML_NUMERIC);
}

