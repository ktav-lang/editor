# Changelog

**Languages:** **English** · [Русский](CHANGELOG.ru.md) · [简体中文](CHANGELOG.zh.md)

All notable changes to the Ktav editor support (VS Code extension,
IntelliJ plugin, LSP server, shared TextMate grammar) are documented
here. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/) with the
pre-1.0 convention that a MINOR bump is breaking.

A single tag (`v0.X.Y`) ships all four subprojects simultaneously.
Per-subproject changes are grouped under the version heading.

This changelog tracks **editor/IDE support releases**, not changes to
the Ktav format itself — for the latter see
[`ktav-lang/spec`](https://github.com/ktav-lang/spec/blob/main/CHANGELOG.md).

## Unreleased

(no changes since 0.3.0)


## [0.3.0] — 2026-05-08

Tracks `ktav` Rust crate `0.3.0`. Picks up the parser-strictness
change (inline `(value)` / `((value))` now an error), duplicate-key
span fix, and hot-path micro-optimisations. Plus one user-visible
LSP win:

### LSP server (`ktav-lsp`)

- **`build_symbols` rewritten as O(N) single-pass scanner** — the
  IDE document outline previously stalled the language server for
  ~11.5 seconds on a 500 KiB document because every top-level key
  triggered a full-document text scan. The new scanner walks the
  text once, recording `(virtual_depth, key, line_range)` for every
  pair, and the DFS over the parsed `Value` advances a sequential
  cursor through the hits. Wall-clock: 11.5 s → 13.2 ms (871×
  faster). Outline-aware editors (JetBrains, VSCode) no longer
  hang on large config files.

- **Format pipeline: line-based reindent runs unconditionally**
  instead of gating on a successful parse. With the `ktav` 0.3.0
  parser strictness rejecting inline `(value)`, the previous
  "format only when parses" gate locked users out of formatting
  exactly when formatting would have repaired the issue.
  `canonicalise_paren_scalar` rewrites `key: (value)` →
  `key:: (value)` on save. 22-test integration suite added in
  `tests/format_pipeline.rs` covering indentation normalisation,
  blank-line preservation, comment preservation, multi-line
  stripped/verbatim forms (byte-exact), auto-fix of inline paren
  scalars, edge cases (CRLF, no trailing newline, empty doc).

### IntelliJ plugin

- Picks up the `ktav-lsp` 0.3.0 binary with the symbols speed-up
  and the format-without-parse change. No standalone IntelliJ
  changes in this release; build-time-stamped version
  `0.3.0+YYYYMMDD-HHMM` keeps the IDE-visible version
  distinguishable across iterative rebuilds.

### VS Code extension

- Picks up the `ktav-lsp` 0.3.0 binary. No standalone VS Code
  changes in this release.


## [0.2.0] — 2026-05-07

First synchronised release across all four subprojects (LSP, VS Code,
IntelliJ, shared grammar). Tracks `ktav` Rust crate `0.2.0`.

### LSP server (`ktav-lsp`)

- **textDocument/formatting** capability + handler:
  parse → render through `ktav` crate. Empty edits when content is
  already canonical or input fails to parse.
- Snap to `ktav = "0.2.0"` from crates.io (was a path-dep during
  development).
- Diagnostics for `:f 42` no longer emitted (handled at `ktav`
  semantics layer — integer literals coerce to float).
- Multi-line strings render in stripped `( ... )` form by default
  (verbatim `(( ... ))` remains as fallback for content with leading
  whitespace or sole-`)` lines).

### VS Code extension

- Explicit `DocumentFormattingEditProvider` registration (so
  `editor.defaultFormatter = ktav-lang.ktav` resolves correctly and
  VS Code does not prompt to install another formatter).
- `configurationDefaults` for `[ktav]`: tabSize 4, insertSpaces,
  defaultFormatter pinned to our extension.
- VSIX packaged with `vscode-languageclient` runtime tree included —
  fixes `Cannot find module 'vscode-languageclient/node'` activation
  failure introduced by an earlier `--no-dependencies` packaging.

### IntelliJ plugin

- Replaced LSP4IJ dependency with an in-house JSON-RPC LSP client
  (no external plugin required).
- Native syntax highlighting via state-machine lexer:
  KEY / KEY_DOT / MARKER_INT / MARKER_FLOAT / DOUBLE_COLON / COLON /
  STRING_VALUE / INT_VALUE / FLOAT_VALUE / BOOLEAN / NULL /
  MULTILINE_OPEN/CLOSE / BRACES / BRACKETS / COMMENT.
- `KtavParserDefinition` (minimal flat parser) — gives PSI tree so
  `ExternalAnnotator` (for LSP diagnostics → tooltip + Problems View)
  works.
- `KtavFoldingBuilder` — folds `{}`, `[]`, `()`, `(())`.
- `KtavBraceMatcher` — paired-brace highlight + auto-close on type.
- `KtavUnicodeAnnotator` — boxed red highlight for non-ASCII chars
  inside keys (analog of VS Code's `editor.unicodeHighlight`).
- `KtavFormattingService` (AsyncDocumentFormattingService) hooks
  Ctrl+Alt+L into LSP `textDocument/formatting`.
- `KtavStartupActivity` syncs already-open `.ktav` files when the
  plugin loads dynamically.
- Bundled cross-platform `ktav-lsp` binaries (Windows x64).
- Plugin distribution ZIP correctly packages binaries via
  `_repackageWithBinaries` task.
- `pluginVerification` pinned to `IC-2024.3 / 2025.1 / 2025.2`
  (avoids `recommended()` failing on metadata-only future releases).
- `untilBuild = provider { null }` — no upper IDE-version pin in
  plugin.xml (was emitting `until-build=""` which the verifier
  rejected).
- Plugin version bumped 0.1.5 → 0.2.0.

### Shared grammar (`grammars/`)

- `pair-float` and `array-item-float` regexes accept integer
  literals after `:f` (decimal point now optional). Synced into
  VS Code `syntaxes/` and IntelliJ `resources/grammars/`.

### Spec submodule

- `typed_float_without_decimal` fixture moved from `invalid/` to
  `valid/typed_float_integer_body` (matches new `:f 42` semantics).
