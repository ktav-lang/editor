# Helix

**Languages:** **English** · [Русский](i18n/editors/ru/helix.md) · [简体中文](i18n/editors/zh/helix.md)

Add the following to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = [".git"]
comment-token = "##"
indent = { tab-width = 2, unit = "  " }
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Install the language server (Rust 1.88+ for the locked 0.8.0 build):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Verify with `:lsp-restart` after opening a `.ktav` file. Diagnostics
appear inline; hover with `K` (default keymap).

Stock Helix does not support LSP semantic-token highlighting. This
configuration enables LSP features such as diagnostics and hover, not
syntax colouring. Highlighting requires a separately implemented Ktav
tree-sitter grammar and highlight queries; this repository does not
provide them. `grammars/ktav.tmLanguage.json` is a TextMate grammar,
not a tree-sitter grammar, and cannot be installed as one.
See [Helix language configuration](https://docs.helix-editor.com/languages.html#tree-sitter-grammar-configuration).
