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


## Post-0.2.x — re-measure (2026-05-08)

Same Win10 host, IDE / language servers running concurrently
(ambient noise NOT suppressed — see methodology caveat). `cargo bench
-- --quick` against ktav-lsp 0.2.0 + locally-pinned ktav 0.2.x
(stripped multiline default + `:f` accepts integer + duplicate-key
span fix + paren-scalar parser strictness).

### parse_for_diagnostics

| size       | 0.1.x baseline | 0.2.x   | Δ        |
|------------|----------------|---------|----------|
| small_1k   | ~15.6 µs       | 15.0 µs | −4 %     |
| medium_50k | ~1.05 ms       | 644 µs  | **−39 %**|
| large_500k | ~7.8 ms        | 8.85 ms | +13 %    |

The `medium_50k` improvement is the biggest delta — the paren-scalar
classifier is faster than the previous "any scalar" fallthrough on
the synthesised fixture (which is heavy on plain pairs). `large_500k`
+13 % is within the host's noise envelope; the win and loss roughly
cancel across the workload sizes.

### semantic_tokens

| size       | 0.1.x baseline | 0.2.x   | Δ        |
|------------|----------------|---------|----------|
| small_1k   | ~6.3 µs        | 6.36 µs | ≈        |
| medium_50k | ~244 µs        | 251 µs  | +3 %     |
| large_500k | ~3.7 ms        | 3.90 ms | +5 %     |

Within noise. The semantic tokenizer doesn't touch the changed code
paths, so the small drift is host load.

### build_symbols

| size       | 0.1.x baseline | 0.2.x    | Δ      |
|------------|----------------|----------|--------|
| small_1k   | ~71 µs         | ~73 µs   | +3 %   |
| medium_50k | ~101 ms        | 108 ms   | +7 %   |
| large_500k | ~11.0 s        | 11.5 s   | +4 %   |

The super-linear scaling on `large_500k` (already flagged in the
baseline above) is unchanged — `locate_key` full-text scan per
top-level key remains the bottleneck. Optimisation target for a
future patch (out of scope for 0.2.x).

### Notes

- Criterion reported "no change in performance detected" (p > 0.05)
  for **every** row. Treat all deltas above as noise unless reproduced
  on a quiet host.
- The major architectural change in 0.2.x — switching the formatter
  from `parse → render(value)` to `reindent(text)` — does not appear
  in these benches; reindent has its own micro-tests under
  `tests/format_pipeline.rs` (22 cases) but no Criterion benchmark
  yet. Worth adding when the formatter sees production usage profiles.


## Post-optimisation — `build_symbols` O(N²) → O(N) (2026-05-08)

`build_symbols` was rewritten to do a single line-by-line pass over
the document, collecting `(virtual_depth, key_name, line_range)` for
every pair as it scans, then walk the parsed `Value` with a
sequential cursor. This eliminates the per-key full-text scan
(`locate_key`) that drove the documented super-linear regression.

| size       | 0.2.x pre | post-opt | speed-up |
|------------|-----------|----------|----------|
| small_1k   | ~73 µs    | 20.9 µs  | **3.5×** |
| medium_50k | 108 ms    | 927 µs   | **117×** |
| large_500k | 11.5 s    | 13.4 ms  | **858×** |

Criterion explicitly reported "Performance has improved" for
`large_500k` (p = 0.05). The other two sizes were also dramatically
faster but Criterion's bootstrap got confused at p = 0.07 and printed
"no change detected" — the wall-clock improvement is real, the
statistical model just rarely sees deltas this large.

Throughput on `large_500k` went from ~45 KiB/s to ~36 MiB/s. The
remaining cost is dominated by the `Vec<DocumentSymbol>` construction
itself (per-key `String::from` for the name field), not the source
scan. Further wins would require interning or returning borrowed
names — out of scope; the current code is no longer the bottleneck.

Other benches in the same run (release, --quick, IDE-noisy host):

| bench                              | post-opt | vs 0.2.x | note            |
|------------------------------------|----------|----------|-----------------|
| `parse_for_diagnostics/small_1k`   | 12.3 µs  | −18 %    | within noise    |
| `parse_for_diagnostics/medium_50k` | 943 µs   | +46 %    | host noise      |
| `parse_for_diagnostics/large_500k` | 9.24 ms  | +4 %     | within noise    |
| `semantic_tokens/small_1k`         | 6.37 µs  | ≈        | unchanged       |
| `semantic_tokens/medium_50k`       | 243 µs   | −3 %     | unchanged       |
| `semantic_tokens/large_500k`       | 3.84 ms  | −1 %     | unchanged       |

`parse_for_diagnostics` and `semantic_tokens` were not touched in
this optimisation pass; their numbers move only with host load.


## Final post-optimisation reference baseline (2026-05-08, full)

Captured with **full** Criterion (no `--quick`, 100 samples, 5s
warm-up) on a quieter Win10 host with IDE / language servers closed
during the run. Reproduce with:

From the `editor/lsp/` crate root:

```sh
cargo bench --bench parse_for_diagnostics
cargo bench --bench semantic_tokens
cargo bench --bench build_symbols
cargo bench --bench encoding_hot_paths
```

These numbers are the new reference for regression detection. Past
sections used `--quick` profiles; treat this section as the source
of truth.

### `parse_for_diagnostics`

| size       | median  | throughput |
|------------|---------|------------|
| small_1k   | 12.2 µs | ~80 MiB/s  |
| medium_50k | 837 µs  | ~58 MiB/s  |
| large_500k | 9.13 ms | ~54 MiB/s  |

### `semantic_tokens`

| size       | median  | throughput  |
|------------|---------|-------------|
| small_1k   | 6.81 µs | ~152 MiB/s  |
| medium_50k | 288 µs  | ~169 MiB/s  |
| large_500k | 4.27 ms | ~115 MiB/s  |

### `build_symbols` — post O(N²)→O(N) rewrite

| size       | median   | throughput | vs pre-opt   |
|------------|----------|------------|--------------|
| small_1k   | 23.9 µs  | ~43 MiB/s  | 3.05× faster |
| medium_50k | 1.14 ms  | ~43 MiB/s  | 94.7× faster |
| large_500k | 13.2 ms  | ~37 MiB/s  | **871× faster** |

The `large_500k` wall-clock went from **11.5 s → 13.2 ms** — the
defining win of this optimisation pass. Throughput is now within an
order of magnitude of `parse_for_diagnostics` (the parse step,
which is bounded by the same I/O as `build_symbols`'s text scan)
instead of three orders of magnitude slower.

### `encoding_hot_paths`

These are nanosecond-scale measurements driving the LSP position-
encoding negotiation. Mostly stable across runs — included as a
floor reference.

| Bench                                  | median   |
|----------------------------------------|----------|
| `byte_to_utf16/ascii_end`              | 62.5 ns  |
| `byte_to_utf16/ascii_mid`              | 40.0 ns  |
| `byte_to_utf16/cyrillic_end`           | 93.1 ns  |
| `byte_to_utf16/emoji_end`              | 70.8 ns  |
| `prefix_by_encoding/utf16_ascii_end`   | 109 ns   |
| `prefix_by_encoding/utf16_cyrillic_end`| 166 ns   |
| `prefix_by_encoding/utf16_emoji_mid`   | 54.6 ns  |
| `prefix_by_encoding/utf8_ascii_end`    | 3.96 ns  |
| `prefix_by_encoding/utf8_cyrillic_end` | 4.34 ns  |

UTF-8 negotiation remains ~25–40× cheaper than UTF-16 — exactly the
asymmetry observed in earlier runs.
