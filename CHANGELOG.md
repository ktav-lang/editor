# Changelog

**Languages:** **English** · [Русский](docs/i18n/CHANGELOG.ru.md) · [简体中文](docs/i18n/CHANGELOG.zh.md)

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

## [0.8.0] — 2026-09-28

Tracks `ktav` Rust crate `0.8.0` and `ktav-lang/spec` `0.8.0`. The
universal breaking change is that **leading-zero decimal integers remain
strings** (§ 5.2, so `01234` keeps its leading zero). Separately,
§ 8.1 requires a strict parsing entry point that rejects lossy scalar
forms with `LossyScalar`. Quoted key segments
and the `\uXXXX` escape were introduced in 0.7.0.

- All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
  plugin) align on **0.8.0** with the `ktav` crate and the
  specification: `lsp/Cargo.toml` now depends on `ktav = "0.8"`, the
  spec submodule is re-pinned to `v0.8.0`, and the LSP conformance test
  now walks the 0.8 corpus (it previously walked the long-gone 0.6
  corpus, and its category check was silently disabled by an oracle-key
  mismatch; categories are now read from `ktav::ErrorEnvelope`).
- Number, keyword and escape highlighting now follows the spec exactly
  in all three components: redundant-leading-zero decimals (`01234`,
  `0_7`, § 5.2) and malformed literals (`1_`, `1__0`, `0X1A`,
  `2026-09-28`, § 3.6) are Strings; values after `::` are never typed,
  including inside inline compounds; quoted key segments are opaque
  inside inline objects (§ 5.3.3). The prebuilt `ktav-lsp` binaries are
  no longer committed (see *Repository and release tooling*).

### LSP server (`ktav-lsp`)

- `Cargo.toml`: `ktav = "0.8"` (was `"0.6"`, the version in the last
  released v0.6.1); `rust-version` raised `1.70` → `1.71`.
- **Quoted keys were already understood by the `ktav` parser in 0.7.0;**
  this release extends support to the LSP's own scanners.
  `ktav::parse`-based diagnostics/symbols already handled 0.7 syntax
  correctly with no code changes (see below), but the LSP's *own*
  line-based classifier (`tokens::classify_line`, `split_dotted`,
  used for semantic tokens, hover and completion) and the document-
  outline scanner (`symbols::collect_key_hits`) each hand-roll their
  own colon/dot scan over raw text and did not know a `:` or `.`
  inside `"..."` / `'...'` / `` `...` `` is ordinary content. A quoted
  key containing a structural byte — e.g. `"a:b": 1` or
  `a."b.c".d: 1` — would have had its colon/dot misread, corrupting
  semantic highlighting, completion context detection, and the
  document-symbol outline's key boundaries. Fixed by adding a
  quote-aware separator scan (`tokens::find_key_separator`, replacing
  `tokens::find_unescaped`) and making `tokens::split_dotted`
  quote-opaque; `symbols.rs`'s independent duplicate scanner was
  removed in favour of importing the same two functions from
  `tokens`, which now lives up to its own "single source of truth"
  doc comment. A quote character NOT at a segment's first position
  (`don't: 1`) is unaffected, matching § 5.3.3's positional rule.
- Fixed a pre-existing span-encoding bug in `textDocument/formatting`,
  found while auditing the byte-offset `Span` contract for this
  bump: the whole-document replace edit's end `Position.character`
  was computed with `str::chars().count()` (Unicode scalar count)
  instead of the same encoding-aware conversion every other handler
  in `lsp/src/server/mod.rs` already uses. This undercounts under both negotiated
  encodings whenever the last line has non-ASCII content (UTF-8:
  undercounts byte length for any multi-byte character; UTF-16:
  undercounts for any astral-plane / surrogate-pair character),
  which could leave trailing bytes of the last line unreplaced by a
  format edit. Extracted into `end_of_document()`, covered by new
  tests.
