# Contributing to ktav-lang/editor

**Languages:** **English** · [Русский](docs/i18n/CONTRIBUTING.ru.md) · [简体中文](docs/i18n/CONTRIBUTING.zh.md)

## Core rules

### 1. Every bug fix ships with a regression test

When you find a bug, **before fixing it**, write a test that reproduces
it — the test **must fail on `main`** and pass after the fix. Include
both in the same PR.

### 2. Don't reinvent the format in the editor layer

Editor extensions and the LSP are thin consumers of the `ktav` parser
crate. Format behaviour belongs in the Rust crate
([`ktav-lang/rust`](https://github.com/ktav-lang/rust)) — changing it
there updates every consumer at once. Only **editor-specific
ergonomics** (TextMate scopes, LSP feature wiring, IDE-specific
integrations) belong in this repo.

If your change requires a format change, start a discussion in
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) first.

### 3. One concept per commit

Commits should be atomic: a bug fix and its test together, a feature
and its tests together, a rename on its own, a refactor on its own.
`git log --oneline` should read like a changelog. Don't prefix commit
messages with `feat:` / `fix:` — no conventional commits here.

## Dev setup

Each subproject has its own toolchain. See the README in each:

- `grammars/` — pure JSON; no build
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.88+ (minimum required by the locked dependency graph)

The prebuilt `ktav-lsp` binaries (`vscode/bin/`, `intellij/bin/`) are not
committed. Build them with `scripts/build-binaries.sh` (it uses `cross`
for the Linux and macOS targets); the release workflow builds every
platform from source. The VS Code project also has a grammar tokenizer
test: `npm run test:unit` in `vscode/`.

## Installing a local build into your IDEs

`scripts/dev-install.mjs` (Node 20+) builds `ktav-lsp` for this machine,
the VSIX and the IntelliJ plugin ZIP, and installs them into IDEs you
name explicitly:

```sh
node scripts/dev-install.mjs list
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium --apply
```

Without `--apply` it only validates the targets and prints the plan. It
never deletes anything: an existing `ktav-intellij` folder must be the
`lang.ktav` plugin and is moved to
`<ide-config>/ktav-dev-backups/<stamp>/previous/`. Symlinks, other
plugins, settings and caches are left alone, and a running JetBrains IDE
is refused. VS Code-family editors are updated only through their own
`--install-extension` CLI.

## Language policy

This repo participates in the org-wide three-language policy (EN / RU /
ZH). Every prose file lives in three parallel versions — see
[`ktav-lang/.github/AGENTS.md`](https://github.com/ktav-lang/.github/blob/main/AGENTS.md)
for the naming convention and the "update all three in one commit"
rule.

### License of contributions

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this project by you, as defined in the
Apache-2.0 license, shall be dual-licensed as **MIT OR Apache-2.0**,
without any additional terms or conditions.
