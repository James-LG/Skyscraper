use criterion::Criterion;
use skyscraper::html;

static HTML: &'static str = include_str!("../tests/samples/James-LG_Skyscraper.html");

pub fn benchmark_html_parse(c: &mut Criterion) {
    c.bench_function("html parse", |b| b.iter(|| html::parse(HTML)));
}
