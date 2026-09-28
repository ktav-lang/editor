# Changelog

**Languages:** **English** · [Русский](docs/CHANGELOG.ru.md) · [简体中文](docs/CHANGELOG.zh.md)

## [0.8.0] — 2026-09-28

All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
plugin) move to 0.8.0 in step with the `ktav` crate and the
specification — see the root [`CHANGELOG.md`](../CHANGELOG.md)
(0.8.0 section) for the full list.

- TextMate grammar: exact § 3.6 / § 5.2 number scopes, surrogate-pair
  aware `\uXXXX`, raw `::` values and quoted keys inside inline objects,
  and a fix for classification that failed off column 0.
- A real tokenizer test (`npm run test:unit`) now runs the grammar
  over the vectors.
- The packaged extension now includes its production dependencies
  (`vscode-languageclient`); the 0.6.1 VSIX was built without them.

## 0.5.0

- Bundles `ktav-lsp 0.5.0` (sync to ktav 0.5.0 + spec 0.5.0).
- TextMate grammar: comment pattern updated to `##` (spec 0.5.0).
- TextMate grammar: removed `:i`/`:f` typed-marker patterns.
- TextMate grammar: added inline-compound patterns (`{key: val, …}`,
  `[v1, v2, …]`), escape sequences, and hex/oct/bin number literals.
- License: dual `MIT OR Apache-2.0`.

## 0.3.1

- Bundles `ktav-lsp 0.3.1` (sync to ktav 0.3.1 + spec 0.1.1).
- Document-symbols outline now lists top-level Array items as
  `[0]`, `[1]`, … entries.

## 0.1.0

Initial release.

- Syntax highlighting for `.ktav` files via shared TextMate grammar
- Bracket matching for `{}` `[]` `()`
- Comment toggle (`#`)
- Auto-indent inside compounds
