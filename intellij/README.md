# Ktav — IntelliJ Platform plugin

**Languages:** **English** · [Русский](docs/README.ru.md) · [简体中文](docs/README.zh.md)

> Editor support for the [Ktav](https://github.com/ktav-lang/spec)
> plain configuration format inside JetBrains IDEs.

This is the `intellij/` subproject of the
[`ktav-lang/editor`](https://github.com/ktav-lang/editor) monorepo. It
ships a native IntelliJ Platform lexer and highlighter (not the shared
TextMate grammar the VS Code extension uses), wrapped as an IntelliJ
Platform plugin.

## Supported IDEs

Anything built on the IntelliJ Platform **2023.1** (build `231`) or
newer — there is no upper bound (`untilBuild` is unset):

- IntelliJ IDEA Community / Ultimate
- RustRover
- GoLand
- WebStorm
- PyCharm Community / Professional
- PhpStorm
- RubyMine
- CLion
- DataGrip
- Android Studio (once its platform base reaches 2023.1)
- Aqua, Rider, Fleet (when on a compatible build)

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

- Native syntax highlighting for `.ktav` files (own lexer, no TextMate).
  Incremental highlighting preserves the first-content root kind and nested
  Object/Array context; quotes inside multiword bare keys remain literal.
- Folding for multiline compounds and strings uses the same lexer tokens,
  leaving raw scalars and multiline string bodies opaque.
- Comment toggle (`Ctrl/Cmd+/`) prepends `## ` per the Ktav spec.
- Bracket matching and auto-closing for `{}` `[]` `()`.
- File icon and File → New → Ktav file (icon TODO; uses the platform
  default text-file icon for now).

### LSP features

The plugin talks to [`ktav-lsp`](../lsp) through its own built-in LSP
client — no separate LSP plugin (e.g. LSP4IJ) is required for live
diagnostics and whole-file formatting (**Reformat Code**). These are
the LSP features currently integrated into the IntelliJ plugin.

The server also supports hover, completion, document symbols and
semantic tokens, but the built-in IntelliJ client does not integrate
them. Syntax highlighting comes from the plugin's native lexer, not
LSP semantic tokens.

Diagnostics are tied to each project's current open client, version and
document session. Cached annotations and direct editor highlights remain
independent: stale publications or closing one project cannot clear another
owner's errors.

Each project owns its LSP client and document subscriptions. A `.ktav`
document open in two projects is synchronized to both independently;
closing it in one project does not stop updates in the other. Restored
editor tabs use the same open path, and reopening sends the current text
as a new document session. Project disposal removes its subscriptions
and closes its client, including a server process that starts late.
Closing the transport fails outstanding requests and rejects new ones
immediately instead of leaving them waiting for a response timeout.

Formatting applies only to the synchronized document session that
requested it. Edits (even undoing back to the original text), close/reopen,
project/client disposal, cancellation or an expired result prevent a
stale response from replacing the document. A successful result is
applied and checked together on the editor thread, is synchronized back
to each owner, and has its own **Undo** step.

The server binary is discovered in this order:

1. The explicit path configured under **Settings → Tools → Ktav**.
2. The binary bundled inside the plugin distribution at
   `lib/bin/<platform>-<arch>/ktav-lsp`, one per supported platform.
3. `ktav-lsp` resolved via your shell `PATH` — install it with
   `cargo install ktav-lsp --version 0.8.0 --locked` (matches the VS Code extension's
   discovery order).

## Building locally

Prerequisites:

- JDK 17 or newer (the build pins the Kotlin toolchain to 17).
- Gradle is **not** required — use the wrapper.
- Rust 1.88+ when rebuilding the current locked LSP.

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

For an installed IDE, close that IDE manually, then run the portable helper
from any working directory:

```sh
./dev-rebuild.sh --plugins-dir /path/to/selected-ide/plugins --platform linux-x64 --no-restart
```

Use `--plugin` (Python 3 required for ZIP installation) to rebuild/install
the whole Ktav plugin.
If multiple output ZIPs exist, select the exact built archive with `--plugin-zip PATH`.
`--ide-executable PATH` optionally launches only the selected IDE afterward;
it never terminates an existing IDE. `--cache-dir PATH` only reports the log
location, without deleting caches or logs. Equivalent environment variables:
`KTAV_PLUGINS_DIR`, `KTAV_PLATFORM`, `KTAV_PLUGIN_ZIP`,
`KTAV_IDE_EXECUTABLE`, `KTAV_IDE_CACHE_DIR`. No machine-specific defaults.
Only `plugins/ktav-intellij` is replaced; unrelated plugins and ZIPs stay put.
See `./dev-rebuild.sh --help` for all supported platform names.

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
