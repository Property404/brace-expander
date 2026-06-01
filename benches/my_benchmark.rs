use brace_expander::BraceExpander;
use brace_expander_old::BraceExpander as OldBraceExpander;
use bracoxide::{self, bracoxidize};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    {
        let multiexpand = black_box("{1..10}{1..10}{1..10}{1..10}{1..10}");
        c.bench_function("Multiexpand", |b| {
            b.iter(|| BraceExpander::new().expand(multiexpand).unwrap())
        });
        c.bench_function("Multiexpand(old version)", |b| {
            b.iter(|| OldBraceExpander::new().expand(multiexpand).unwrap())
        });
        c.bench_function("Multiexpand(bracoxide cmp)", |b| {
            b.iter(|| bracoxidize(multiexpand).unwrap())
        });
    }

    {
        let large_numeric = black_box("{1..300000}");
        c.bench_function("Large Numeric", |b| {
            b.iter(|| BraceExpander::new().expand(large_numeric).unwrap())
        });
        c.bench_function("Large Numeric(old version)", |b| {
            b.iter(|| OldBraceExpander::new().expand(large_numeric).unwrap())
        });
        c.bench_function("Large Numeric(bracoxide cmp)", |b| {
            b.iter(|| bracoxidize(large_numeric).unwrap())
        });
    }

    {
        let simple = black_box("{a,b}");
        c.bench_function("Simple", |b| {
            b.iter(|| BraceExpander::new().expand(simple).unwrap())
        });
        c.bench_function("Simple(old version)", |b| {
            b.iter(|| OldBraceExpander::new().expand(simple).unwrap())
        });
        c.bench_function("Simple(bracoxide cmp)", |b| {
            b.iter(|| bracoxidize(simple).unwrap())
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
