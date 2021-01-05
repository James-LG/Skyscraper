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
            <meta name="description" content="A language empowering everyone to build reliable and efficient software.">
    
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
                            <div class="highlight"></div>
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
                            <section class="w-100 pv2 pv0-l mt4 mh5-l">
                                <h3 class="f2 f1-l">Reliability</h3>
                                <p class="f3 lh-copy">
                                  Rust’s rich type system and ownership model guarantee memory-safety
                                and thread-safety — enabling you to eliminate many classes of
                                bugs at compile-time.
                                </p>
                            </section>
                            <section class="w-100 pv2 pv0-l mt4">
                                <h3 class="f2 f1-l">Productivity</h3>
                                <p class="f3 lh-copy">
                                  Rust has great documentation, a friendly compiler with useful error
                                messages, and top-notch tooling — an integrated package manager
                                and build tool, smart multi-editor support with auto-completion and
                                type inspections, an auto-formatter, and more.
                                </p>
                            </section>
                        </div>
                    </div>
                </section>
            </main>
            <script src="./Rust Programming Language_files/languages.js.download"></script>
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
        Symbol::Text(String::from("en-US")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("head")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("title")),
        Symbol::TagClose,
        Symbol::Identifier(String::from("Rust")),
        Symbol::Identifier(String::from("Programming")),
        Symbol::Identifier(String::from("Language")),
        Symbol::EndTag(String::from("title")),
        Symbol::TagClose,
        Symbol::StartTag(String::from("meta")),
        Symbol::Identifier(String::from("name")),
        Symbol::AssignmentSign,
        Symbol::Text(String::from("viewport")),
        Symbol::Identifier(String::from("content")),
        Symbol::AssignmentSign,
        Symbol::Text(String::from("width=device-width,initial-scale=1.0")),
    ];
    
    for (e, r) in expected.into_iter().zip(result) {
        assert_eq!(e, r);
    }
}