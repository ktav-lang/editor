# Changelog

**Languages:** **English** · [Русский](CHANGELOG.ru.md) · [简体中文](CHANGELOG.zh.md)

All notable changes to the Ktav IntelliJ Platform plugin are documented
here. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/) with the
pre-1.0 convention that a MINOR bump is breaking.

This file is also displayed in the IDE's plugin pane — only the first
twenty lines are forwarded by the build (see `build.gradle.kts`
`changeNotes` mapping), so keep recent releases at the top and prefer
short bullet points.

## 0.5.1

- Compatibility range raised to IntelliJ 2023.1+ (`since-build 231`).
  The Marketplace verifier reported a hard incompatibility on 2022.1–2022.3;
  every build 231+ verifies as Compatible, so the range now matches reality.
- API-deprecation cleanup (no behaviour change): `CodeInsightColors.INFO_ATTRIBUTES`
  → `WEAK_WARNING_ATTRIBUTES`; `TextFieldWithBrowseButton.addBrowseFolderListener(title, …)`
  → manual `FileChooser.chooseFile` with the title on the descriptor;
  `TextAttributesKey.createTextAttributesKey(String, TextAttributes)` →
  `enforcedTextAttributes`. The plugin now verifies warning-free on 2023.1–2024.3.
- Bundles the same `ktav-lsp 0.5.0`.

## 0.5.0

- Bundles `ktav-lsp 0.5.0` (sync to ktav 0.5.0 + spec 0.5.0).
- TextMate grammar: comment pattern updated to `##`, removed `:i`/`:f`
  typed-marker patterns, added inline-compound and number-literal patterns.
- License: dual `MIT OR Apache-2.0`.

## 0.3.1

- Bundles `ktav-lsp 0.3.1` (sync to ktav 0.3.1 + spec 0.1.1).
- Document-symbols outline now lists top-level Array items as
  `[0]`, `[1]`, … entries.

## 0.1.0

- Initial release of the Ktav IntelliJ Platform plugin.
- Registers the `Ktav` file type for `*.ktav` files.
- Comment toggle (`# `) wired through `lang.commenter`.
- Bundles the shared TextMate grammar from `editor/grammars/`.
- Targets IntelliJ Platform 2024.3 (build `243`) through 2025.1 (`251.*`).
