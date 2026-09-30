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
cargo install ktav-lsp --version 0.8.0 --locked
```

Проверьте `:LspInfo` после открытия файла `.ktav` — клиент `ktav`
должен отображаться как `Active`. Диагностика приходит через
`vim.diagnostic` (клавиши по умолчанию `]d` / `[d`).

Semantic-подсветка LSP зависит от поддержки и настройки клиента Neovim.
Для подсветки без LSP через tree-sitter нужны отдельные грамматика Ktav
и запросы подсветки, которых в этом репозитории нет.
`grammars/ktav.tmLanguage.json` — TextMate-грамматика, а не tree-sitter.
Ни назначение filetype, ни настройка LSP выше не устанавливают
интеграцию синтаксической подсветки.
