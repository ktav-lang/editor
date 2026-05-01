//! Criterion benchmark: `semantic_tokens` on small/medium/large docs.
//! Runs on every `semanticTokens/full` request — editors send these
//! frequently while scrolling.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use ktav_lsp::semantic::semantic_tokens;

include!("fixtures.rs");

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_tokens");
    for (name, doc) in [
        ("small_1k", small()),
        ("medium_50k", medium()),
        ("large_500k", large()),
    ] {
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let t = semantic_tokens(black_box(&doc));
                black_box(t);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
