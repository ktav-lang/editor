# ktav-lang/editor

**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**

> 为 [Ktav](https://github.com/ktav-lang/spec) 配置格式提供的编辑器
> 支持 —— 语法高亮、IDE 插件以及 Language Server。一个仓库,四个
> 子项目,共享一份 TextMate 语法。

## 仓库内容

| 子项目                  | 是什么                                                            | 发布到                                                  |
|-------------------------|------------------------------------------------------------------|---------------------------------------------------------|
| [`grammars/`](grammars/)| 共享 TextMate 语法 + VS Code language configuration              | 由 `vscode/` 与 `intellij/` 复用                        |
| [`vscode/`](vscode/)    | Visual Studio Code 扩展                                          | VS Code Marketplace + Open VSX                          |
| [`intellij/`](intellij/)| IntelliJ Platform 插件(IDEA、RustRover、GoLand …)               | JetBrains Marketplace                                   |
| [`lsp/`](lsp/)          | Language Server Protocol 实现(Rust,`tower-lsp`)                | crates.io 上的 `ktav-lsp`                               |
| [`docs/`](docs/)        | Helix / Neovim / Emacs 等编辑器的接入片段                        | —                                                       |

## Ktav 用户能得到什么

- **语法高亮** —— 键、标量、类型化标记体(`:i` / `:f`)、原始字符串
  标记(`::`)、多行字符串、注释
- **括号匹配 + 自动闭合** —— `{}` `[]` `()`
- **注释切换** —— `Ctrl/Cmd+/` → `# 注释`
- **实时诊断**(配合 LSP)—— 每一个 `MissingSeparatorSpace`、重复
  键、dotted-前缀冲突都会以红色波浪线显示在出错行上,信息与解析器
  发出的完全一致
- **悬停提示**(配合 LSP)—— 光标处键的 dotted 路径、值的推断类型
- **补全**(配合 LSP)—— 关键字(`null` / `true` / `false`)、标记
  (`:i` / `:f` / `::`)、复合体开括号
- **文档符号**(配合 LSP)—— outline 反映 `Value::Object` 的结构

## 架构

```
                    ┌─────────────────────────────────────────┐
                    │  ktav-lsp  (本仓库的 Rust 二进制)      │
                    │  • 通过 `ktav` crate 解析              │
                    │  • diagnostics、hover、completion、    │
                    │    semantic tokens、document symbols   │
                    └─────────────────────────────────────────┘
                                       ▲
                                       │ LSP(stdio 上的 JSON-RPC)
                ┌──────────────────────┼──────────────────────┐
                │                      │                      │
       ┌────────┴────────┐   ┌─────────┴─────────┐   ┌────────┴────────┐
       │ VS Code         │   │ IntelliJ Platform │   │ Helix / Neovim  │
       │ ext (`vscode/`) │   │ plugin (`intellij/`)│  │ + LSP 配置      │
       │                 │   │                     │  │                 │
       │ TextMate 语法   │   │ TextMate 语法       │  │ (LSP semantic   │
       │ 来自 grammars/  │   │ 来自 grammars/      │  │  tokens 用于    │
       │                 │   │                     │  │  高亮)          │
       └─────────────────┘   └─────────────────────┘  └─────────────────┘
```

TextMate 语法即时给出表层高亮(无需 LSP)。LSP 层增加*智能*功能 ——
诊断、悬停、补全。两层叠加:只装扩展就有高亮;把 `ktav-lsp` 加入
PATH 即获得其余功能。

## 用户安装

### VS Code

```
ext install ktav-lang.ktav
```

扩展自动捆绑主流平台的 LSP 服务器 —— 无需额外步骤。

### IntelliJ IDEA / RustRover / GoLand 等

Plugins → Marketplace → 搜索 **Ktav** → Install。

### Helix

在 `~/.config/helix/languages.toml` 中:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

然后 `cargo install ktav-lsp`。

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

然后 `cargo install ktav-lsp`。

### 其他编辑器

参见 [`docs/`](docs/),包含 Emacs(eglot)、Sublime、Zed 的配置。

## 开发

每个子项目都有自己的工具链。详见各子项目下的 `README.md`:

- `grammars/` —— 纯 JSON,无需构建
- `vscode/` —— Node + `vsce`
- `intellij/` —— JDK 17 + Gradle
- `lsp/` —— Rust 1.70+

一个 tag 触发所有四个子项目同时发布
(参见 [`.github/workflows/release.yml`](.github/workflows/release.yml))。

## 版本

整个 monorepo 共用一个 semver:`0.1.0`、`0.1.1`、`0.2.0`。四个子项目
在同一个 tag 下同时发布。CHANGELOG 在每个版本标题下按子项目列出
变更。

## 许可证

MIT。详见 [LICENSE](LICENSE)。

## 其他 Ktav 实现

- [`spec`](https://github.com/ktav-lang/spec) —— 规范 + 一致性测试套件
- [`rust`](https://github.com/ktav-lang/rust) —— 参考 Rust crate(`cargo add ktav`)
- [`csharp`](https://github.com/ktav-lang/csharp) —— C# / .NET(`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) —— Go(`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) —— Java / JVM(`io.github.ktav-lang:ktav`,Maven Central)
- [`js`](https://github.com/ktav-lang/js) —— JS / TS(`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) —— PHP(`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) —— Python(`pip install ktav`)
