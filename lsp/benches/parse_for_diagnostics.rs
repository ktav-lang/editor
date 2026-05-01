//! Criterion benchmark: `parse_for_diagnostics` on small/medium/large
//! synthesized Ktav documents. This is the hot path for `did_change`.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use ktav_lsp::diagnostics::parse_for_diagnostics;

include!("fixtures.rs");

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_for_diagnostics");
    for (name, doc) in [
        ("small_1k", small()),
        ("medium_50k", medium()),
        ("large_500k", large()),
    ] {
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let d = parse_for_diagnostics(black_box(&doc));
                black_box(d);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
