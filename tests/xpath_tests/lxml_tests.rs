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
            assert_eq!(
                lxml_elem.attrib, skyscraper_elem.attrib,
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
    let html_text = GITHUB_HTML.to_string();

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    let lxml_elements = get_lxml_elements(xpath, html_text);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

/// Helper: run a count-only comparison test between skyscraper and lxml for a given XPath.
fn run_lxml_count_comparison(xpath: &str) {
    let html_text = GITHUB_HTML.to_string();

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    let lxml_output = get_lxml_output(xpath, html_text, true);
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

