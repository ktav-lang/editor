# Helix

Add the following to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = [".git"]
comment-token = "#"
indent = { tab-width = 2, unit = "  " }
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Install the language server:

```sh
cargo install ktav-lsp
```

Verify with `:lsp-restart` after opening a `.ktav` file. Diagnostics
appear inline; hover with `K` (default keymap).

Highlighting is provided by the LSP's semantic-tokens response — no
TextMate / tree-sitter grammar is needed in Helix for cosmetic
colouring. If you want richer colouring without an LSP running, you
can drop the shared grammar from `editor/grammars/` into a custom
tree-sitter setup, but that's an upstream Helix concern.
