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
cargo install ktav-lsp
```

打开 `.ktav` 文件后用 `:LspInfo` 验证 —— `ktav` 客户端应显示为
`Active`。诊断信息通过 `vim.diagnostic` 呈现(默认按键 `]d` / `[d`)。

若不运行 LSP 也想要高亮,可以让 Neovim 通过你偏好的 TextMate /
tree-sitter 集成插件指向共享的
`editor/grammars/ktav.tmLanguage.json` —— LSP 的 semantic tokens
运行时已覆盖常见情况。
