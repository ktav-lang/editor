>>>>> lang=en
### Neovim (with `nvim-lspconfig`)

`ktav-lsp` is not (yet) in the `lspconfig` registry, so register it
manually:

```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.ktav_lsp then
  configs.ktav_lsp = {
    default_config = {
      cmd = { "ktav-lsp" },
      filetypes = { "ktav" },
      root_dir = lspconfig.util.find_git_ancestor,
      settings = {},
    },
  }
end

lspconfig.ktav_lsp.setup({})
```

You will also want a `ftdetect` snippet:

```vim
au BufRead,BufNewFile *.ktav set filetype=ktav
```

>>>>> lang=ru
### Neovim (with `nvim-lspconfig`)

`ktav-lsp` (пока) нет в реестре `lspconfig`, поэтому регистрируем
вручную:

```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.ktav_lsp then
  configs.ktav_lsp = {
    default_config = {
      cmd = { "ktav-lsp" },
      filetypes = { "ktav" },
      root_dir = lspconfig.util.find_git_ancestor,
      settings = {},
    },
  }
end

lspconfig.ktav_lsp.setup({})
```

Также понадобится сниппет `ftdetect`:

```vim
au BufRead,BufNewFile *.ktav set filetype=ktav
```

>>>>> lang=zh
### Neovim (with `nvim-lspconfig`)

`ktav-lsp` 目前尚未进入 `lspconfig` 注册表,需手动注册:

```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.ktav_lsp then
  configs.ktav_lsp = {
    default_config = {
      cmd = { "ktav-lsp" },
      filetypes = { "ktav" },
      root_dir = lspconfig.util.find_git_ancestor,
      settings = {},
    },
  }
end

lspconfig.ktav_lsp.setup({})
```

还需要 `ftdetect` 片段:

```vim
au BufRead,BufNewFile *.ktav set filetype=ktav
```

