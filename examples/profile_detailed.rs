//! Detailed profiling harness that isolates different costs.
//! Run with: cargo run --example profile_detailed --release

use std::time::Instant;

static HTML: &str = include_str!("../tests/samples/James-LG_Skyscraper.html");

fn bench(label: &str, iterations: u32, f: impl Fn()) -> std::time::Duration {
    // Warm up
    for _ in 0..3 {
        f();
    }
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let total = start.elapsed();
    let avg = total / iterations;
    println!("  {}: {:?}", label, avg);
    avg
}

fn main() {
    let iterations = 30;

    println!("=== Baseline ===");
    let real_time = bench("Real document (344KB)", iterations, || {
        std::hint::black_box(skyscraper::html::parse(HTML).unwrap());
    });
    let chars_count = HTML.chars().count();
    println!(
        "  => {:.0} ns/char, {:.1} MB/s",
        real_time.as_nanos() as f64 / chars_count as f64,
        HTML.len() as f64 / real_time.as_secs_f64() / 1_000_000.0
    );

    // Isolate: chars().collect() cost (input preparation)
    println!("\n=== Input preparation ===");
    bench("chars().collect::<Vec<char>>()", iterations, || {
        let chars: Vec<char> = HTML.chars().collect();
        std::hint::black_box(&chars);
    });

    // Isolate: tag count vs text in the real document
    println!("\n=== Content analysis ===");
    let mut tag_count = 0u32;
    let mut attr_count = 0u32;
    let mut in_tag = false;
    let mut in_attr_value = false;
    let mut tag_chars = 0u64;
    let mut attr_value_chars = 0u64;
    let mut text_chars = 0u64;
    for c in HTML.chars() {
        if c == '<' {
            in_tag = true;
            tag_count += 1;
        }
        if in_tag {
            if c == '=' && !in_attr_value {
                attr_count += 1;
            }
            if c == '"' {
                in_attr_value = !in_attr_value;
            }
            if in_attr_value && c != '"' {
                attr_value_chars += 1;
            } else {
                tag_chars += 1;
            }
        } else {
            text_chars += 1;
        }
        if c == '>' {
            in_tag = false;
            in_attr_value = false;
        }
    }
    println!("  Tags: ~{}", tag_count);
    println!("  Attributes: ~{}", attr_count);
    println!("  Tag structure chars: {} ({:.1}%)", tag_chars, tag_chars as f64 / chars_count as f64 * 100.0);
    println!("  Attribute value chars: {} ({:.1}%)", attr_value_chars, attr_value_chars as f64 / chars_count as f64 * 100.0);
    println!("  Text chars: {} ({:.1}%)", text_chars, text_chars as f64 / chars_count as f64 * 100.0);

    // Isolate: per-component cost via synthetic documents
    println!("\n=== Synthetic isolation benchmarks ===");

    // 1. Pure text (no tags at all) - measures text tokenizer + tree builder text insertion
    let text_only = "x".repeat(344000);
    bench("344K text-only chars", iterations, || {
        std::hint::black_box(skyscraper::html::parse(&text_only).unwrap());
    });

    // 2. Many small tags, no attributes - measures tag tokenizer + tree builder element creation
    let tags_no_attrs: String = (0..5000).map(|_| "<div></div>").collect();
    let t2 = bench(&format!("5K empty divs ({}B)", tags_no_attrs.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&tags_no_attrs).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/tag", t2.as_nanos() as f64 / tags_no_attrs.len() as f64, t2.as_nanos() as f64 / 5000.0);

    // 3. Tags with 1 short attribute - measures attribute creation cost
    let tags_1attr: String = (0..5000).map(|_| "<div class=\"x\"></div>").collect();
    let t3 = bench(&format!("5K divs w/ 1 attr ({}B)", tags_1attr.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&tags_1attr).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/tag", t3.as_nanos() as f64 / tags_1attr.len() as f64, t3.as_nanos() as f64 / 5000.0);

    // 4. Tags with 5 attributes - measures multi-attribute + duplicate detection
    let tags_5attr: String = (0..2000).map(|_| "<div class=\"x\" id=\"y\" data-a=\"1\" data-b=\"2\" data-c=\"3\"></div>").collect();
    let t4 = bench(&format!("2K divs w/ 5 attrs ({}B)", tags_5attr.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&tags_5attr).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/tag", t4.as_nanos() as f64 / tags_5attr.len() as f64, t4.as_nanos() as f64 / 2000.0);

    // 5. Tags with long attribute values (like URLs) - measures attr value batching
    let long_val = "x".repeat(200);
    let tags_long_val: String = (0..2000).map(|_| format!("<a href=\"{}\"></a>", long_val)).collect();
    let t5 = bench(&format!("2K tags w/ 200-char attr ({}B)", tags_long_val.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&tags_long_val).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/tag", t5.as_nanos() as f64 / tags_long_val.len() as f64, t5.as_nanos() as f64 / 2000.0);

    // 6. Many unique tag names - measures tag name processing
    let unique_tags: String = (0..5000).map(|i| format!("<x{i}></x{i}>")).collect();
    let t6 = bench(&format!("5K unique tag names ({}B)", unique_tags.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&unique_tags).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/tag", t6.as_nanos() as f64 / unique_tags.len() as f64, t6.as_nanos() as f64 / 5000.0);

    // 7. Measure the emit overhead: same tags but with various attribute counts
    println!("\n=== Per-tag overhead scaling with attributes ===");
    for n_attrs in [0, 1, 2, 5, 10] {
        let attrs: String = (0..n_attrs).map(|i| format!(" a{}=\"v\"", i)).collect();
        let doc: String = (0..2000).map(|_| format!("<div{}></div>", attrs)).collect();
        let t = bench(&format!("2K divs × {} attrs ({}B)", n_attrs, doc.len()), iterations, || {
            std::hint::black_box(skyscraper::html::parse(&doc).unwrap());
        });
        println!("  => {:.0} ns/tag", t.as_nanos() as f64 / 2000.0);
    }

    // 8. Measure tree builder: deep nesting vs flat
    println!("\n=== Tree builder: nesting depth ===");
    for depth in [10, 100, 500, 1000] {
        let doc = format!("{}x{}", "<div>".repeat(depth), "</div>".repeat(depth));
        let t = bench(&format!("depth-{} ({}B)", depth, doc.len()), iterations, || {
            std::hint::black_box(skyscraper::html::parse(&doc).unwrap());
        });
        println!("  => {:.0} ns/char", t.as_nanos() as f64 / doc.len() as f64);
    }

    // 9. Measure scope-check overhead: many end tags that trigger has_an_element_in_scope
    println!("\n=== Scope checking ===");
    // Many different tag types that each need scope checking
    let scope_heavy: String = (0..1000).map(|_| "<p><span>x</span></p>").collect();
    let t9 = bench(&format!("1K p>span patterns ({}B)", scope_heavy.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&scope_heavy).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/pattern", t9.as_nanos() as f64 / scope_heavy.len() as f64, t9.as_nanos() as f64 / 1000.0);

    // 10. Measure element_in_scope cost when stack is deep
    println!("\n=== Scope checking with deep stack ===");
    let deep_scope = format!(
        "{}<p>x</p>{}",
        "<div>".repeat(100),
        "</div>".repeat(100)
    );
    let deep_scope_repeated: String = (0..500).map(|_| deep_scope.clone()).collect();
    let t10 = bench(&format!("500 × deep scope check ({}B)", deep_scope_repeated.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&deep_scope_repeated).unwrap());
    });
    println!("  => {:.0} ns/char", t10.as_nanos() as f64 / deep_scope_repeated.len() as f64);

    // 11. Measure character reference cost
    println!("\n=== Character reference overhead ===");

    // Attribute values with &quot; entities (like the real document)
    let quot_heavy: String = (0..500)
        .map(|_| "<div data-x=\"a&quot;b&quot;c&quot;d\"></div>")
        .collect();
    let t11a = bench(&format!("500 divs w/ &quot; attrs ({}B)", quot_heavy.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&quot_heavy).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/tag", t11a.as_nanos() as f64 / quot_heavy.len() as f64, t11a.as_nanos() as f64 / 500.0);

    // Same but without entities for comparison
    let no_entity: String = (0..500)
        .map(|_| "<div data-x=\"a-b-c-d-e-f-g-h-i\"></div>")
        .collect();
    let t11b = bench(&format!("500 divs w/ plain attrs ({}B)", no_entity.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&no_entity).unwrap());
    });
    println!("  => {:.0} ns/char, {:.0} ns/tag", t11b.as_nanos() as f64 / no_entity.len() as f64, t11b.as_nanos() as f64 / 500.0);

    // Many entities in a single attribute
    let many_entities: String = (0..100)
        .map(|_| format!("<div data-x=\"{}\"></div>", "&quot;x".repeat(50)))
        .collect();
    let t11c = bench(&format!("100 divs × 50 &quot; each ({}B)", many_entities.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&many_entities).unwrap());
    });
    let entity_count = 100 * 50;
    println!("  => {:.0} ns/char, {:.0} ns/entity", t11c.as_nanos() as f64 / many_entities.len() as f64, t11c.as_nanos() as f64 / entity_count as f64);

    // Text entities (outside attributes)
    let text_entities: String = (0..5000)
        .map(|_| "&quot;")
        .collect();
    let t11d = bench(&format!("5000 &quot; in text ({}B)", text_entities.len()), iterations, || {
        std::hint::black_box(skyscraper::html::parse(&text_entities).unwrap());
    });
    println!("  => {:.0} ns/entity", t11d.as_nanos() as f64 / 5000.0);
}
