# Changelog

**Languages:** **English** · [Русский](docs/CHANGELOG.ru.md) · [简体中文](docs/CHANGELOG.zh.md)

All notable changes to the Ktav IntelliJ Platform plugin are documented
here. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/) with the
pre-1.0 convention that a MINOR bump is breaking.

This file is also displayed in the IDE's plugin pane — only the first
twenty lines are forwarded by the build (see `build.gradle.kts`
`changeNotes` mapping), so keep recent releases at the top and prefer
short bullet points.

## [0.8.0] — 2026-09-28

All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
plugin) move to 0.8.0 in step with the `ktav` crate and the
specification — see the root [`CHANGELOG.md`](../CHANGELOG.md)
(0.8.0 section) for the full list.

- Incremental highlighting preserves root/container context and whole Array
  Strings, including lone paren closers and multiword bare keys with quotes.
- Folding uses lexer tokens, keeping raw values and multiline bodies opaque.
- Diagnostics are project/client/version/session-owned; delayed publications
  and other project closures cannot overwrite current results. Split-editor
  highlighters are cleared through each actual owning model.
- Highlighting lexer: exact § 3.6 / § 5.2 numbers (ASCII digits only),
  § 3.3 whitespace, quoted key segments, raw `::` values inside inline
  compounds.
- The stale bundled `ktav-lsp.exe` copy (built from `ktav` 0.1.5) under
  `src/main/resources/bin/` is removed; the plugin never used it.
- Marketplace/Settings descriptions no longer say `#` for comment
  toggle (it's `##`) or claim LSP4IJ / a non-bundled binary are
  needed — the plugin has its own built-in LSP client and bundles a
  per-platform `ktav-lsp` binary.
- Lexer: multi-line `(` / `((` block bodies are opaque and CRLF no longer
  breaks value typing; a corpus-wide lexer test covers every valid
  fixture.
- LSP document subscriptions are owned per project, including shared
  documents, restored tabs and close/reopen. Disposal removes only that
  project's listeners; initialization publishes only a ready client
  and safely stops a server process created during disposal.
  Transport closure fails pending requests and immediately rejects
  requests racing with disposal.
- Formatting checks the synchronized document/version/session and client
  again when applying the result, so intervening edits, close/reopen,
  disposal, cancellation and expired responses cannot overwrite current
  text. Application is atomic with the final check and has a separate
  Undo step; the applied text is sent to every subscribed project.

## 0.5.1

- Compatibility range raised to IntelliJ 2023.1+ (`since-build 231`).
  The Marketplace verifier reported a hard incompatibility on 2022.1–2022.3;
  every build 231+ verifies as Compatible, so the range now matches reality.
- API-deprecation / internal-API cleanup (no behaviour change): `INFO_ATTRIBUTES`
  → `WEAK_WARNING_ATTRIBUTES`; `addBrowseFolderListener(title, …)` → manual
  `FileChooser.chooseFile`; `createTextAttributesKey(String, TextAttributes)` →
  `enforcedTextAttributes`; `FileChooserDescriptorFactory.createSingleFileDescriptor()`
  → the `FileChooserDescriptor` constructor; `Document.addDocumentListener(l)` → the
  `Disposable` overload (also fixes a listener leak); `DaemonCodeAnalyzer.restart()` →
  per-file `restart(PsiFile)`; internal `PluginManagerCore.getPlugin(id)` → the
  plugin's own class-loader descriptor. Verifies warning-free on 2023.1–2024.3.
- Bundles the same `ktav-lsp 0.5.0`.

## 0.5.0

- Bundles `ktav-lsp 0.5.0` (sync to ktav 0.5.0 + spec 0.5.0).
- TextMate grammar: comment pattern updated to `##`, removed `:i`/`:f`
  typed-marker patterns, added inline-compound and number-literal patterns.
- License: dual `MIT OR Apache-2.0`.

## 0.3.1

- Bundles `ktav-lsp 0.3.1` (sync to ktav 0.3.1 + spec 0.1.1).
- The bundled server's document-symbols response now lists top-level
  Array items as `[0]`, `[1]`, … entries. The built-in IntelliJ client
  does not display a document-symbols outline.

## 0.1.0

- Initial release of the Ktav IntelliJ Platform plugin.
- Registers the `Ktav` file type for `*.ktav` files.
- Comment toggle (`# `) wired through `lang.commenter`.
- Bundles the shared TextMate grammar from `editor/grammars/`.
- Targets IntelliJ Platform 2024.3 (build `243`) through 2025.1 (`251.*`).
