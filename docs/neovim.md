# Neovim

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
cargo install ktav-lsp
```

Verify with `:LspInfo` after opening a `.ktav` file — the `ktav`
client should be listed as `Active`. Diagnostics appear via
`vim.diagnostic` (default keymap `]d` / `[d`).

For highlighting without the LSP, point Neovim at the shared
`editor/grammars/ktav.tmLanguage.json` via your preferred TextMate /
tree-sitter integration plugin — the LSP's semantic tokens already
cover the common case once it's running.
