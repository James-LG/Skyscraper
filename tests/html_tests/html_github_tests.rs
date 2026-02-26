use skyscraper::html;

static HTML: &'static str = include_str!("../samples/James-LG_Skyscraper.html");

#[test]
fn parse_should_return_document() {
    // arrange
    let text: String = HTML.parse().unwrap();

    // act
    let document = html::parse(&text).unwrap();

    // assert
    let displayed_document = document.to_string();

    assert_eq!(displayed_document, text);
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
