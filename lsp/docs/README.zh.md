# ktav-lsp

> [Ktav](https://github.com/ktav-lang/spec) 配置格式的 Language Server
> Protocol 实现。单一 Rust 二进制;`ktav` 解析器 crate 的薄封装。

**Languages:** [English](../README.md) · [Русский](README.ru.md) · **简体中文**

---

## 这是什么

`ktav-lsp` 是一个 LSP 服务器。编辑器通过 stdin/stdout 与它进行 JSON-RPC
通信,获取 `.ktav` 文件的诊断、hover、补全、文档符号和 semantic tokens。
它是 [`ktav`](https://crates.io/crates/ktav) crate 的薄封装 ——
所有其他 Ktav 绑定(PHP / JS / Python / Go / Java / C#)使用的同一个
解析器 —— 因此错误消息和行为完全一致。

## 安装

当前锁定依赖图的源码构建需要 Rust 1.88+。CI 在声明的最低版本上
构建所有 targets,同时保留原有 stable 工具链检查。若传递依赖提高
要求,不带 `--locked` 的安装可能需要更新的 Rust。

```bash
cargo install ktav-lsp
```

这会将 `ktav-lsp` 二进制安装到 `~/.cargo/bin/`。无需配置文件。

普通安装可能解析到更新的传递依赖。0.8.0 发布后，可用以下命令复现该版本的依赖图：

```bash
cargo install ktav-lsp --version 0.8.0 --locked
```

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

此配置启用 LSP 诊断和悬停,不会在标准 Helix 中启用语法高亮。
高亮需要独立的 Ktav tree-sitter 语法和查询,本仓库不提供它们;
TextMate JSON 不能代替它们。

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

### VS Code

使用 [Ktav VS Code 扩展](../../vscode) —— 它包含语言配置,并且在 CI
构建二进制的六个平台上还捆绑了 `ktav-lsp` 本身,因此无需单独安装。
仅在不受支持的平台上,或从源代码构建扩展时,才需要自行安装
[`ktav-lsp`](https://crates.io/crates/ktav-lsp)。

### Emacs (`eglot`)

```elisp
(add-to-list 'auto-mode-alist '("\\.ktav\\'" . ktav-mode))
(define-derived-mode ktav-mode prog-mode "Ktav")

(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(ktav-mode . ("ktav-lsp"))))
```

## 功能

- **诊断**:每次 `did_open` / `did_change` 都会重新解析文档,并报告
  `ktav` 的结构化 `ErrorKind`,精确映射到出错行(们)的字节区间。
- **Hover**:在 `key:` 行悬停可显示推断的类型和值。
- **补全**:在 `:` 分隔符之后上下文感知补全:`null`、`true`、`false`、
  开括号(`{`、`[`、`(`、`((`)、空字面量(`{}`、`[]`、`()`)以及
  值标记(`:`、`::`)。
- **文档符号**:大纲视图反映已解析的对象/数组树;标量为
  Property/Number/String,对象为 Module,数组为 Array。声明范围包含值、
  子符号和结束分隔符(多行字符串亦然),但不包含外围空白。
  导航仅选择源键或数组项的定位部分。重新打开的点状前缀覆盖所有定义,
  并保留首次出现的导航位置。
- **Semantic tokens**:token 类型 `comment`、`keyword`、`number`、
  `string`、`property`、`operator`、`null`。编辑器可使用它们替代(或叠加于)
  TextMate 语法,尤其在点状键和 `::` 原始值附近获得更准确的着色。
- **格式化**:`textDocument/formatting` 规范化对象/数组/括号分组的
  嵌套缩进(每级 4 个空格),完整保留空行、注释以及多行字符串块的
  内容。

## 架构

单一 Rust crate,单一二进制。技术栈:

- [`tower-lsp`](https://crates.io/crates/tower-lsp):JSON-RPC 与服务器
  能力管线。
- [`tokio`](https://tokio.rs/):运行时,从 stdin 读、向 stdout 写。
- [`ktav`](https://crates.io/crates/ktav):每个 Ktav 绑定都使用的同一个
  解析器 crate。诊断、hover、文档符号都走 `ktav::parse`。
- [`dashmap`](https://crates.io/crates/dashmap):按 `Url` 键的线程安全
  文档存储。`TextDocumentSyncKind::FULL` 让循环保持简单:小型配置文件
  重新解析足够快,增量同步只会徒增代码而不省时间。

日志写到 stderr(stdout 留给 LSP 流量)。设置 `KTAV_LSP_LOG=debug`
提高日志级别。

## 许可证

MIT OR Apache-2.0 —— 见 [LICENSE-MIT](../LICENSE-MIT) 与
[LICENSE-APACHE](../LICENSE-APACHE)。
