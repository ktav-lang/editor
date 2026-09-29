# ktav-lang/editor

[![Release](https://img.shields.io/github/v/release/ktav-lang/editor?style=flat-square&sort=semver&label=release)](https://github.com/ktav-lang/editor/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/editor/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/editor/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

**Languages:** [English](../../README.md) · [Русский](README.ru.md) · **简体中文**

**演练场：** 在浏览器中互转 JSON / YAML / TOML / INI ⇄ Ktav —— **[ktav-lang.github.io](https://ktav-lang.github.io/)**。

> 为 [Ktav](https://github.com/ktav-lang/spec) 配置格式提供的编辑器
> 支持 —— 语法高亮、IDE 插件以及 Language Server。一个仓库、四个
> 子项目、共享一份 TextMate 语法。

## 包含内容

| 子项目                  | 是什么                                          | 发布到                                  |
|-------------------------|-------------------------------------------------|-----------------------------------------|
| [`grammars/`](../../grammars/) | 共享 TextMate 语法 + VS Code language configuration | 由 `vscode/` 与 `intellij/` 复用        |
| [`vscode/`](../../vscode/)    | Visual Studio Code 扩展                          | VS Code Marketplace + Open VSX          |
| [`intellij/`](../../intellij/)| IntelliJ Platform 插件(IDEA、RustRover、GoLand …) | JetBrains Marketplace                   |
| [`lsp/`](../../lsp/)          | Language Server Protocol 实现(Rust、`tower-lsp`)  | crates.io 上的 `ktav-lsp`               |
| [`docs/`](../../docs/)        | Helix / Neovim / Emacs / Sublime / Zed 等编辑器的接入片段 | —                              |

## Ktav 用户能得到什么

LSP 功能取决于编辑器的客户端。IntelliJ 插件仅集成实时诊断和
整文件格式化,未集成悬停、补全、文档符号或语义令牌。
其语法高亮由原生词法分析器提供。

- **语法高亮** —— 键、标量、原始字符串标记(`::`)、多行字符串、注释
- **括号匹配 + 自动闭合** —— `{}` `[]` `()`
- **注释切换** —— `Ctrl/Cmd+/` → `## comment`
- **实时诊断**(配合 LSP)—— 每一个 `MissingSeparatorSpace`、重复键、
  dotted-前缀冲突都会以红色波浪线显示在出错行上,信息与解析器发出的
  完全一致
- **悬停提示**(配合 LSP)—— 光标处键的 dotted 路径、值的推断类型
- **补全**(配合 LSP)—— 关键字(`null` / `true` / `false`)、原始字符串标记(`::`)、复合体开括号
- **文档符号**(配合 LSP)—— outline 反映 `Value::Object` 的结构

## 架构

```
                       ┌─────────────────────────────────────┐
                       │ ktav-lsp  (Rust binary, this repo)  │
                       │ • parses via the `ktav` crate       │
                       │ • diagnostics, hover, completion,   │
                       │   semantic tokens, document symbols │
                       └─────────────────────────────────────┘
                                          ▲
                                          │ LSP (JSON-RPC over stdio)
                 ┌────────────────────────┼───────────────────────┐
                 │                        │                       │
       ┌─────────┴────────┐   ┌───────────┴──────────┐   ┌────────┴───────┐
       │ VS Code          │   │ IntelliJ Platform    │   │ Helix / Neovim │
       │ ext (`vscode/`)  │   │ plugin (`intellij/`) │   │ + LSP config   │
       │                  │   │                      │   │                │
       │ TextMate grammar │   │ Native lexer         │   │ (LSP semantic  │
       │ from grammars/   │   │ (no TextMate)        │   │  tokens for    │
       │                  │   │                      │   │  highlighting) │
       └──────────────────┘   └──────────────────────┘   └────────────────┘
```

TextMate 语法(VS Code)与 IntelliJ 的原生词法分析器都能即时提供
表层高亮(无需语言服务器)。LSP 服务器提供*智能*功能,但具体集成
取决于编辑器客户端。IntelliJ 内置客户端仅提供实时诊断和整文件
格式化,未集成悬停、补全、文档符号或语义令牌。将 `ktav-lsp`
加入 PATH 不会补上这些缺失的客户端集成。

## 用户安装

### VS Code

```
ext install ktav-lang.ktav
```

扩展会自动捆绑主流平台的 LSP 服务器 —— 无需额外安装步骤。

### IntelliJ IDEA / RustRover / GoLand / etc.

插件 → Marketplace → 搜索 **Ktav** → 安装。

### Helix

在 `~/.config/helix/languages.toml` 中添加:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

然后执行 `cargo install ktav-lsp`。

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

然后执行 `cargo install ktav-lsp`。

### 其他编辑器

参见 [`docs/`](../../docs/) 中 Emacs(eglot)、Sublime 与 Zed 的配置。

## 开发本仓库

每个子项目都有自己的工具链。详见各子项目自己的 `README.md`:

- `grammars/` —— 纯 JSON,无需构建
- `vscode/` —— Node + `vsce`
- `intellij/` —— JDK 17 + Gradle
- `lsp/` —— Rust 1.71+

一个 tag 同时触发全部四个子项目的发布(参见 [`.github/workflows/release.yml`](../../.github/workflows/release.yml))。

## 版本管理

整个 monorepo 共用一个 semver。四个子项目
在同一个 tag 下同时发布。CHANGELOG 在每个版本标题下按子项目列出
变更。

## 许可证

MIT OR Apache-2.0。详见 [LICENSE-MIT](../../LICENSE-MIT) 和 [LICENSE-APACHE](../../LICENSE-APACHE)。

## 其他 Ktav 实现

- [`spec`](https://github.com/ktav-lang/spec) —— 规范 + 一致性测试套件
- [`rust`](https://github.com/ktav-lang/rust) —— 参考 Rust crate(`cargo add ktav`)
- [`csharp`](https://github.com/ktav-lang/csharp) —— C# / .NET(`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) —— Go(`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) —— Java / JVM(`io.github.ktav-lang:ktav`,Maven Central)
- [`js`](https://github.com/ktav-lang/js) —— JS / TS(`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) —— PHP(`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) —— Python(`pip install ktav`)
