use criterion::{criterion_group, criterion_main};

mod html_benchmark;

use crate::html_benchmark::benchmark_html_parse;

criterion_group!(benches, benchmark_html_parse);
criterion_main!(benches);