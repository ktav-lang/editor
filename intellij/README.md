# Ktav — IntelliJ Platform plugin

**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)

> Editor support for the [Ktav](https://github.com/ktav-lang/spec)
> plain configuration format inside JetBrains IDEs.

This is the `intellij/` subproject of the
[`ktav-lang/editor`](https://github.com/ktav-lang/editor) monorepo. It
ships the same TextMate grammar that the VS Code extension uses,
wrapped as an IntelliJ Platform plugin.

## Supported IDEs

Anything built on the IntelliJ Platform **2024.3** (build `243`) or newer:

- IntelliJ IDEA Community / Ultimate
- RustRover
- GoLand
- WebStorm
- PyCharm Community / Professional
- PhpStorm
- RubyMine
- CLion
- DataGrip
- Android Studio (Iguana / Jellyfish or newer, once their platform base catches up)
- Aqua, Rider, Fleet (when on a compatible build)

`untilBuild` is currently `251.*` (covers 2024.3 → 2025.1). New IDE
releases are verified before each plugin release.

## Installation

### From JetBrains Marketplace (recommended)

1. Open **Settings → Plugins → Marketplace** in any supported IDE.
2. Search for **Ktav**.
3. Click **Install** and restart the IDE.

### From a local zip (for testing)

Build the plugin (see below) and then in **Settings → Plugins** open
the gear menu → **Install Plugin from Disk…** and pick
`build/distributions/ktav-intellij-<version>.zip`.

## Features

- Syntax highlighting for `.ktav` files via the shared TextMate grammar.
- Comment toggle (`Ctrl/Cmd+/`) prepends `# ` per the Ktav spec.
- Bracket matching and auto-closing for `{}` `[]` `()`.
- File icon and File → New → Ktav file (icon TODO; uses the platform
  default text-file icon for now).

### LSP features (optional)

When the [LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij)
plugin is installed alongside Ktav, you get live diagnostics, hover,
completion, document symbols, and semantic tokens served by
[`ktav-lsp`](../lsp). Without LSP4IJ the plugin still works in
TextMate-only mode — install LSP4IJ from the Marketplace whenever you
want the richer features.

The server binary is discovered in this order:

1. The explicit path configured under **Settings → Tools → Ktav**.
2. A binary bundled inside the plugin distribution at
   `bin/<platform>-<arch>/ktav-lsp` (not bundled in the current
   release).
3. `ktav-lsp` resolved via your shell `PATH` — install it with
   `cargo install ktav-lsp` (matches the VS Code extension's
   discovery order).

## Building locally

Prerequisites:

- JDK 17 or newer (the build pins the Kotlin toolchain to 17).
- Gradle is **not** required — use the wrapper.

```sh
./gradlew syncGrammars   # mirror ../grammars/ into resources/
./gradlew buildPlugin    # produces build/distributions/*.zip
./gradlew runIde         # boot a sandbox IDE with the plugin loaded
./gradlew verifyPlugin   # run the JetBrains plugin verifier
./gradlew test           # JUnit 5 smoke tests
```

The `processResources` and `compileKotlin` tasks depend on
`syncGrammars` so a plain `./gradlew buildPlugin` already pulls in the
latest grammar from `../grammars/`. If you forget and the file is stale,
just rerun `syncGrammars`.

## Publishing

CI invokes `./gradlew publishPlugin` with the marketplace PAT in the
`INTELLIJ_PUBLISH_TOKEN` environment variable. The token is generated at
<https://plugins.jetbrains.com/author/me/tokens> and must belong to a
maintainer of the `lang.ktav` plugin id on the marketplace. Local
publishing is intentionally not supported — release via tagged CI runs
only.

## Reference

- Format spec & reference parser: [`ktav-lang/spec`](https://github.com/ktav-lang/spec)
- Reference Rust implementation: [`ktav-lang/rust`](https://github.com/ktav-lang/rust)
- Other bindings, LSP server, and the VS Code extension live in the
  [editor monorepo](https://github.com/ktav-lang/editor).
