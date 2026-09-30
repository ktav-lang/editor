>>>>> lang=en
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
>>>>> lang=ru
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
>>>>> lang=zh
借助 [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig)
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

安装服务器:

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

打开 `.ktav` 文件后用 `:LspInfo` 验证 —— `ktav` 客户端应显示为
`Active`。诊断信息通过 `vim.diagnostic` 呈现(默认按键 `]d` / `[d`)。

LSP 语义高亮取决于 Neovim 客户端的支持和配置。
不使用 LSP 时,tree-sitter 高亮需要独立的 Ktav 语法和高亮查询,
本仓库不提供这些文件。`grammars/ktav.tmLanguage.json` 是 TextMate
语法,不是 tree-sitter 语法;上面的 filetype 和 LSP 配置都不会安装
语法高亮集成。
