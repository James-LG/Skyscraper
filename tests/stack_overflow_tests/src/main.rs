fn main() {
    // arrange
    let xpath_text = r###"//head//script[not(@src) and not(contains(., "gtag"))]"###;

    // act
    let xpath = skyscraper::xpath::parse(xpath_text).unwrap();

    // assert
    assert_eq!(xpath.to_string(), xpath_text);
}
