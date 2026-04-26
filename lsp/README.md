# ktav-lsp

> Language Server Protocol implementation for the
> [Ktav](https://github.com/ktav-lang/spec) configuration format.
> Single Rust binary; thin wrapper over the `ktav` parser crate.

**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)

---

## What it is

`ktav-lsp` is an LSP server. Editors talk JSON-RPC to it over stdin/stdout
and get back diagnostics, hover, completion, document symbols, and
semantic tokens for `.ktav` files. It is a thin wrapper over the
[`ktav`](https://crates.io/crates/ktav) crate — the same parser every
other Ktav binding (PHP / JS / Python / Go / Java / C#) walks through —
so error messages and behaviour match exactly.

## Install

```bash
cargo install ktav-lsp
```

This drops a `ktav-lsp` binary into `~/.cargo/bin/`. No configuration file
required.

## Editor setup

### Helix (`languages.toml`)

```toml
[language-server.ktav-lsp]
command = "ktav-lsp"

[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = []
language-servers = ["ktav-lsp"]
```

### Neovim (with `nvim-lspconfig`)

`ktav-lsp` is not (yet) in the `lspconfig` registry, so register it
manually:

```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.ktav_lsp then
  configs.ktav_lsp = {
    default_config = {
      cmd = { "ktav-lsp" },
      filetypes = { "ktav" },
      root_dir = lspconfig.util.find_git_ancestor,
      settings = {},
    },
  }
end

lspconfig.ktav_lsp.setup({})
```

You will also want a `ftdetect` snippet:

```vim
au BufRead,BufNewFile *.ktav set filetype=ktav
```

### VS Code

Use the [Ktav VS Code extension](../vscode) — it bundles language
configuration and bridges to `ktav-lsp` for you. Install
[`ktav-lsp`](https://crates.io/crates/ktav-lsp) separately.

### Emacs (`eglot`)

```elisp
(add-to-list 'auto-mode-alist '("\\.ktav\\'" . ktav-mode))
(define-derived-mode ktav-mode prog-mode "Ktav")

(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(ktav-mode . ("ktav-lsp"))))
```

## Features

- **Diagnostics** — every `did_open` / `did_change` re-parses the document
  and surfaces `ktav::Error::Syntax` messages on the offending line.
- **Hover** — hover on a `key:` line shows the inferred type and value.
- **Completion** — context-aware after a `:` separator: suggests `null`,
  `true`, `false`, openers (`{`, `[`, `(`, `((`), empty literals (`{}`,
  `[]`, `()`), and the typed-scalar markers (`:`, `:i`, `:f`).
- **Document symbols** — outline view reflects the parsed object tree;
  scalars become Property/Number/String, objects become Module, arrays
  become Array.
- **Semantic tokens** — token types `comment`, `keyword`, `number`,
  `string`, `property`, `operator`. Editors can use these instead of (or
  layered over) TextMate grammars for more accurate colouring,
  especially around dotted keys and typed-scalar markers.

## Architecture

Single Rust crate, single binary. Stack:

- [`tower-lsp`](https://crates.io/crates/tower-lsp) for the JSON-RPC /
  capability plumbing.
- [`tokio`](https://tokio.rs/) runtime, reading from stdin and writing
  to stdout.
- [`ktav`](https://crates.io/crates/ktav) — the same parser crate every
  Ktav binding uses. Diagnostics, hover, and document symbols all go
  through `ktav::parse`.
- [`dashmap`](https://crates.io/crates/dashmap) — thread-safe per-`Url`
  document store. `TextDocumentSyncKind::FULL` keeps the loop simple:
  small config files re-parse fast enough that incremental sync would
  add code without saving wall time.

Logs go to stderr (stdout is reserved for LSP traffic). Set
`KTAV_LSP_LOG=debug` to crank verbosity.

## License

MIT — see [LICENSE](LICENSE).
