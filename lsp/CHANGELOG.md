# Changelog — `ktav-lsp`

All notable changes to the `ktav-lsp` crate are documented here. The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate adheres to [Semantic Versioning](https://semver.org/) with
the Cargo convention that a minor bump is breaking while pre-1.0.

**Languages:** **English** · [Русский](CHANGELOG.ru.md) · [简体中文](CHANGELOG.zh.md)

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
