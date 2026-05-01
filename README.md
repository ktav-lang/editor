# ktav-lang/editor

**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)

> Editor support for the **[Ktav](https://github.com/ktav-lang/spec)** configuration format —
> syntax highlighting, IDE plugins, and a Language Server. One repo,
> four subprojects, one shared TextMate grammar.

## What's inside

| Subproject              | What it is                                                      | Install / publish to                                    |
|-------------------------|------------------------------------------------------------------|---------------------------------------------------------|
| [`grammars/`](grammars/)| Shared TextMate grammar + VS Code language configuration         | Reused by `vscode/` and `intellij/`                     |
| [`vscode/`](vscode/)    | Visual Studio Code extension                                     | VS Code Marketplace + Open VSX                          |
| [`intellij/`](intellij/)| IntelliJ Platform plugin (IntelliJ IDEA, RustRover, GoLand, …)   | JetBrains Marketplace                                   |
| [`lsp/`](lsp/)          | Language Server Protocol implementation (Rust, `tower-lsp`)      | crates.io as `ktav-lsp`                                 |
| [`docs/`](docs/)        | Editor-specific setup snippets (Helix, Neovim, Emacs)            | —                                                       |

## What you get as a Ktav user

- **Syntax highlighting** — keys, scalars, typed-marker bodies (`:i` / `:f`), raw-string marker (`::`), multi-line strings, comments
- **Bracket matching & auto-close** — `{}` `[]` `()`
- **Comment toggle** — `Ctrl/Cmd+/` → `# comment`
- **Live diagnostics** (with the LSP) — every `MissingSeparatorSpace`, duplicate key, dotted-prefix conflict surfaces as a red squiggle on the offending line, with the same message the parser emits
- **Hover info** (with the LSP) — dotted path of the key under cursor, inferred type of the value
- **Completion** (with the LSP) — keywords (`null` / `true` / `false`), markers (`:i` / `:f` / `::`), compound openers
- **Document symbols** (with the LSP) — outline reflecting the parsed `Value::Object` structure

## Architecture

```
                    ┌─────────────────────────────────────────┐
                    │  ktav-lsp  (Rust binary, this repo)    │
                    │  • parses via the `ktav` crate         │
                    │  • diagnostics, hover, completion,     │
                    │    semantic tokens, document symbols   │
                    └─────────────────────────────────────────┘
                                       ▲
                                       │ LSP (JSON-RPC over stdio)
                ┌──────────────────────┼──────────────────────┐
                │                      │                      │
       ┌────────┴────────┐   ┌─────────┴─────────┐   ┌────────┴────────┐
       │ VS Code         │   │ IntelliJ Platform │   │ Helix / Neovim  │
       │ ext (`vscode/`) │   │ plugin (`intellij/`)│  │ + LSP config    │
       │                 │   │                     │  │                 │
       │ TextMate grammar│   │ TextMate grammar    │  │ (LSP semantic   │
       │ from grammars/  │   │ from grammars/      │  │  tokens for     │
       │                 │   │                     │  │  highlighting)  │
       └─────────────────┘   └─────────────────────┘  └─────────────────┘
```

The TextMate grammar gives instant cosmetic highlighting (no language
server needed). The LSP layer adds the *intelligent* features —
diagnostics, hover, completion. They stack: install the extension
alone for highlighting, add `ktav-lsp` to your PATH for everything else.

## Installing as a user

### VS Code

```
ext install ktav-lang.ktav
```

The extension auto-bundles the LSP server for the major platforms — no
extra install step.

### IntelliJ IDEA / RustRover / GoLand / etc.

Plugins → Marketplace → search **Ktav** → Install.

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Then `cargo install ktav-lsp`.

### Neovim

With [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig):

```lua
vim.filetype.add({ extension = { ktav = "ktav" } })
require("lspconfig.configs").ktav = {
  default_config = {
    cmd = { "ktav-lsp" },
    filetypes = { "ktav" },
    root_dir = require("lspconfig.util").root_pattern(".git"),
    settings = {},
  },
}
require("lspconfig").ktav.setup({})
```

Then `cargo install ktav-lsp`.

### Other editors

See [`docs/`](docs/) for Emacs (eglot), Sublime, and Zed.

## Hacking on this repo

Each subproject has its own toolchain. See its `README.md`:

- `grammars/` — pure JSON; no build
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.70+

A single tag triggers a release of all four (see [`.github/workflows/release.yml`](.github/workflows/release.yml)).

## Versioning

Single semver across the monorepo: `0.1.0`, `0.1.1`, `0.2.0`. All four
subprojects publish under the same tag at the same time. The
CHANGELOG lists changes per subproject under each version heading.

## License

MIT. See [LICENSE](LICENSE).

## Other Ktav implementations

- [`spec`](https://github.com/ktav-lang/spec) — specification + conformance suite
- [`rust`](https://github.com/ktav-lang/rust) — reference Rust crate (`cargo add ktav`)
- [`csharp`](https://github.com/ktav-lang/csharp) — C# / .NET (`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) — Go (`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) — Java / JVM (`io.github.ktav-lang:ktav` on Maven Central)
- [`js`](https://github.com/ktav-lang/js) — JS / TS (`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) — PHP (`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) — Python (`pip install ktav`)
