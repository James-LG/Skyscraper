use criterion::{criterion_group, criterion_main};

mod html_benchmark;
mod xpath_benchmark;
mod xpath_eval_benchmark;

use crate::html_benchmark::*;
use crate::xpath_benchmark::*;
use crate::xpath_eval_benchmark::*;

criterion_group!(
    benches,
    benchmark_html_parse,
    benchmark_xpath_parse,
    benchmark_axes,
    benchmark_predicates,
    benchmark_functions,
    benchmark_expressions,
    benchmark_real_world,
);
criterion_main!(benches);
