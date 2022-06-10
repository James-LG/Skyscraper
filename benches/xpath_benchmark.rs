use criterion::Criterion;
use skyscraper::xpath;

pub fn benchmark_xpath_parse(c: &mut Criterion) {
    c.bench_function("xpath parse", |b| {
        b.iter(|| {
            xpath::parse("//div[@class='BorderGrid-cell']/div[@class=' text-small']/a").unwrap();
        })
    });
}