- No code changes were needed for the new `ErrorKind` variants:
  `diagnostics::parse_for_diagnostics` already drives everything off
  `ErrorKind::span()` / `::line()` / `Display` generically, so
  `UnterminatedQuotedKey` gets a correct, tight diagnostic for free.
  `Error::InvalidUtf8` cannot occur through this crate's `&str`-based
  entry points (a Rust `&str` is valid UTF-8 by construction — the
  variant only fires for the byte-level `ktav::from_file`, which this
  LSP never calls); verified, no live code path to fix.
- `reindent::canonicalise_paren_scalar`: removed dead reasoning about
  the `:i` / `:f` typed markers, dropped from the spec back in 0.5.0.
  The special case was already unreachable (any `:i` / `:f` sequence
  already fails the "separator must be followed by whitespace" check
  for an unrelated reason), so removing it changes no behaviour.
  Renamed/rewrote the tests built on the removed markers in both
  `reindent.rs` and `tests/format_pipeline.rs` (including the
  `port:i 8080` / `timeout:f 5.0` sample lines in
  `complex_document_canonicalised`, now plain pairs). The `::` raw
  marker is untouched — it is still current syntax.
- Scalar classification (`tokens::classify_value`) now implements § 3.6
  exactly plus the § 5.2 redundant-leading-zero exception: `01234`, `00`,
  `-045`, `0_7`, `01.5`, `05e3` are Strings; so are `1_`, `1__0`,
  `0x_1`, `0X1A`, `1.`, `.5`, `2026-09-28` and IPv4-like runs, while
  `0`, `0.5`, `0e0`, `0x1_A` stay numbers. The old heuristic accepted a
  sign or an underscore anywhere. Integers beyond the i64 range still
  highlight as numbers (the domain check belongs to the parser).
- Trimming uses the exact 25-code-point whitespace set of § 3.3
  (previously only space, tab and CR were trimmed at the end of a
  line), so a trailing NBSP no longer turns `true` into a string.
- Inline compounds: a value after `::` is a raw String and is never
  highlighted as a number or keyword; quoted key segments
  (`{"a,b": 1}`, `{'x:y': 2}`) are opaque to `,` `:` and brackets; a
  value containing an escape is a String (§ 3.7).
- Hover: fixed a crash — a string value longer than 80 bytes with a
  multi-byte character at the cut point panicked, and because the
  release profile aborts on panic it terminated the server. Hover now
  resolves the full key path, so keys nested in objects and in arrays
  of objects, quoted keys (`"a.b"`) and escaped keys (`a\.b`) show
  their value; the labels read `integer` / `float` (there have been no
  typed markers since 0.5).
- `tests/spec_conformance.rs` validates the corpus manifest (schema,
  categories, fixture counts), also runs `parseable-unrepresentable`
  and `strict-lossy` (through `ktav::parse_strict`), checks the
  `unrepresentable` oracles, and fails instead of silently passing
  when the spec submodule is missing.
- Semantic tokens: a multi-line string block (`(` / `((`) had no
  cross-line state, so `classify_line` re-ran on every content line in
  isolation — a line that merely looked like a comment, a pair or a
  lone closer was highlighted as one, and the verbatim terminator `))`
  fell through to a String token instead of an Operator. Fixed with a
  small `MultiForm` state carried across lines in `semantic_tokens`;
  content lines now emit one trimmed String token each, and `(` / `((`
  / `)` / `))` markers are always Operator. `hover` and `completion`
  gained the same guard (`tokens::line_is_multiline_content`) so they
  no longer misread a line inside an open block as a real `key:` pair.
