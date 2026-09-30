>>>>> lang=en
### Neovim

With [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig):

```lua
vim.filetype.add({ extension = { ktav = "ktav" } })
require("lspconfig.configs").ktav = {
  default_config = {
    cmd = { "ktav-lsp" },
    filetypes = { "ktav" },
    root_dir = require("lspconfig.util").root_pattern(".git"),
    settings = {},
  },
}
require("lspconfig").ktav.setup({})
```

Then `cargo install ktav-lsp --version 0.8.0 --locked`.

>>>>> lang=ru
### Neovim

С [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig):

```lua
vim.filetype.add({ extension = { ktav = "ktav" } })
require("lspconfig.configs").ktav = {
  default_config = {
    cmd = { "ktav-lsp" },
    filetypes = { "ktav" },
    root_dir = require("lspconfig.util").root_pattern(".git"),
    settings = {},
  },
}
require("lspconfig").ktav.setup({})
```

Затем `cargo install ktav-lsp --version 0.8.0 --locked`.

>>>>> lang=zh
### Neovim

借助 [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig):

```lua
vim.filetype.add({ extension = { ktav = "ktav" } })
require("lspconfig.configs").ktav = {
  default_config = {
    cmd = { "ktav-lsp" },
    filetypes = { "ktav" },
    root_dir = require("lspconfig.util").root_pattern(".git"),
    settings = {},
  },
}
require("lspconfig").ktav.setup({})
```

然后执行 `cargo install ktav-lsp --version 0.8.0 --locked`。

