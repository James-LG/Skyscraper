use rxpath_lexer::{lex, Symbol};

#[test]
fn lex_should_work_with_html() {
    // arrange
    let html = r###"<!DOCTYPE html>
    <!-- saved from url=(0026)https://www.rust-lang.org/ -->
    <html lang="en-US">
        <head>
            <title>Rust Programming Language</title>
            <meta name="viewport" content="width=device-width,initial-scale=1.0">
    
            <!-- Twitter card -->
            <meta name="twitter:card" content="summary">
        </head>
        <body>
            <main>
                <section id="language-values" class="green">
                    <div class="w-100 mw-none ph3 mw8-m mw9-l center f3">
                        <header class="pb0">
                            <h2>
                            Why Rust?
                            </h2>
                        </header>
                        <div class="flex-none flex-l">
                            <section class="w-100 pv2 pv0-l mt4">
                                <h3 class="f2 f1-l">Performance</h3>
                                <p class="f3 lh-copy">
                                  Rust is blazingly fast and memory-efficient: with no runtime or
                                garbage collector, it can power performance-critical services, run on
                                embedded devices, and easily integrate with other languages.
                                </p>
                            </section>
                        </div>
                    </div>
                </section>
            </main>
            <script src="./Rust Programming Language_files/languages.js.download"/>
        </body>
    </html>"###;

    // act
    let result = lex(html).unwrap();

    // assert
    let expected = vec![
        Symbol::StartTag(String::from("!DOCTYPE")),
        Symbol::Identifier(String::from("html")),
        Symbol::TagClose,
        Symbol::Comment(String::from(" saved from url=(0026)https://www.rust-lang.org/ ")),
        Symbol::StartTag(String::from("html")),
        Symbol::Identifier(String::from("lang")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("en-US")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("head")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("title")),
        Symbol::TagClose,
        Symbol::Text(String::from("Rust Programming Language")),
        Symbol::EndTag(String::from("title")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("meta")),
        Symbol::Identifier(String::from("name")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("viewport")),
        Symbol::Identifier(String::from("content")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("width=device-width,initial-scale=1.0")),
        Symbol::TagClose,
        Symbol::Comment(String::from(" Twitter card ")),
        Symbol::StartTag(String::from("meta")),
        Symbol::Identifier(String::from("name")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("twitter:card")),
        Symbol::Identifier(String::from("content")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("summary")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("head")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("body")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("main")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("section")),
        Symbol::Identifier(String::from("id")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("language-values")),
        Symbol::Identifier(String::from("class")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("green")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("div")),
        Symbol::Identifier(String::from("class")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("w-100 mw-none ph3 mw8-m mw9-l center f3")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("header")),
        Symbol::Identifier(String::from("class")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("pb0")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("h2")),
        Symbol::TagClose,
        Symbol::Text(String::from(r#"
                            Why Rust?
                            "#)),
        Symbol::EndTag(String::from("h2")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("header")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("div")),
        Symbol::Identifier(String::from("class")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("flex-none flex-l")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("section")),
        Symbol::Identifier(String::from("class")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("w-100 pv2 pv0-l mt4")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("h3")),
        Symbol::Identifier(String::from("class")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("f2 f1-l")),
        Symbol::TagClose,
        Symbol::Text(String::from("Performance")),
        Symbol::EndTag(String::from("h3")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("p")),
        Symbol::Identifier(String::from("class")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("f3 lh-copy")),
        Symbol::TagClose,
        Symbol::Text(String::from(r#"
                                  Rust is blazingly fast and memory-efficient: with no runtime or
                                garbage collector, it can power performance-critical services, run on
                                embedded devices, and easily integrate with other languages.
                                "#)),
        Symbol::EndTag(String::from("p")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("section")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("div")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("div")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("section")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("main")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("script")),
        Symbol::Identifier(String::from("src")),
        Symbol::AssignmentSign,
        Symbol::Literal(String::from("./Rust Programming Language_files/languages.js.download")),
        Symbol::TagCloseAndEnd,
        Symbol::EndTag(String::from("body")),
        Symbol::TagClose,
        Symbol::EndTag(String::from("html")),
        Symbol::TagClose,
    ];
    
    // looping makes debugging much easier than just asserting the entire vectors are equal
    for (e, r) in expected.into_iter().zip(result) {
        assert_eq!(e, r);
    }
}