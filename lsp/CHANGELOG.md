# Changelog — `ktav-lsp`

All notable changes to the `ktav-lsp` crate are documented here. The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate adheres to [Semantic Versioning](https://semver.org/) with
the Cargo convention that a minor bump is breaking while pre-1.0.

**Languages:** **English** · [Русский](CHANGELOG.ru.md) · [简体中文](CHANGELOG.zh.md)

## [0.1.5] — 2026-05-01

Internal refactor of the diagnostic pipeline. No public API changes.

### Changed

- **Diagnostics** now consume `ktav::Error::Structured(ErrorKind)` from
  `ktav 0.1.5+`: the `Diagnostic.range` is built directly from the
  variant's byte-offset `Span` (via `Span::line_col`) instead of being
  re-derived by regex-matching the formatted error message and re-running
  the offending line through the shared line classifier. Range tightness
  is now driven by the parser's own knowledge of the failure point,
  yielding strictly equal-or-narrower ranges for every category — e.g.
  `MissingSeparatorSpace` for `key:value\n` now highlights bytes 4..9
  (the glued body `value`) rather than 3..9 (colon + body).
- Existing `tests/error_format_pinning.rs` accepts both `Error::Syntax(_)`
  and `Error::Structured(_)` and asserts on the rendered Display string —
  the contract is the message text, not the enum variant.
- `tests/integration.rs` range expectations tightened to the new
  structured spans (`MissingSeparatorSpace` 4..9, `InvalidTypedScalar`
  6..10 etc.). Cyrillic byte-column test re-pinned to the body span
  start (byte 7 for `имя:значение\n`).
- `tests/spec_conformance.rs` accepts the new `MissingSeparator` /
  `UnbalancedBracket` Display strings as aliases for the spec
  categories `OrphanLine` / `MismatchedBracket`.

### Added

- `tests/structured_diagnostics.rs` — per-variant assertions covering
  all 10 spec-defined `ErrorKind` variants plus a Cyrillic byte-column
  test that exercises `tokens::byte_to_utf16` on the diagnostic-range
  conversion path.

### Internal

- Regex-extraction code (`extract_line_number`, `extract_quoted_key`,
  per-category `range_for_*` helpers) is preserved as
  `compute_range_legacy`, reachable only via `Error::Syntax(_)`. The
  parser in `ktav 0.1.5+` no longer constructs that variant, but
  `ktav::Error` is `#[non_exhaustive]` and downstream wrappers may
  still surface it — the legacy path is kept as defence-in-depth.
- `ktav` dependency bumped from `0.1.4` to `0.1.5` (registry pin),
  picking up the structured-error API. Local sibling-checkout
  development can be restored via `.cargo/config.toml` patch (per-
  developer, not tracked) when iterating against an unpublished
  ktav.

### Breaking (internal)

- The crate version is bumped to **0.1.5**, so `ktav-lsp` 0.1.4-shaped
  callers (none external — this is a leaf binary) recompile cleanly
  against `ktav 0.1.5+`.

## [0.1.0] — 2026-04-26

Initial release.

### Added

- Single-binary LSP server (`ktav-lsp`) over stdin/stdout, built on
  `tower-lsp` 0.20 and `tokio`.
- **Diagnostics** — re-parses on `did_open` / `did_change` / `did_save`,
  publishes `ktav::Error::Syntax` messages on the offending line. Line
  numbers are recovered from the message via two regexes that cover all
  shapes emitted by `ktav` 0.1.4 (`Line N: …`, `Invalid key at line N: …`,
  `Empty key at line N`).
- **Hover** — for `key: …` lines, looks the dotted path up in the parsed
  tree and reports the inferred type and value.
- **Completion** — context-aware after a `:` separator: offers `null`,
  `true`, `false`, openers (`{`, `[`, `(`, `((`), empty literals (`{}`,
  `[]`, `()`), the raw marker (`:`), and typed-scalar markers (`i`, `f`).
- **Document symbols** — outline tree from `Value::Object`; scalars map
  to Property/Number/String, objects to Module, arrays to Array.
- **Semantic tokens (full)** — six token types: `comment`, `keyword`,
  `number`, `string`, `property`, `operator`. Index ordering is part of
  the public legend.
- Logging via `tracing` to stderr; `KTAV_LSP_LOG` env var controls level.
- Integration and unit tests covering the diagnostic regexes, semantic
  token legend, and document-symbol tree shape.
