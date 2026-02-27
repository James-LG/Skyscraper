use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

static HTML: &'static str = include_str!("../samples/large.html");

#[test]
fn parse_should_return_document() {
    // arrange
    let text: String = HTML.parse().unwrap();

    // act
    let document = html::parse(&text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_doctype("html")
        .add_comment(" saved from url=(0038)https://github.com/James-LG/Skyscraper ")
        .add_element("html", |html| {
            html.add_attributes_str(vec![
                ("lang", "en"),
                ("data-color-mode", "dark"),
                ("data-light-theme", "light"),
                ("data-dark-theme", "dark"),
            ])
            .add_element("head", |head| {
                head.add_element("meta", |meta| {
                    meta.add_attributes_str(vec![
                        ("http-equiv", "Content-Type"),
                        ("content", "text/html; charset=UTF-8"),
                    ])
                })
                .add_element("link", |link| {
                    link.add_attributes_str(vec![
                        ("rel", "dns-prefetch"),
                        ("href", "https://github.githubassets.com/"),
                    ])
                })
                .add_element("script", |script| {
                    script.add_attributes_str(vec![
                        ("crossorigin", "anonymous"),
                        ("defer", "defer"),
                        ("type", "application/javascript"),
                        (
                            "src",
                            "./James-LG_Skyscraper_files/environment-2bf92300.js.download",
                        ),
                    ])
                })
                .add_element("title", |title| title.add_text("James-LG/Skyscraper"))
            })
            .add_element("body", |body| {
                body.add_element("div", |div| {
                    // div.position-relative.js-header-wrapper
                    div.add_attributes_str(vec![
                        ("class", "position-relative js-header-wrapper "),
                    ])
                    .add_element("a", |a| {
                        a.add_attributes_str(vec![
                            ("href", "https://github.com/James-LG/Skyscraper#start-of-content"),
                            ("class", "p-3 color-bg-accent-emphasis color-text-white show-on-focus js-skip-to-content"),
                        ])
                        .add_text("Skip to content")
                    })
                    .add_element("span", |span| {
                        span.add_attributes_str(vec![
                            ("data-view-component", "true"),
                            ("class", "progress-pjax-loader js-pjax-loader-bar Progress position-fixed width-full"),
                        ])
                        .add_element("span", |inner| {
                            inner.add_attributes_str(vec![
                                ("style", "width: 0%;"),
                                ("data-view-component", "true"),
                                ("class", "Progress-item progress-pjax-loader-bar color-bg-info-inverse"),
                            ])
                        })
                    })
                    .add_element("script", |script| {
                        script.add_attributes_str(vec![
                            ("crossorigin", "anonymous"),
                            ("defer", "defer"),
                            ("integrity", "sha512-18ADRS+iEo2KaRjmRMSvy59l6oUJtsMgahabrGMf45z3P3eLyMrmL+SVo7GMGifQdat4j82JSIRy8bkzkCFSzg=="),
                            ("type", "application/javascript"),
                            ("src", "./James-LG_Skyscraper_files/command-palette-d7c00345.js.download"),
                        ])
                    })
                    .add_element("header", |header| {
                        header.add_attributes_str(vec![
                            ("class", "Header js-details-container Details px-3 px-md-4 px-lg-5 flex-wrap flex-md-nowrap"),
                            ("role", "banner"),
                        ])
                        .add_element("div", |div| {
                            div.add_attribute_str("class", "Header-item position-relative mr-0 d-none d-md-flex")
                            .add_element("details", |details| {
                                details.add_attributes_str(vec![
                                    ("class", "details-overlay details-reset js-feature-preview-indicator-container"),
                                    ("data-feature-preview-indicator-src", "/users/James-LG/feature_preview/indicator_check"),
                                ])
                                .add_element("summary", |summary| {
                                    summary.add_attributes_str(vec![
                                        ("class", "Header-link"),
                                        ("aria-label", "View profile and more"),
                                        ("data-hydro-click", r#"{"event_type":"analytics.event","payload":{"category":"Header","action":"show menu","label":"icon:avatar","originating_url":"https://github.com/James-LG/Skyscraper","user_id":1709432}}"#),
                                        ("data-hydro-click-hmac", "129bfbd34deeb4d67d1239fb6817a2645147d2ec7b74b16fd4c8ab86fca17dde"),
                                        ("data-analytics-event", r#"{"category":"Header","action":"show menu","label":"icon:avatar"}"#),
                                        ("aria-haspopup", "menu"),
                                        ("role", "button"),
                                    ])
                                    .add_element("img", |img| {
                                        img.add_attributes_str(vec![
                                            ("src", "./James-LG_Skyscraper_files/1709432"),
                                            ("alt", "@James-LG"),
                                            ("size", "20"),
                                            ("height", "20"),
                                            ("width", "20"),
                                            ("data-view-component", "true"),
                                            ("class", "avatar avatar-small circle"),
                                        ])
                                    })
                                    .add_element("span", |span| {
                                        span.add_attributes_str(vec![
                                            ("class", "feature-preview-indicator js-feature-preview-indicator"),
                                            ("style", "top: 1px;"),
                                            ("hidden", ""),
                                        ])
                                    })
                                    .add_element("span", |span| {
                                        span.add_attribute_str("class", "dropdown-caret")
                                    })
                                })
                                .add_element("details-menu", |dm| {
                                    dm.add_attributes_str(vec![
                                        ("class", "dropdown-menu dropdown-menu-sw"),
                                        ("style", "width: 180px"),
                                        ("src", "/users/1709432/menu"),
                                        ("preload", ""),
                                        ("role", "menu"),
                                    ])
                                    .add_element("include-fragment", |incfrag| {
                                        incfrag.add_element("p", |p| {
                                            p.add_attributes_str(vec![
                                                ("class", "text-center mt-3"),
                                                ("data-hide-on-error", ""),
                                            ])
                                            .add_element("span", |span| {
                                                span.add_attribute_str("role", "status")
                                                .add_element("span", |inner| {
                                                    inner.add_attribute_str("class", "sr-only")
                                                    .add_text("Loading")
                                                })
                                                .add_element("svg", |svg| {
                                                    svg.add_attributes_str(vec![
                                                        ("style", "box-sizing: content-box; color: var(--color-icon-primary);"),
                                                        ("width", "32"),
                                                        ("height", "32"),
                                                        ("viewBox", "0 0 16 16"),
                                                        ("fill", "none"),
                                                        ("data-view-component", "true"),
                                                        ("class", "anim-rotate"),
                                                    ])
                                                    .add_element("circle", |circle| {
                                                        circle.add_attributes_str(vec![
                                                            ("cx", "8"),
                                                            ("cy", "8"),
                                                            ("r", "7"),
                                                            ("stroke", "currentColor"),
                                                            ("stroke-opacity", "0.25"),
                                                            ("stroke-width", "2"),
                                                            ("vector-effect", "non-scaling-stroke"),
                                                        ])
                                                    })
                                                    .add_element("path", |path| {
                                                        path.add_attributes_str(vec![
                                                            ("d", "M15 8a7.002 7.002 0 00-7-7"),
                                                            ("stroke", "currentColor"),
                                                            ("stroke-width", "2"),
                                                            ("stroke-linecap", "round"),
                                                            ("vector-effect", "non-scaling-stroke"),
                                                        ])
                                                    })
                                                })
                                            })
                                        })
                                        .add_element("p", |p| {
                                            p.add_attributes_str(vec![
                                                ("class", "ml-1 mb-2 mt-2 color-fg-default"),
                                                ("data-show-on-error", ""),
                                            ])
                                            .add_element("svg", |svg| {
                                                svg.add_attributes_str(vec![
                                                    ("aria-hidden", "true"),
                                                    ("height", "16"),
                                                    ("viewBox", "0 0 16 16"),
                                                    ("version", "1.1"),
                                                    ("width", "16"),
                                                    ("data-view-component", "true"),
                                                    ("class", "octicon octicon-alert"),
                                                ])
                                                .add_element("path", |path| {
                                                    path.add_attributes_str(vec![
                                                        ("fill-rule", "evenodd"),
                                                        ("d", "M8.22 1.754a.25.25 0 00-.44 0L1.698 13.132a.25.25 0 00.22.368h12.164a.25.25 0 00.22-.368L8.22 1.754zm-1.763-.707c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0114.082 15H1.918a1.75 1.75 0 01-1.543-2.575L6.457 1.047zM9 11a1 1 0 11-2 0 1 1 0 012 0zm-.25-5.25a.75.75 0 00-1.5 0v2.5a.75.75 0 001.5 0v-2.5z"),
                                                    ])
                                                })
                                            })
                                            .add_text("\n                Sorry, something went wrong.\n              ")
                                        })
                                    })
                                })
                            })
                        })
                    })
                })
                .add_element("div", |div| {
                    div.add_attributes_str(vec![
                        ("id", "start-of-content"),
                        ("class", "show-on-focus"),
                    ])
                })
                .add_element("include-fragment", |incfrag| {
                    incfrag.add_attributes_str(vec![
                        ("class", "js-notification-shelf-include-fragment"),
                        ("data-base-src", "https://github.com/notifications/beta/shelf"),
                    ])
                })
                .add_element("style", |style| {
                    style.add_text("\n    .user-mention[href$=\"/James-LG\"] {\n      color: var(--color-user-mention-fg);\n      background-color: var(--color-user-mention-bg);\n      border-radius: 2px;\n      margin-left: -2px;\n      margin-right: -2px;\n      padding: 0 2px;\n    }\n  ")
                })
                .add_element("div", |div| {
                    div.add_attributes_str(vec![
                        ("aria-live", "polite"),
                        ("class", "sr-only"),
                    ])
                })
            })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

/// Diagnostic test: find the first line where the round-trip output diverges.
#[test]
fn parse_roundtrip_divergence_diagnostic() {
    let text: String = HTML.parse().unwrap();
    let document = html::parse(&text).unwrap();
    let displayed = document.to_string();

    let text_lines: Vec<&str> = text.lines().collect();
    let disp_lines: Vec<&str> = displayed.lines().collect();

    for i in 0..std::cmp::min(text_lines.len(), disp_lines.len()) {
        if text_lines[i] != disp_lines[i] {
            let start = i.saturating_sub(3);
            let end = std::cmp::min(i + 5, std::cmp::min(text_lines.len(), disp_lines.len()));
            eprintln!("First difference at line {} (1-indexed):", i + 1);
            eprintln!("--- Expected (original) ---");
            for j in start..end {
                let marker = if j == i { ">>>" } else { "   " };
                eprintln!("{} {:>5}: {}", marker, j + 1, text_lines[j]);
            }
            eprintln!("--- Got (parsed output) ---");
            for j in start..end {
                let marker = if j == i { ">>>" } else { "   " };
                if j < disp_lines.len() {
                    eprintln!("{} {:>5}: {}", marker, j + 1, disp_lines[j]);
                }
            }
            panic!(
                "Round-trip divergence at line {}: expected {:?}, got {:?}",
                i + 1,
                text_lines[i],
                disp_lines[i]
            );
        }
    }

    if text_lines.len() != disp_lines.len() {
        panic!(
            "Line count differs: expected {} got {}",
            text_lines.len(),
            disp_lines.len()
        );
    }
}
