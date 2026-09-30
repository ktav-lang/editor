# Neovim

**Languages:** **English** · [Русский](i18n/editors/ru/neovim.md) · [简体中文](i18n/editors/zh/neovim.md)

With [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig)
(Neovim 0.9+):

```lua
-- Recognise the file extension.
vim.filetype.add({ extension = { ktav = "ktav" } })

-- Register the LSP definition (one-time).
require("lspconfig.configs").ktav = {
  default_config = {
    cmd = { "ktav-lsp" },
    filetypes = { "ktav" },
    root_dir = require("lspconfig.util").root_pattern(".git"),
    settings = {},
  },
}

-- Attach.
require("lspconfig").ktav.setup({})
```

Install the server:

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Verify with `:LspInfo` after opening a `.ktav` file — the `ktav`
client should be listed as `Active`. Diagnostics appear via
`vim.diagnostic` (default keymap `]d` / `[d`).

LSP semantic highlighting depends on the Neovim client's support and
configuration. For highlighting without LSP, tree-sitter needs a
separate Ktav grammar and highlight queries, which this repository
does not provide. `grammars/ktav.tmLanguage.json` is a TextMate grammar,
not a tree-sitter grammar; neither the filetype nor the LSP setup above
installs a syntax-highlighting integration.
