# Changelog

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
