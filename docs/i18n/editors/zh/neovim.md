# Neovim

**Languages:** [English](../../../neovim.md) · [Русский](../ru/neovim.md) · **简体中文**

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
