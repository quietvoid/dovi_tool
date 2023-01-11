use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::parsing::parse_rpus,
    benchmarks::rewriting::rewrite_rpus
}
