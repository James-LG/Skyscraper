use criterion::{BenchmarkId, Criterion};
use skyscraper::{html, xpath};

static HTML: &str = include_str!("../tests/samples/James-LG_Skyscraper.html");

fn tree() -> xpath::XpathItemTree {
    html::parse(HTML).unwrap()
}

/// Benchmark different XPath axis traversals on a real HTML document.
pub fn benchmark_axes(c: &mut Criterion) {
    let tree = tree();

    let cases = [
        ("child", "//body/div"),
        ("descendant", "//body//div"),
        ("descendant-or-self", "//div/descendant-or-self::div"),
        ("self", "//div/self::div"),
        ("attribute", "//div/@class"),
        ("parent", "//a/.."),
        ("ancestor", "//a/ancestor::div"),
        ("ancestor-or-self", "//div/ancestor-or-self::div"),
        ("following-sibling", "//div/following-sibling::div"),
        ("preceding-sibling", "//div/preceding-sibling::div"),
    ];

    let mut group = c.benchmark_group("axes");
    for (name, expr) in cases {
        let xpath = xpath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("eval", name), &xpath, |b, xpath| {
            b.iter(|| xpath.apply(&tree).unwrap());
        });
    }
    group.finish();
}

/// Benchmark predicate evaluation: positional, attribute equality, and compound.
pub fn benchmark_predicates(c: &mut Criterion) {
    let tree = tree();

    let cases = [
        ("positional_first", "//div[1]"),
        ("positional_last", "//div[last()]"),
        ("attribute_eq", "//div[@class='repository-content']"),
        ("attribute_contains", "//div[contains(@class, 'Border')]"),
        ("compound_and", "//div[@class and @id]"),
        ("nested", "//div[div[@class]]"),
    ];

    let mut group = c.benchmark_group("predicates");
    for (name, expr) in cases {
        let xpath = xpath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("eval", name), &xpath, |b, xpath| {
            b.iter(|| xpath.apply(&tree).unwrap());
        });
    }
    group.finish();
}

/// Benchmark builtin XPath functions.
pub fn benchmark_functions(c: &mut Criterion) {
    let tree = tree();

    let cases = [
        ("count", "count(//div)"),
        ("string-length", "string-length(//title)"),
        ("concat", "concat('hello', ' ', 'world')"),
        ("contains", "contains('skyscraper', 'sky')"),
        ("not", "not(false())"),
        ("sum", "sum((1, 2, 3, 4, 5))"),
        ("string-join", "string-join(('a','b','c'), '-')"),
        ("substring", "substring('skyscraper', 4, 3)"),
        ("reverse", "reverse((1, 2, 3, 4, 5))"),
        ("name", "name(//div[1])"),
    ];

    let mut group = c.benchmark_group("functions");
    for (name, expr) in cases {
        let xpath = xpath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("eval", name), &xpath, |b, xpath| {
            b.iter(|| xpath.apply(&tree).unwrap());
        });
    }
    group.finish();
}

/// Benchmark XPath expression types: arithmetic, comparison, logical, conditional.
pub fn benchmark_expressions(c: &mut Criterion) {
    let tree = tree();

    let cases = [
        ("arithmetic", "3 * 4 + 2 div 1 - 1"),
        ("comparison_value", "count(//div) > 10"),
        ("logical_and_or", "true() and (false() or true())"),
        ("conditional", "if (count(//div) > 0) then 'yes' else 'no'"),
        ("range", "1 to 100"),
        ("for_expr", "for $x in (1, 2, 3) return $x * 2"),
        ("let_expr", "let $x := count(//div) return $x + 1"),
        ("quantified_some", "some $x in (1, 2, 3) satisfies $x > 2"),
    ];

    let mut group = c.benchmark_group("expressions");
    for (name, expr) in cases {
        let xpath = xpath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("eval", name), &xpath, |b, xpath| {
            b.iter(|| xpath.apply(&tree).unwrap());
        });
    }
    group.finish();
}

/// Benchmark real-world-like compound XPath queries.
pub fn benchmark_real_world(c: &mut Criterion) {
    let tree = tree();

    let cases = [
        (
            "links_in_nav",
            "//nav//a/@href",
        ),
        (
            "deep_text_select",
            "//div[@class='repository-content']//span/text()",
        ),
        (
            "multi_predicate",
            "//div[@class='BorderGrid-cell']/div[@class=' text-small']/a",
        ),
        (
            "ancestor_filter",
            "//a[ancestor::div[@class='BorderGrid-cell']]",
        ),
    ];

    let mut group = c.benchmark_group("real_world");
    for (name, expr) in cases {
        let xpath = xpath::parse(expr).unwrap();
        group.bench_with_input(BenchmarkId::new("eval", name), &xpath, |b, xpath| {
            b.iter(|| xpath.apply(&tree).unwrap());
        });
    }
    group.finish();
}
