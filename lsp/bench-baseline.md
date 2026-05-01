# LSP benchmark baseline

Captured with `cargo bench -- --quick` on a Windows 10 dev host
(unspecified hardware, foreground noise NOT suppressed). Numbers are
indicative — single-digit-percent regressions here are within noise.
For accurate local comparisons re-run `cargo bench` (without `--quick`)
and compare against your own freshly captured baseline.

## Methodology

- Generator: `benches/fixtures.rs` — deterministic synthesizer mixing
  plain pairs, dotted keys, typed scalars (`:i`, `:f`), raw markers
  (`::`), nested objects, arrays, multi-line raw blocks, and comments.
- Sizes: `small_1k` ≈ 1 KiB, `medium_50k` ≈ 50 KiB, `large_500k` ≈ 500 KiB.
- Criterion `--quick` profile: shorter measurement windows. Treat the
  numbers below as order-of-magnitude only.

## parse_for_diagnostics

| size       | median time | throughput |
|------------|-------------|------------|
| small_1k   | ~15.6 µs    | ~65 MiB/s  |
| medium_50k | ~1.05 ms    | ~47 MiB/s  |
| large_500k | ~7.8 ms     | ~62 MiB/s  |

## semantic_tokens

| size       | median time | throughput |
|------------|-------------|------------|
| small_1k   | ~6.3 µs     | ~163 MiB/s |
| medium_50k | ~244 µs     | ~200 MiB/s |
| large_500k | ~3.7 ms     | ~132 MiB/s |

## build_symbols (post-parse, time excludes parse)

| size       | median time | throughput  |
|------------|-------------|-------------|
| small_1k   | ~71 µs      | ~14 MiB/s   |
| medium_50k | ~101 ms     | ~494 KiB/s  |
| large_500k | ~11.0 s     | ~45 KiB/s   |

The cliff at medium/large is the `locate_key` full-text scan per top-
level key (super-linear with key count × text length). Optimization
target — out of scope for this baseline pass.

## Encoding hot paths

| bench                                  | median time |
|----------------------------------------|-------------|
| `byte_to_utf16` ascii_end              | ~78 ns      |
| `byte_to_utf16` ascii_mid              | ~42 ns      |
| `byte_to_utf16` cyrillic_end           | ~86 ns      |
| `byte_to_utf16` emoji_end              | ~70 ns      |
| `prefix_by_encoding` utf16_ascii_end   | ~96 ns      |
| `prefix_by_encoding` utf16_cyrillic_end| ~137 ns     |
| `prefix_by_encoding` utf16_emoji_mid   | ~48 ns      |
| `prefix_by_encoding` utf8_ascii_end    | ~3.6 ns     |
| `prefix_by_encoding` utf8_cyrillic_end | ~3.6 ns     |

UTF-8 negotiation is ~25–40× cheaper than UTF-16 on the same input —
expected, since the UTF-8 path is just a clamp + boundary-walk and the
UTF-16 path counts code units char-by-char.

## CI

Benches are NOT run in CI — they're a developer tool. Regressions
should be caught manually by re-running `cargo bench` and diffing
against this file (or a freshly captured local baseline).