- Semantic tokens: an inline-object key escaping a structural byte
  right after a comma (e.g. `{x: 0, \[a: 1}`) desynced the inline
  scanner — the lone `\` became a PROPERTY token, the escaped `[`
  opened a bogus nested array, and the real value collapsed into one
  String. `emit_inline`'s key-run scanner now treats `\X` as one
  escaped unit, matching the value-run scanner's existing behaviour.
- `tests/spec_conformance.rs` gained a corpus-wide regression guard:
  for every `valid/**.ktav` fixture, the number of Number/Bool/Null
  *leaves* in the parsed `Value` tree must equal the number of
  Number/Keyword/"null" *tokens* `semantic_tokens` emits (five
  magnitude-overflow-to-String fixtures are pinned exceptions, since
  highlighting is lexical and does not enforce i64/f64 range).
- Every handler now splits a document into lines on LF, CR and CRLF
  alike (§ 3.2, and what the LSP position model counts) through one
  helper, `tokens::lines`. A CR-only document gets correct positions,
  tokens, hover, completion, symbols, diagnostics (ranges no longer come
  from `ktav::Span::line_col`, which counts only `\n`) and formatting. The
  formatter always emits LF.
- Formatter: a comment is only a leading `##` (a single `#` is ordinary
  content, so `#child: {` nests like any key); block and compound openers
  are recognised structurally, so `key:: ((` — a literal string — no longer
  opens a multi-line block; the `name: (value)` → `name:: (value)`
  canonicalisation finds the key separator like the parser does and leaves
  the empty forms `()` / `(())` alone; a leading BOM is kept.
- `key:: {`, `key:: ((` and friends are String values, never openers;
  `(())` is classified as an empty-compound shortcut (§ 5.7).
- A leading BOM (§ 3.1) is no longer part of the first key for hover,
  symbols, tokens or diagnostics.
- Document symbols: keys inside inline objects and arrays get their own
  ranges (they used to collapse onto line 0), and the top-level `{ … }` /
  `[ … ]` wrapper is transparent.
- Hover skips comment and block-content lines through the shared
  classifier instead of its own text heuristics.
- Corpus tests: `spec_conformance.rs` now requires complete
  `.ktav` / `.canonical.ktav` / `.json` triplets and the exact fixture count
  per category, and an `invalid/` fixture without an oracle fails instead of
  being skipped; `editor_features_corpus.rs` checks document symbols, hover
  and semantic tokens on all 223 valid fixtures, and that formatting is
  idempotent and value-preserving on all 446 `.ktav` files.

### TextMate grammar (VS Code + shared `grammars/`)

- Quoted key segments: the dotted-key pattern shared by every
  `pair-*` / `inline-pair` rule now accepts `"..."`, `'...'` or
  `` `...` `` as an alternative to a bare segment at each segment
  boundary, respecting the positional rule (`don't: 1` is unaffected).
  `key-name` sub-highlights each quoted form with its own scope
  (`string.quoted.{double,single,backtick}.key.ktav`).
- `\uXXXX` escape recognised in `key-name` and `inline-scalar-body`
  (`constant.character.escape.unicode.ktav`); `inline-scalar-body`'s
  named-escape character class also gained `\.` `\:` `\"` `\'`
  `` \` `` — present in the spec since 0.6.0/0.7.0 but missing from
  this grammar's escape-highlighting list until now.
- `grammars/ktav.tmLanguage.json` is the source of truth;
  `vscode/syntaxes/ktav.tmLanguage.json` is a generated mirror kept in
  sync by `vscode/scripts/sync-grammars.js` (run via `npm run
  sync-grammars` / `compile` / `vscode:prepublish`, and explicitly in
  the release workflow before packaging). Both files are updated here
  through that script so they stay byte-identical.
- Number scopes follow § 3.6 / § 5.2 exactly (whole-line pair values,
  array items and inline values): redundant leading zeros (`01234`,
  `0_7`, `01.5`), misplaced underscores (`1_`, `1__0`, `0x_1`),
  upper-case base prefixes (`0X1A`) and dates or dotted runs
  (`2026-09-28`, `127.0.0.1`) are strings, not numbers.
- `\uXXXX`: a high+low surrogate pair is one escape token; a lone
  surrogate or a malformed `\u` gets
  `invalid.illegal.escape.unicode.ktav` (§ 3.7.1).
- Inline objects: a `::` value is always `string.unquoted.raw.ktav`
  (never number or keyword); quoted key segments are recognised at any
  indentation and after `{` or `,`.
- Fixed a structural defect: `^` / `$` inside a rule reached through
  `captures` anchor to the line, not to the capture, so the
  number/keyword/quoted-key classification silently failed whenever the
  value did not start at column 0. The classification is now part of
  the directly scanned pattern.
- A tokenizer test (`vscode/src/test/unit/grammar-tokens.test.ts`, on
  `vscode-textmate` + `vscode-oniguruma`) runs the real grammar over
  these vectors in whole-line, array-item and inline contexts.
- A document that is a single-line inline object or array at the top
  level (no leading key) was not highlighted at all; the root patterns
  now include `top-level-inline-object` / `top-level-inline-array`. A
  corpus-wide tokenizer test (`corpus-coverage.test.ts`) runs the grammar
  over every valid fixture: no `invalid.*` scope, and the number, boolean
  and null scope counts match the fixture's expected value.

### IntelliJ plugin

- The highlighting lexer follows the spec: exact § 3.6 number grammar
  with the § 5.2 leading-zero exception (ASCII digits only — the old
  check used `Char.isDigit()` and accepted other scripts' digits),
  exact keywords, and the exact 25-code-point whitespace set of § 3.3.
- Quoted key segments (§ 5.3.3) are supported in whole-line and inline
  keys; an unterminated quote degrades to a plain value and keeps the
  incremental re-lexing state valid.
- `::` values are Strings inside inline compounds as well; an inline
  scalar with an escape is a String; a literal `:` inside an inline
  value is no longer taken for a separator.
- Fixed stale Marketplace/Settings copy: the comment-toggle description
  said `#` instead of `##` (`plugin.xml`, `build.gradle.kts`); the
  Settings → Tools → Ktav help text and `KtavLanguage`/`KtavConfigurable`
  KDoc claimed LSP features need the separate LSP4IJ plugin and that no
  binary is bundled — both were true only before this plugin grew its
  own built-in LSP client and per-platform bundled binaries. The
  Marketplace description's example also had `# ...` inline comments,
  which are not valid Ktav (`#` is ordinary content outside a leading
  `##`) — rewritten with standalone `##` lines.
- Lexer: the body of a multi-line `(` / `((` block is now opaque
  (`MULTILINE_TEXT`, closed by `)` / `))`) — before, `{`, `[` and
  `key: value` lines inside it were lexed as structure and a bare `(`
  array item was a bad character. A `\r` before `\n` is whitespace, so
  CRLF documents keep `true` and numbers typed.
- `KtavCorpusCoverageTest` runs the lexer over every valid fixture (no
  gaps or overlaps in the token stream; number, keyword and null counts
  match the fixture's expected value). Three older `KtavLexerTest`
  assertions compared against the platform's `TokenType.BAD_CHARACTER`
  instead of the plugin's own and could never fail — fixed.

### Spec submodule

- Pinned to `5871254` (`v0.8.0`), up from `04f867f` (`v0.7.0`).

### Repository and release tooling

- The prebuilt `ktav-lsp` binaries are no longer committed: the
  tracked copies had drifted (five platforms embedded `ktav` 0.5.0,
  win32-x64 embedded 0.7.1, and a stray copy under
  `intellij/src/main/resources/bin/` embedded 0.1.5). `intellij/bin/`,
  `vscode/bin/` and `intellij/src/main/resources/bin/` are git-ignored;
  the release workflow builds all six platforms from source, and
  `scripts/build-binaries.sh` does the same locally.
- The TextMate tokenizer test adds pinned dev-only dependencies
  (`vscode-textmate`, `vscode-oniguruma`) to the VS Code project — not
  shipped in the VSIX — and `tsconfig.json` gains the `DOM` lib for
  their WebAssembly typings.
- Release workflow: `vsce package` / `vsce publish` no longer pass
  `--no-dependencies`. The extension needs `vscode-languageclient` at
  runtime and the 0.6.1 VSIX shipped without `node_modules`, so the
  packed extension could not load its language client; production
  dependencies are now packed (devDependencies are not).
- CI: the docs job uses `actions/setup-node@v6`.
- Documentation: the VS Code and LSP READMEs now agree that the
  Marketplace and Open VSX extension bundles `ktav-lsp` for six
  platforms (a separate install is only needed elsewhere);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` and
  `lsp/docs/bench-baseline.md` are marked as historical.

## [0.6.1] — 2026-06-05

- Docs: rewrite all README examples to spec 0.6 syntax (bare numbers instead of removed `:i`/`:f` markers; `##` comments instead of `#`).
- LSP: remove `:i`/`:f` from completion items (typed markers removed in spec 0.5).

## [0.6.0] — 2026-06-01

Tracks `ktav` Rust crate `0.6.0` and `ktav-lang/spec` `0.6.0`. The
version is realigned to move in lockstep with the format/core (the
previous editor release was `0.3.1`). The spec change is **key
escaping**: keys now process the § 3.7 escape set and add `\.` (a
literal dot that does *not* split a dotted path) and `\:` (a literal
colon that is *not* a pair separator); a literal backslash in a key is
now written `\\`. Editor support is updated end-to-end so highlighting,
tokens and diagnostics treat escaped key bytes correctly.

### LSP server (`ktav-lsp`)

- Semantic tokens, document symbols and diagnostics are now
  **escape-aware**: the pair separator is the first *unescaped* `:` / `::`
  and dotted-path splitting happens only on *unescaped* `.`. So
  `a\.b: v` is the single key `a.b`, and `a\:: v` is the key `a:` with
  value `v`.
- Depends on `ktav` 0.6.0.

### IntelliJ plugin

- `KtavLexer` lexes escaped key segments (`\\ \. \: \, \} \] \{ \[ \n \r`),
  so a key containing an escaped dot or colon highlights as one key.

### TextMate grammar (VS Code + shared `grammars/`)

- The key-segment pattern accepts `\`-escapes, matching the spec's key
  escape set.


## [0.3.1] — 2026-05-10

Tracks `ktav` Rust crate `0.3.1` and `ktav-lang/spec` `0.1.1`. The
spec change adds top-level Array as a recognised root kind (additive,
existing Object documents parse identically); the editor surfaces it
in the LSP outline and pins the formatter.

### LSP server (`ktav-lsp`)

- **Top-level Array** (spec § 5.0.1) is now first-class in
  `build_symbols`: when the parsed root is an Array, items render as
  `[0]`, `[1]`, … entries in the document outline (mirroring how
  nested arrays already render). For object items the symbol's range
  points at the line of the item's first key; for bare-scalar items
  the range covers the item's own line.
- `reindent` is verified to preserve bare top-level Array form — the
  formatter does not synthesise `[ ... ]` brackets at the document
  root. Pinned by new test cases in `tests/format_pipeline.rs`.
- The 0.3.0 `name: (value)` → `name:: (value)` auto-disambiguation in
  `reindent` is preserved.
- Existing diagnostic / pinning tests updated to use a leading anchor
  pair (`anchor: 1\n…`) for inputs that were previously
  bare-pair-shaped at the document start — under spec 0.1.1 those
  bare lines now parse as top-level Array string items, so the
  fixtures are anchored explicitly to keep the malformed-pair branch
  exercised.

### Sync

- `lsp/Cargo.toml`: `ktav = "0.3.1"` (was a path-dep during local
  development of 0.3.0 → 0.3.1).
- `editor/spec` submodule pinned at `7256816` (spec 0.1.1).


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
