# Neovim

**Languages:** [English](../../../neovim.md) · **Русский** · [简体中文](../zh/neovim.md)

С [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig)
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

Установите сервер:

```sh
cargo install ktav-lsp
```

Проверьте `:LspInfo` после открытия файла `.ktav` — клиент `ktav`
должен отображаться как `Active`. Диагностика приходит через
`vim.diagnostic` (клавиши по умолчанию `]d` / `[d`).

Для подсветки без LSP направьте Neovim на общую грамматику
`editor/grammars/ktav.tmLanguage.json` через предпочитаемый плагин
интеграции TextMate / tree-sitter — semantic tokens LSP и так
покрывают основной случай, когда он запущен.
