//! Criterion benchmark: `build_symbols` on small/medium/large docs.
//! Document outline / `documentSymbol` request. We pre-parse outside
//! the timed region — this benchmark isolates the symbol-tree walk.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use ktav_lsp::symbols::build_symbols;

include!("fixtures.rs");

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_symbols");
    for (name, doc) in [
        ("small_1k", small()),
        ("medium_50k", medium()),
        ("large_500k", large()),
    ] {
        let value = ktav::parse(&doc).expect("fixture parses");
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let s = build_symbols(black_box(&value), black_box(&doc));
                black_box(s);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
