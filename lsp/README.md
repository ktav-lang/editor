# ktav-lsp

> Language Server Protocol implementation for the
> [Ktav](https://github.com/ktav-lang/spec) configuration format.
> Single Rust binary; thin wrapper over the `ktav` parser crate.

**Languages:** **English** · [Русский](docs/README.ru.md) · [简体中文](docs/README.zh.md)

---

## What it is

`ktav-lsp` is an LSP server. Editors talk JSON-RPC to it over stdin/stdout
and get back diagnostics, hover, completion, document symbols, and
semantic tokens for `.ktav` files. It is a thin wrapper over the
[`ktav`](https://crates.io/crates/ktav) crate — the same parser every
other Ktav binding (PHP / JS / Python / Go / Java / C#) walks through —
so error messages and behaviour match exactly.

## Install

Source builds require Rust 1.88+ for the current locked dependency graph.
CI builds all targets on the declared minimum as well as running the
existing stable-toolchain checks. Unlocked installs may need newer Rust
if transitive dependencies raise their requirements.

```bash
cargo install ktav-lsp
```

This drops a `ktav-lsp` binary into `~/.cargo/bin/`. No configuration file
required.

An ordinary install may resolve newer transitive dependencies. Once 0.8.0 is
published, reproduce that release's dependency graph with:

```bash
cargo install ktav-lsp --version 0.8.0 --locked
```

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

This enables LSP diagnostics and hover, not syntax highlighting in stock
Helix. Highlighting requires a separate Ktav tree-sitter grammar and
queries, which this repo does not supply; TextMate JSON is not a substitute.

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
configuration and, on the six platforms CI ships binaries for, a
`ktav-lsp` binary too, so no separate install is needed. Install
[`ktav-lsp`](https://crates.io/crates/ktav-lsp) yourself only on an
unsupported platform, or when building the extension from source.

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
  and reports ktav's structured `ErrorKind`, mapped to a tight byte-span
  range on the offending line(s).
- **Hover** — hover on a `key:` line shows the inferred type and value.
- **Completion** — context-aware after a `:` separator: suggests `null`,
  `true`, `false`, openers (`{`, `[`, `(`, `((`), empty literals (`{}`,
  `[]`, `()`), and the value markers (`:`, `::`).
- **Document symbols** — outline view reflects the parsed object/array tree;
  scalars become Property/Number/String, objects become Module, arrays
  become Array. Declaration ranges include values, nested children and closing
  delimiters (also for multiline strings), excluding outer whitespace.
  Navigation selects only the source key or item anchor. Reopened dotted
  prefixes enclose all their definitions and keep their first navigation anchor.
- **Semantic tokens** — token types `comment`, `keyword`, `number`,
  `string`, `property`, `operator`, `null`. Editors can use these instead of (or
  layered over) TextMate grammars for more accurate colouring,
  especially around dotted keys and `::` raw values.
- **Formatting** — `textDocument/formatting` canonically re-indents
  object/array/parenthesised nesting (4 spaces per level), preserving
  blank lines, comments and multi-line string block contents verbatim.

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

MIT OR Apache-2.0. See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE).
