//! Quick profiling harness for the HTML parser.
//! Run with: cargo run --example profile_html --release

use std::time::Instant;

static HTML: &str = include_str!("../tests/samples/James-LG_Skyscraper.html");

fn main() {
    // Warm up
    for _ in 0..5 {
        let _ = skyscraper::html::parse(HTML);
    }

    let iterations = 30;

    // Overall parse
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = skyscraper::html::parse(HTML);
    }
    let total = start.elapsed();
    let avg = total / iterations;
    println!("=== Overall ===");
    println!("Average parse time: {:?}", avg);
    println!(
        "HTML size: {} bytes, {} chars",
        HTML.len(),
        HTML.chars().count()
    );

    // Phase 1: chars().collect() (input preparation)
    let start = Instant::now();
    for _ in 0..iterations {
        let chars: Vec<char> = HTML.chars().collect();
        std::hint::black_box(&chars);
    }
    let t = start.elapsed() / iterations;
    println!("\n=== Phase breakdown ===");
    println!("1. chars().collect(): {:?}", t);

    // Phase 2: Just tokenize (count tokens without tree building)
    // We can't easily separate these, but we can measure
    // text-heavy vs tag-heavy parsing by using different inputs.

    // Measure a text-heavy document (mostly character tokens)
    let text_heavy = "x".repeat(344000);
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = skyscraper::html::parse(&text_heavy);
    }
    let t_text = start.elapsed() / iterations;
    println!("2. Parse 344K text-only: {:?}", t_text);

    // Measure a tag-heavy document (many small tags, little text)
    let tag_heavy = "<div><span></span></div>".repeat(2000);
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = skyscraper::html::parse(&tag_heavy);
    }
    let t_tag = start.elapsed() / iterations;
    println!(
        "3. Parse 2K div/span pairs ({}B): {:?}",
        tag_heavy.len(),
        t_tag
    );

    // Measure attribute-heavy document
    let attr_heavy: String = (0..1000)
        .map(|i| format!("<div class=\"c{}\" id=\"i{}\" data-x=\"{}\"></div>", i, i, i))
        .collect();
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = skyscraper::html::parse(&attr_heavy);
    }
    let t_attr = start.elapsed() / iterations;
    println!(
        "4. Parse 1K attr-heavy divs ({}B): {:?}",
        attr_heavy.len(),
        t_attr
    );

    // Measure deep nesting
    let depth = 500;
    let deep = format!(
        "{}x{}",
        "<div>".repeat(depth),
        "</div>".repeat(depth)
    );
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = skyscraper::html::parse(&deep);
    }
    let t_deep = start.elapsed() / iterations;
    println!(
        "5. Parse depth-{} nesting ({}B): {:?}",
        depth,
        deep.len(),
        t_deep
    );

    // Measure formatting elements (triggers reconstruct_active_formatting)
    let fmt_heavy: String = (0..500)
        .map(|_| "<b><i><u>text</u></i></b>")
        .collect::<String>();
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = skyscraper::html::parse(&fmt_heavy);
    }
    let t_fmt = start.elapsed() / iterations;
    println!(
        "6. Parse 500 b/i/u nests ({}B): {:?}",
        fmt_heavy.len(),
        t_fmt
    );

    // Now let's profile specific aspects of the real document
    println!("\n=== Real document analysis ===");

    // Count different character types
    let mut tag_chars = 0u64;
    let mut text_chars = 0u64;
    let mut in_tag = false;
    for c in HTML.chars() {
        if c == '<' {
            in_tag = true;
        }
        if in_tag {
            tag_chars += 1;
        } else {
            text_chars += 1;
        }
        if c == '>' {
            in_tag = false;
        }
    }
    println!(
        "Characters in tags: {} ({:.1}%)",
        tag_chars,
        tag_chars as f64 / HTML.len() as f64 * 100.0
    );
    println!(
        "Characters in text: {} ({:.1}%)",
        text_chars,
        text_chars as f64 / HTML.len() as f64 * 100.0
    );

    // Measure with pre-allocated arena hint
    println!("\n=== Throughput ===");
    println!(
        "Overall: {:.1} MB/s",
        HTML.len() as f64 / avg.as_secs_f64() / 1_000_000.0
    );
    println!(
        "Per char: {:.0} ns/char",
        avg.as_nanos() as f64 / HTML.chars().count() as f64
    );
}
