use criterion::{criterion_group, criterion_main};

mod html_benchmark;
mod xpath_benchmark;

use crate::html_benchmark::*;
use crate::xpath_benchmark::*;

criterion_group!(benches, benchmark_html_parse, benchmark_xpath_parse);
criterion_main!(benches);
