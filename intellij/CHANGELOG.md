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
