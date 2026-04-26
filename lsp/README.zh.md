# ktav-lsp

> [Ktav](https://github.com/ktav-lang/spec) 配置格式的 Language Server
> Protocol 实现。单一 Rust 二进制;`ktav` 解析器 crate 的薄封装。

**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**

---

## 这是什么

`ktav-lsp` 是一个 LSP 服务器。编辑器通过 stdin/stdout 与它进行 JSON-RPC
通信,获取 `.ktav` 文件的诊断、hover、补全、文档符号和 semantic tokens。
它是 [`ktav`](https://crates.io/crates/ktav) crate 的薄封装 ——
所有其他 Ktav 绑定(PHP / JS / Python / Go / Java / C#)使用的同一个
解析器 —— 因此错误消息和行为完全一致。

## 安装

```bash
cargo install ktav-lsp
```

这会将 `ktav-lsp` 二进制安装到 `~/.cargo/bin/`。无需配置文件。

## 编辑器配置

### Helix (`languages.toml`)

```toml
[language-server.ktav-lsp]
command = "ktav-lsp"

[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = []
language-servers = ["ktav-lsp"]
```

### Neovim(配合 `nvim-lspconfig`)

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

文件类型检测片段:

```vim
au BufRead,BufNewFile *.ktav set filetype=ktav
```

### VS Code

使用 [Ktav VS Code 扩展](../vscode) —— 它包含语言配置并桥接
`ktav-lsp`。`ktav-lsp` 需通过 `cargo install` 单独安装。

### Emacs (`eglot`)

```elisp
(add-to-list 'auto-mode-alist '("\\.ktav\\'" . ktav-mode))
(define-derived-mode ktav-mode prog-mode "Ktav")

(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(ktav-mode . ("ktav-lsp"))))
```

## 功能

- **诊断**:每次 `did_open` / `did_change` 都会重新解析文档,并在出错
  行上显示 `ktav::Error::Syntax` 消息。
- **Hover**:在 `key:` 行悬停可显示推断的类型和值。
- **补全**:在 `:` 分隔符之后上下文感知补全:`null`、`true`、`false`、
  开括号(`{`、`[`、`(`、`((`)、空字面量(`{}`、`[]`、`()`)以及
  类型标记(`:`、`:i`、`:f`)。
- **文档符号**:大纲视图反映已解析的对象树;标量为
  Property/Number/String,对象为 Module,数组为 Array。
- **Semantic tokens**:token 类型 `comment`、`keyword`、`number`、
  `string`、`property`、`operator`。编辑器可使用它们替代(或叠加于)
  TextMate 语法,尤其在点状键和类型标记附近能获得更准确的着色。

## 架构

单一 Rust crate,单一二进制。技术栈:

- [`tower-lsp`](https://crates.io/crates/tower-lsp):JSON-RPC 与服务器
  能力管线。
- [`tokio`](https://tokio.rs/):运行时,从 stdin 读、向 stdout 写。
- [`ktav`](https://crates.io/crates/ktav):每个 Ktav 绑定都使用的同一个
  解析器 crate。诊断、hover、文档符号都走 `ktav::parse`。
- [`dashmap`](https://crates.io/crates/dashmap):按 `Url` 键的线程安全
  文档存储。`TextDocumentSyncKind::FULL` 让循环简单:小型配置文件
  重新解析足够快,增量同步只会徒增代码而不省时间。

日志写到 stderr(stdout 留给 LSP 流量)。设置 `KTAV_LSP_LOG=debug`
提高日志级别。

## 许可证

MIT —— 见 [LICENSE](LICENSE)。
