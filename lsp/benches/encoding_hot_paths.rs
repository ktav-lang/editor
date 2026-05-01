//! Criterion benchmark: `byte_to_utf16` and `prefix_by_encoding` —
//! the per-keystroke hot paths under UTF-16 negotiation.
//!
//! These run on every column conversion in diagnostic / semantic-token
//! / completion handlers. Inputs cover ASCII, Cyrillic (BMP, 2-byte
//! UTF-8), and an emoji (4-byte UTF-8 / surrogate pair) so regressions
//! that quietly slow down non-ASCII paths surface here.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ktav_lsp::server::PositionEncoding;
use ktav_lsp::tokens::{byte_to_utf16, prefix_by_encoding};

fn bench_byte_to_utf16(c: &mut Criterion) {
    let mut group = c.benchmark_group("byte_to_utf16");

    let ascii = "name: alice and bob and the rest of the team";
    let cyrillic = "имя: значение из конфигурации сервиса для теста";
    let emoji = "greeting: hello 😀 and 🚀 and 🎉 to everyone here!";

    group.bench_function("ascii_end", |b| {
        b.iter(|| black_box(byte_to_utf16(black_box(ascii), black_box(ascii.len()))))
    });
    group.bench_function("ascii_mid", |b| {
        b.iter(|| black_box(byte_to_utf16(black_box(ascii), black_box(ascii.len() / 2))))
    });
    group.bench_function("cyrillic_end", |b| {
        b.iter(|| {
            black_box(byte_to_utf16(
                black_box(cyrillic),
                black_box(cyrillic.len()),
            ))
        })
    });
    group.bench_function("emoji_end", |b| {
        b.iter(|| black_box(byte_to_utf16(black_box(emoji), black_box(emoji.len()))))
    });

    group.finish();
}

fn bench_prefix_by_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("prefix_by_encoding");

    let ascii = "name: alice and bob and the rest of the team";
    let cyrillic = "имя: значение из конфигурации сервиса для теста";
    let emoji = "greeting: hello 😀 and 🚀 and 🎉 to everyone here!";

    // UTF-16 is the LSP default — most clients negotiate this.
    group.bench_function("utf16_ascii_end", |b| {
        let n = ascii.encode_utf16().count() as u32;
        b.iter(|| {
            black_box(prefix_by_encoding(
                black_box(ascii),
                black_box(n),
                PositionEncoding::Utf16,
            ))
        })
    });
    group.bench_function("utf16_cyrillic_end", |b| {
        let n = cyrillic.encode_utf16().count() as u32;
        b.iter(|| {
            black_box(prefix_by_encoding(
                black_box(cyrillic),
                black_box(n),
                PositionEncoding::Utf16,
            ))
        })
    });
    group.bench_function("utf16_emoji_mid", |b| {
        // Stop just before the first emoji.
        let n = "greeting: hello ".encode_utf16().count() as u32;
        b.iter(|| {
            black_box(prefix_by_encoding(
                black_box(emoji),
                black_box(n),
                PositionEncoding::Utf16,
            ))
        })
    });

    // UTF-8 path: cheaper; pin a baseline for it too.
    group.bench_function("utf8_ascii_end", |b| {
        let n = ascii.len() as u32;
        b.iter(|| {
            black_box(prefix_by_encoding(
                black_box(ascii),
                black_box(n),
                PositionEncoding::Utf8,
            ))
        })
    });
    group.bench_function("utf8_cyrillic_end", |b| {
        let n = cyrillic.len() as u32;
        b.iter(|| {
            black_box(prefix_by_encoding(
                black_box(cyrillic),
                black_box(n),
                PositionEncoding::Utf8,
            ))
        })
    });

    group.finish();
}

criterion_group!(benches, bench_byte_to_utf16, bench_prefix_by_encoding);
criterion_main!(benches);
